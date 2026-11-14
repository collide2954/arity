use arity_core::{License, Notice, NoticeKind, NoticeSource, Pmcid};
use jiff::Timestamp;
use serde::Deserialize;

use crate::http::{Http, SourceError};

/// The open-access store of PubMed Central, which NCBI keeps on AWS.
pub const BUCKET: &str = "https://pmc-oa-opendata.s3.amazonaws.com";
const S3_PREFIX: &str = "s3://pmc-oa-opendata/";

/// The record of one version of an article in the store.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PmcRecord {
    pub pmcid: String,
    pub version: u32,
    pub pmid: Option<u64>,
    pub doi: Option<String>,
    pub title: Option<String>,
    pub citation: Option<String>,
    #[serde(default)]
    pub is_pmc_openaccess: bool,
    #[serde(default)]
    pub is_manuscript: bool,
    #[serde(default)]
    pub is_retracted: bool,
    pub license_code: Option<String>,
    pub pdf_url: Option<String>,
    pub xml_url: Option<String>,
    pub text_url: Option<String>,
    #[serde(default)]
    pub media_urls: Vec<String>,
}

pub fn list_url(pmcid: &Pmcid) -> String {
    format!("{BUCKET}/?list-type=2&prefix={pmcid}.&delimiter=/")
}

pub fn record_url(pmcid: &Pmcid, version: u32) -> String {
    format!("{BUCKET}/{pmcid}.{version}/{pmcid}.{version}.json")
}

/// Reads the versions of an article from a listing of the store.
pub fn parse_versions(xml: &str, pmcid: &Pmcid) -> Result<Vec<u32>, String> {
    let document = roxmltree::Document::parse(xml).map_err(|error| error.to_string())?;
    let head = format!("{pmcid}.");
    let mut versions: Vec<u32> = document
        .descendants()
        .filter(|node| node.tag_name().name() == "Prefix")
        .filter(|node| {
            node.parent()
                .is_some_and(|parent| parent.tag_name().name() == "CommonPrefixes")
        })
        .filter_map(|node| node.text())
        .filter_map(|prefix| prefix.strip_prefix(&head)?.strip_suffix('/')?.parse().ok())
        .collect();
    versions.sort_unstable();
    Ok(versions)
}

/// The record of the newest version of an article, or nothing when the
/// store does not have the article.
pub fn latest_record(http: &dyn Http, pmcid: &Pmcid) -> Result<Option<PmcRecord>, SourceError> {
    let url = list_url(pmcid);
    let listing = http.get_ok(&url)?;
    let text = String::from_utf8_lossy(&listing.body);
    let versions =
        parse_versions(&text, pmcid).map_err(|message| SourceError::Invalid { url, message })?;
    let Some(version) = versions.last() else {
        return Ok(None);
    };
    let url = record_url(pmcid, *version);
    let response = http.get_ok(&url)?;
    let record = serde_json::from_slice(&response.body).map_err(|error| SourceError::Invalid {
        url,
        message: error.to_string(),
    })?;
    Ok(Some(record))
}

/// Gives the HTTPS address of a file that the store names with an `s3://`
/// address.
pub fn https_url(s3_url: &str) -> Option<String> {
    let key = s3_url.strip_prefix(S3_PREFIX)?;
    let key = key.split('?').next()?;
    (!key.is_empty()).then(|| format!("{BUCKET}/{key}"))
}

/// The MD5 digest that the store gives for a file.
pub fn md5_of(s3_url: &str) -> Option<&str> {
    s3_url
        .split_once("?md5=")
        .map(|(_, md5)| md5)
        .filter(|md5| md5.len() == 32 && md5.bytes().all(|b| b.is_ascii_hexdigit()))
}

impl PmcRecord {
    /// The version identifier, for example `PMC10538757.1`.
    pub fn version_id(&self) -> String {
        format!("{}.{}", self.pmcid, self.version)
    }

    pub fn license(&self) -> Option<License> {
        self.license_code
            .as_deref()
            .and_then(|code| code.parse().ok())
            .filter(|license| !matches!(license, License::Unknown(_)))
    }

    /// The address of a media file, for example a supplementary file that
    /// the article names.
    pub fn media_url(&self, file_name: &str) -> Option<String> {
        let suffix = format!("/{file_name}");
        self.media_urls
            .iter()
            .find(|url| {
                url.split('?')
                    .next()
                    .is_some_and(|key| key.ends_with(&suffix))
            })
            .and_then(|url| https_url(url))
    }

    pub fn notices(&self, now: Timestamp) -> Vec<Notice> {
        if !self.is_retracted {
            return Vec::new();
        }
        vec![Notice {
            kind: NoticeKind::Retraction,
            reported_type: "is_retracted".to_owned(),
            source: NoticeSource::Pmc,
            notice_doi: None,
            notice_pmid: None,
            notice_citation: None,
            date: None,
            source_record: Some(self.version_id()),
            retrieved_at: now,
        }]
    }
}
