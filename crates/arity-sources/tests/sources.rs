#![cfg(test)]

use std::collections::HashMap;

use arity_core::{CcElements, License, NoticeKind, NoticeSource, Pmcid};
use arity_sources::{Http, Response, SourceError, crossref, europepmc, pmc};
use jiff::Timestamp;

struct Fake(HashMap<String, Response>);

impl Fake {
    fn with(entries: &[(&str, u16, &[u8])]) -> Self {
        Self(
            entries
                .iter()
                .map(|(url, status, body)| {
                    (
                        (*url).to_owned(),
                        Response {
                            status: *status,
                            content_type: None,
                            body: body.to_vec(),
                        },
                    )
                })
                .collect(),
        )
    }
}

impl Http for Fake {
    fn get(&self, url: &str) -> Result<Response, SourceError> {
        self.0
            .get(url)
            .cloned()
            .ok_or_else(|| SourceError::Request {
                url: url.to_owned(),
                message: "no stored response".to_owned(),
            })
    }
}

const SEARCH: &[u8] = include_bytes!("../../../testdata/sources/europepmc-search.json");
const CROSSREF: &[u8] = include_bytes!("../../../testdata/sources/crossref-work.json");
const RECORD: &[u8] = include_bytes!("../../../testdata/sources/pmc-record.json");
const LISTING: &[u8] = include_bytes!("../../../testdata/sources/pmc-list.xml");

fn now() -> Timestamp {
    "2026-11-14T10:00:00Z".parse().unwrap()
}

#[test]
fn reads_works_from_a_search() {
    let page = europepmc::parse_search(SEARCH, now()).unwrap();
    assert_eq!(page.hit_count, 2);
    let open = page
        .found
        .iter()
        .find(|found| found.metadata.pmcid.is_some())
        .unwrap();
    let work = &open.metadata;
    assert_eq!(work.pmcid.as_ref().unwrap().as_str(), "PMC10538757");
    assert_eq!(
        work.doi.as_ref().unwrap().as_str(),
        "10.1371/journal.pone.0291981"
    );
    assert_eq!(work.year, Some(2023));
    assert_eq!(work.published, Some(jiff::civil::date(2023, 9, 28)));
    assert_eq!(work.authors[0].name, "Zainab Lafi");
    assert_eq!(
        work.license,
        Some(License::CreativeCommons {
            elements: CcElements::By,
            version: None
        })
    );
    assert!(open.open_access && open.in_pmc);
    assert!(!work.is_preprint);
    assert_eq!(open.notices, []);
}

#[test]
fn reads_titles_without_markup() {
    let body = br#"{"hitCount": 2, "resultList": {"result": [
        {"source": "MED", "title": "From molecular design to &lt;i&gt;in vivo&lt;/i&gt; validation."},
        {"source": "MED", "title": "Conjugates of <i>o</i>-phenylenediamine &amp; IC<sub>50</sub> &lt; 10&#160;&#x3bc;M"}
    ]}}"#;
    let page = europepmc::parse_search(body, now()).unwrap();
    let titles: Vec<&str> = page
        .found
        .iter()
        .map(|found| found.metadata.title.as_str())
        .collect();
    assert_eq!(
        titles,
        [
            "From molecular design to in vivo validation.",
            "Conjugates of o-phenylenediamine & IC50 < 10 μM",
        ]
    );
}

#[test]
fn reads_retraction_and_concern_links_but_not_comments() {
    let page = europepmc::parse_search(SEARCH, now()).unwrap();
    let retracted = page
        .found
        .iter()
        .find(|found| {
            found
                .metadata
                .pmid
                .as_ref()
                .is_some_and(|pmid| pmid.as_str() == "9500320")
        })
        .unwrap();
    let kinds: Vec<(NoticeKind, &str)> = retracted
        .notices
        .iter()
        .map(|notice| (notice.kind, notice.reported_type.as_str()))
        .collect();
    assert_eq!(
        kinds,
        [
            (NoticeKind::Retraction, "Retraction in"),
            (NoticeKind::Retraction, "Retraction in"),
            (NoticeKind::ExpressionOfConcern, "Expression of concern in"),
        ]
    );
    let last = &retracted.notices[1];
    assert_eq!(last.source, NoticeSource::EuropePmc);
    assert_eq!(last.notice_pmid.as_ref().unwrap().as_str(), "20137807");
    assert_eq!(
        last.notice_doi.as_ref().unwrap().as_str(),
        "10.1016/s0140-6736(10)60175-4"
    );
    assert!(
        last.notice_citation
            .as_deref()
            .unwrap()
            .starts_with("Lancet. 2010")
    );
}

