use std::sync::LazyLock;

use arity_core::{Author, License, Notice, NoticeKind, NoticeSource, WorkMetadata};
use jiff::Timestamp;
use regex::Regex;
use serde::Deserialize;

use crate::http::{Http, SourceError};

pub const SEARCH_URL: &str = "https://www.ebi.ac.uk/europepmc/webservices/rest/search";

/// A work that a search found, with the notices that Europe PMC links to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    pub metadata: WorkMetadata,
    pub notices: Vec<Notice>,
    pub open_access: bool,
    pub in_pmc: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchPage {
    pub hit_count: u64,
    pub found: Vec<Found>,
    pub next_cursor: Option<String>,
}

/// The query for open-access works in PubMed Central that match a topic.
pub fn open_access_query(topic: &str) -> String {
    format!("({topic}) AND OPEN_ACCESS:y AND IN_PMC:y")
}

pub fn search_url(query: &str, page_size: u32, cursor: &str) -> String {
    let size = page_size.to_string();
    url::Url::parse_with_params(
        SEARCH_URL,
        [
            ("query", query),
            ("format", "json"),
            ("resultType", "core"),
            ("pageSize", size.as_str()),
            ("cursorMark", cursor),
        ],
    )
    .map_or_default(String::from)
}

pub fn search(
    http: &dyn Http,
    query: &str,
    page_size: u32,
    cursor: &str,
    now: Timestamp,
) -> Result<SearchPage, SourceError> {
    let url = search_url(query, page_size, cursor);
    let response = http.get_ok(&url)?;
    parse_search(&response.body, now).map_err(|message| SourceError::Invalid { url, message })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchResponse {
    hit_count: Option<u64>,
    next_cursor_mark: Option<String>,
    result_list: Option<ResultList>,
}

#[derive(Deserialize)]
struct ResultList {
    #[serde(default)]
    result: Vec<Item>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Item {
    #[serde(default)]
    source: String,
    pmid: Option<String>,
    pmcid: Option<String>,
    doi: Option<String>,
    #[serde(default)]
    title: String,
    author_list: Option<AuthorList>,
    journal_info: Option<JournalInfo>,
    pub_year: Option<String>,
    first_publication_date: Option<String>,
    pub_type_list: Option<PubTypeList>,
    comment_correction_list: Option<CommentCorrectionList>,
    is_open_access: Option<String>,
    #[serde(rename = "inPMC")]
    in_pmc: Option<String>,
    license: Option<String>,
}

#[derive(Deserialize)]
struct AuthorList {
    #[serde(default)]
    author: Vec<AuthorItem>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthorItem {
    full_name: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    collective_name: Option<String>,
    author_id: Option<AuthorId>,
}

#[derive(Deserialize)]
struct AuthorId {
    #[serde(rename = "type")]
    kind: Option<String>,
    value: Option<String>,
}

#[derive(Deserialize)]
struct JournalInfo {
    journal: Option<Journal>,
}

#[derive(Deserialize)]
struct Journal {
    title: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PubTypeList {
    pub_type: Option<OneOrMany>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommentCorrectionList {
    #[serde(default)]
    comment_correction: Vec<CommentCorrection>,
}

#[derive(Deserialize)]
struct CommentCorrection {
    id: Option<String>,
    source: Option<String>,
    reference: Option<String>,
    #[serde(rename = "type")]
    kind: Option<String>,
}

static TAGS: LazyLock<Regex> = LazyLock::new(|| crate::compile(r"<[^>]+>"));
static DOI_IN_TEXT: LazyLock<Regex> = LazyLock::new(|| crate::compile(r"(?i)doi:\s*(10\.\S+)"));

fn plain(text: &str) -> String {
    TAGS.replace_all(text, "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// The kind of notice for a link type of Europe PMC, for the links that are
/// notices about the work itself.
pub fn notice_kind(link_type: &str) -> Option<NoticeKind> {
    match link_type.trim().to_ascii_lowercase().as_str() {
        "retraction in" => Some(NoticeKind::Retraction),
        "partial retraction in" => Some(NoticeKind::PartialRetraction),
        "erratum in" => Some(NoticeKind::Erratum),
        "expression of concern in" => Some(NoticeKind::ExpressionOfConcern),
        "corrected and republished in" => Some(NoticeKind::Correction),
        "update in" => Some(NoticeKind::NewVersion),
        "republished in" => Some(NoticeKind::NewEdition),
        _ => None,
    }
}

fn notices(item: &Item, pub_types: &[String], now: Timestamp) -> Vec<Notice> {
    let mut notices: Vec<Notice> = item
        .comment_correction_list
        .iter()
        .flat_map(|list| &list.comment_correction)
        .filter_map(|link| {
            let reported_type = link.kind.clone()?;
            let kind = notice_kind(&reported_type)?;
            let citation = link.reference.clone();
            let notice_doi = citation
                .as_deref()
                .and_then(|text| DOI_IN_TEXT.captures(text))
                .and_then(|captures| captures.get(1))
                .and_then(|doi| doi.as_str().trim_end_matches(['.', ',', ';']).parse().ok());
            let notice_pmid = (link.source.as_deref() == Some("MED"))
                .then(|| link.id.as_deref().and_then(|id| id.parse().ok()))
                .flatten();
            Some(Notice {
                kind,
                reported_type,
                source: NoticeSource::EuropePmc,
                notice_doi,
                notice_pmid,
                notice_citation: citation,
                date: None,
                source_record: None,
                retrieved_at: now,
            })
        })
        .collect();
    let flagged = pub_types
        .iter()
        .any(|kind| kind.eq_ignore_ascii_case("Retracted Publication"));
    if flagged
        && !notices
            .iter()
            .any(|notice| notice.kind == NoticeKind::Retraction)
    {
        notices.push(Notice {
            kind: NoticeKind::Retraction,
            reported_type: "Retracted Publication".to_owned(),
            source: NoticeSource::EuropePmc,
            notice_doi: None,
            notice_pmid: None,
            notice_citation: None,
            date: None,
            source_record: None,
            retrieved_at: now,
        });
    }
    notices
}

fn author(item: &AuthorItem) -> Option<Author> {
    let name = match (&item.first_name, &item.last_name) {
        (Some(first), Some(last)) => format!("{first} {last}"),
        _ => item
            .full_name
            .clone()
            .or_else(|| item.collective_name.clone())?,
    };
    let orcid = item
        .author_id
        .as_ref()
        .filter(|id| id.kind.as_deref() == Some("ORCID"))
        .and_then(|id| id.value.as_deref())
        .and_then(|value| value.parse().ok());
    Some(Author {
        name: plain(&name),
        orcid,
    })
}

fn found(item: Item, now: Timestamp) -> Found {
    let pub_types = match item
        .pub_type_list
        .as_ref()
        .and_then(|list| list.pub_type.as_ref())
    {
        Some(OneOrMany::One(one)) => vec![one.clone()],
        Some(OneOrMany::Many(many)) => many.clone(),
        None => Vec::new(),
    };
    let is_preprint = item.source == "PPR"
        || pub_types
            .iter()
            .any(|kind| kind.eq_ignore_ascii_case("preprint"));
    let notices = notices(&item, &pub_types, now);
    let metadata = WorkMetadata {
        title: plain(&item.title),
        doi: item.doi.as_deref().and_then(|doi| doi.parse().ok()),
        pmid: item.pmid.as_deref().and_then(|pmid| pmid.parse().ok()),
        pmcid: item.pmcid.as_deref().and_then(|pmcid| pmcid.parse().ok()),
        is_preprint,
        journal: item
            .journal_info
            .and_then(|info| info.journal)
            .and_then(|journal| journal.title),
        published: item
            .first_publication_date
            .as_deref()
            .and_then(|date| date.parse().ok()),
        year: item.pub_year.as_deref().and_then(|year| year.parse().ok()),
        authors: item
            .author_list
            .map_or_default(|list| list.author.iter().filter_map(author).collect()),
        license: item
            .license
            .as_deref()
            .and_then(|license| license.parse::<License>().ok()),
        ..WorkMetadata::default()
    };
    Found {
        metadata,
        notices,
        open_access: item.is_open_access.as_deref() == Some("Y"),
        in_pmc: item.in_pmc.as_deref() == Some("Y"),
    }
}

pub fn parse_search(body: &[u8], now: Timestamp) -> Result<SearchPage, String> {
    let response: SearchResponse =
        serde_json::from_slice(body).map_err(|error| error.to_string())?;
    let found: Vec<Found> = response
        .result_list
        .map_or_default(|list| list.result)
        .into_iter()
        .map(|item| found(item, now))
        .collect();
    Ok(SearchPage {
        hit_count: response.hit_count.unwrap_or(0),
        next_cursor: response.next_cursor_mark,
        found,
    })
}
