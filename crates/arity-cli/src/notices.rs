use arity_core::Notice;
use arity_sources::{crossref, europepmc, pmc};
use arity_store::WorkId;
use jiff::Timestamp;

use crate::catalogue::identifier_query;
use crate::context::Context;
use crate::error::Result;

/// Reads the notices about a work again from each source, and stores the
/// new ones. Each notice keeps its source; arity does not merge them.
pub fn refresh(context: &Context, work: WorkId, now: Timestamp) -> Result<Vec<Notice>> {
    let metadata = context.store.work(work)?.metadata;
    let http = context.http.as_ref();
    let mut notices = Vec::new();
    let identifier = metadata
        .pmcid
        .as_ref()
        .map(ToString::to_string)
        .or_else(|| metadata.doi.as_ref().map(ToString::to_string))
        .or_else(|| metadata.pmid.as_ref().map(ToString::to_string));
    if let Some(identifier) = identifier {
        let query = identifier_query(&identifier)?;
        let page = europepmc::search(http, &query, 1, "*", now)?;
        if let Some(found) = page.found.first() {
            notices.extend(found.notices.iter().cloned());
        }
    }
    if let Some(doi) = &metadata.doi {
        let contact = context.config.http.contact.as_deref();
        notices.extend(crossref::notices(http, doi, contact, now)?);
    }
    if let Some(pmcid) = &metadata.pmcid
        && let Some(record) = pmc::latest_record(http, pmcid)?
    {
        notices.extend(record.notices(now));
    }
    let mut added = Vec::new();
    for notice in notices {
        if context.store.add_notice(work, &notice)? {
            added.push(notice);
        }
    }
    Ok(added)
}
