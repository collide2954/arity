use arity_core::{Doi, Notice, NoticeKind, NoticeSource};
use jiff::Timestamp;
use jiff::civil::Date;
use serde::Deserialize;

use crate::http::{Http, SourceError};

pub const WORKS_URL: &str = "https://api.crossref.org/works/";

/// The address of the Crossref record of a work. Crossref asks clients to
/// give a contact address; arity sends one only when the operator sets it.
pub fn work_url(doi: &Doi, contact: Option<&str>) -> String {
    let Ok(mut url) = url::Url::parse(WORKS_URL) else {
        return String::new();
    };
    if let Ok(mut segments) = url.path_segments_mut() {
        segments.pop_if_empty().push(doi.as_str());
    }
    if let Some(contact) = contact {
        url.query_pairs_mut().append_pair("mailto", contact);
    }
    url.into()
}

/// The kind of notice for an update type of Crossref.
pub fn notice_kind(update_type: &str) -> NoticeKind {
    let normalized: String = update_type
        .trim()
        .to_ascii_lowercase()
        .chars()
        .map(|c| if c == '-' || c == ' ' { '_' } else { c })
        .collect();
    match normalized.as_str() {
        "retraction" => NoticeKind::Retraction,
        "partial_retraction" => NoticeKind::PartialRetraction,
        "withdrawal" => NoticeKind::Withdrawal,
        "removal" => NoticeKind::Removal,
        "expression_of_concern" => NoticeKind::ExpressionOfConcern,
        "correction" => NoticeKind::Correction,
        "erratum" => NoticeKind::Erratum,
        "corrigendum" => NoticeKind::Corrigendum,
        "addendum" => NoticeKind::Addendum,
        "clarification" => NoticeKind::Clarification,
        "new_version" => NoticeKind::NewVersion,
        "new_edition" => NoticeKind::NewEdition,
        "reinstatement" => NoticeKind::Reinstatement,
        _ => NoticeKind::Other,
    }
}

#[derive(Deserialize)]
struct WorkResponse {
    message: Message,
}

#[derive(Deserialize)]
struct Message {
    #[serde(rename = "updated-by", default)]
    updated_by: Vec<Update>,
}

#[derive(Deserialize)]
struct Update {
    #[serde(rename = "DOI")]
    doi: Option<String>,
    #[serde(rename = "type")]
    kind: Option<String>,
    label: Option<String>,
    source: Option<String>,
    updated: Option<DateParts>,
    #[serde(rename = "record-id")]
    record_id: Option<String>,
}

#[derive(Deserialize)]
struct DateParts {
    #[serde(rename = "date-parts", default)]
    date_parts: Vec<Vec<i64>>,
}

fn date(parts: Option<&DateParts>) -> Option<Date> {
    let parts = parts?.date_parts.first()?;
    let year = i16::try_from(*parts.first()?).ok()?;
    let month = i8::try_from(*parts.get(1)?).ok()?;
    let day = i8::try_from(*parts.get(2)?).ok()?;
    Date::new(year, month, day).ok()
}

pub fn parse_notices(body: &[u8], now: Timestamp) -> Result<Vec<Notice>, String> {
    let response: WorkResponse = serde_json::from_slice(body).map_err(|error| error.to_string())?;
    Ok(response
        .message
        .updated_by
        .into_iter()
        .filter_map(|update| {
            let reported_type = update.kind.or(update.label)?;
            let source = if update.source.as_deref() == Some("retraction-watch") {
                NoticeSource::RetractionWatch
            } else {
                NoticeSource::Crossref
            };
            Some(Notice {
                kind: notice_kind(&reported_type),
                reported_type,
                source,
                notice_doi: update.doi.as_deref().and_then(|doi| doi.parse().ok()),
                notice_pmid: None,
                notice_citation: None,
                date: date(update.updated.as_ref()),
                source_record: update.record_id,
                retrieved_at: now,
            })
        })
        .collect())
}

/// The notices that Crossref gives for a work. A DOI that Crossref does not
/// know gives no notices.
pub fn notices(
    http: &dyn Http,
    doi: &Doi,
    contact: Option<&str>,
    now: Timestamp,
) -> Result<Vec<Notice>, SourceError> {
    let url = work_url(doi, contact);
    let response = http.get(&url)?;
    match response.status {
        200 => parse_notices(&response.body, now)
            .map_err(|message| SourceError::Invalid { url, message }),
        404 => Ok(Vec::new()),
        status => Err(SourceError::Status { url, status }),
    }
}
