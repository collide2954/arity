use arity_core::{Doi, Pmcid, Pmid};
use arity_sources::europepmc::{self, Found};
use arity_store::WorkId;
use jiff::Timestamp;

use crate::context::Context;
use crate::error::{CliError, Result};

const PAGE_SIZE: usize = 100;

/// A work that the catalogue found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Added {
    pub work: WorkId,
    pub pmcid: Option<Pmcid>,
    pub title: String,
    /// False when the work was already in the queue.
    pub new: bool,
}

/// Finds open-access works for a topic and adds them to the queue, in the
/// order that the search gives them.
pub fn search(context: &Context, topic: &str, limit: usize, now: Timestamp) -> Result<Vec<Added>> {
    let query = europepmc::open_access_query(topic);
    let mut cursor = "*".to_owned();
    let mut added = Vec::new();
    while added.len() < limit {
        let size = u32::try_from((limit - added.len()).min(PAGE_SIZE)).unwrap_or(1);
        let page = europepmc::search(context.http.as_ref(), &query, size, &cursor, now)?;
        if page.found.is_empty() {
            break;
        }
        for found in &page.found {
            if added.len() >= limit {
                break;
            }
            added.push(record(context, found, Some(topic), now)?);
        }
        match page.next_cursor {
            Some(next) if next != cursor => cursor = next,
            _ => break,
        }
    }
    Ok(added)
}

/// The Europe PMC query that finds one work by its identifier.
pub fn identifier_query(identifier: &str) -> Result<String> {
    if let Ok(pmcid) = identifier.parse::<Pmcid>() {
        return Ok(format!("PMCID:{pmcid}"));
    }
    if let Ok(doi) = identifier.parse::<Doi>() {
        return Ok(format!("DOI:\"{doi}\""));
    }
    if let Ok(pmid) = identifier.parse::<Pmid>() {
        return Ok(format!("EXT_ID:{pmid} AND SRC:MED"));
    }
    Err(CliError::Usage(format!(
        "{identifier:?} is not a PMCID, a DOI, or a PMID"
    )))
}

/// Adds one work by its identifier.
pub fn add(context: &Context, identifier: &str, now: Timestamp) -> Result<Added> {
    let query = identifier_query(identifier)?;
    let page = europepmc::search(context.http.as_ref(), &query, 1, "*", now)?;
    let found = page
        .found
        .first()
        .ok_or_else(|| CliError::Usage(format!("Europe PMC has no work {identifier}")))?;
    record(context, found, None, now)
}

fn record(context: &Context, found: &Found, query: Option<&str>, now: Timestamp) -> Result<Added> {
    let store = &context.store;
    let existing = [
        found.metadata.pmcid.as_ref().map(ToString::to_string),
        found.metadata.doi.as_ref().map(ToString::to_string),
    ]
    .into_iter()
    .flatten()
    .find_map(|identifier| store.find_work(&identifier).ok().flatten());
    let work = match existing {
        Some(work) if store.latest_text(work)?.is_some() => work,
        _ => store.upsert_work(&found.metadata, now)?,
    };
    let new = store.enqueue(work, query, now)?;
    for notice in &found.notices {
        store.add_notice(work, notice)?;
    }
    Ok(Added {
        work,
        pmcid: found.metadata.pmcid.clone(),
        title: found.metadata.title.clone(),
        new,
    })
}