#[test]
fn reads_updates_from_crossref_with_their_sources() {
    let notices = crossref::parse_notices(CROSSREF, now()).unwrap();
    let summary: Vec<(NoticeKind, NoticeSource, Option<String>)> = notices
        .iter()
        .map(|notice| {
            (
                notice.kind,
                notice.source,
                notice.date.map(|d| d.to_string()),
            )
        })
        .collect();
    assert_eq!(
        summary,
        [
            (
                NoticeKind::Correction,
                NoticeSource::RetractionWatch,
                Some("2004-03-06".to_owned())
            ),
            (
                NoticeKind::Retraction,
                NoticeSource::RetractionWatch,
                Some("2010-02-06".to_owned())
            ),
        ]
    );
    assert_eq!(notices[1].source_record.as_deref(), Some("4036"));
}

#[test]
fn asks_crossref_for_a_doi_and_accepts_an_unknown_doi() {
    let doi = "10.1016/S0140-6736(97)11096-0".parse().unwrap();
    let url = crossref::work_url(&doi, None);
    assert_eq!(
        url,
        "https://api.crossref.org/works/10.1016%2Fs0140-6736(97)11096-0"
    );
    assert_eq!(
        crossref::work_url(&doi, Some("someone@example.org")),
        format!("{url}?mailto=someone%40example.org")
    );
    let fake = Fake::with(&[(url.as_str(), 404, b"Resource not found.")]);
    assert_eq!(crossref::notices(&fake, &doi, None, now()).unwrap(), []);
    let fake = Fake::with(&[(url.as_str(), 200, CROSSREF)]);
    assert_eq!(
        crossref::notices(&fake, &doi, None, now()).unwrap().len(),
        2
    );
}

#[test]
fn finds_the_newest_version_in_the_pmc_store() {
    let pmcid: Pmcid = "PMC10538757".parse().unwrap();
    let fake = Fake::with(&[
        (pmc::list_url(&pmcid).as_str(), 200, LISTING),
        (pmc::record_url(&pmcid, 1).as_str(), 200, RECORD),
    ]);
    let record = pmc::latest_record(&fake, &pmcid).unwrap().unwrap();
    assert_eq!(record.version_id(), "PMC10538757.1");
    assert_eq!(
        record.license(),
        Some(License::CreativeCommons {
            elements: CcElements::By,
            version: None
        })
    );
    assert!(!record.is_retracted);
    assert_eq!(record.notices(now()), []);
    assert_eq!(
        pmc::https_url(record.pdf_url.as_deref().unwrap()).unwrap(),
        "https://pmc-oa-opendata.s3.amazonaws.com/PMC10538757.1/PMC10538757.1.pdf"
    );
    assert_eq!(
        pmc::md5_of(record.pdf_url.as_deref().unwrap()),
        Some("a174bd1041b3f55fba4e787c5c3f55a0")
    );
    assert_eq!(
        record.media_url("pone.0291981.g003.jpg").unwrap(),
        "https://pmc-oa-opendata.s3.amazonaws.com/PMC10538757.1/pone.0291981.g003.jpg"
    );
    assert_eq!(record.media_url("missing.xlsx"), None);
}

#[test]
fn reports_a_retraction_flag_of_pmc() {
    let mut record: pmc::PmcRecord = serde_json::from_slice(RECORD).unwrap();
    record.is_retracted = true;
    let notices = record.notices(now());
    assert_eq!(notices.len(), 1);
    assert_eq!(notices[0].source, NoticeSource::Pmc);
    assert_eq!(notices[0].reported_type, "is_retracted");
}

#[test]
fn gives_nothing_for_an_article_that_the_store_does_not_have() {
    let pmcid: Pmcid = "PMC1".parse().unwrap();
    let empty = br#"<?xml version="1.0" encoding="UTF-8"?><ListBucketResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/"><KeyCount>0</KeyCount></ListBucketResult>"#;
    let fake = Fake::with(&[(pmc::list_url(&pmcid).as_str(), 200, empty)]);
    assert_eq!(pmc::latest_record(&fake, &pmcid).unwrap(), None);
}

#[test]
fn builds_search_queries_for_open_access_works() {
    assert_eq!(
        europepmc::open_access_query("doxorubicin AND MCF-7"),
        "(doxorubicin AND MCF-7) AND OPEN_ACCESS:y AND IN_PMC:y"
    );
    let url = europepmc::search_url("a b", 25, "*");
    assert!(url.starts_with("https://www.ebi.ac.uk/europepmc/webservices/rest/search?query=a+b"));
    assert!(url.contains("pageSize=25"));
    assert!(url.ends_with("cursorMark=*"));
}
