use arity_core::{Digest, License};
use arity_sources::pmc;
use arity_store::{QueueState, Retrieval, RetrievalId, RetrievalRole, WorkId};
use arity_text::Attachment;
use jiff::Timestamp;

use crate::context::Context;
use crate::error::{CliError, Result};

const SHEETS_PARSER: &str = concat!("arity-sheets ", env!("CARGO_PKG_VERSION"));

/// What a fetch stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fetched {
    pub work: WorkId,
    pub version: String,
    pub blocks: usize,
    pub pdf: bool,
    pub supplements: usize,
    /// The supplementary data files that arity could not read, with the
    /// reason.
    pub skipped: Vec<String>,
    pub license: Option<License>,
}

/// Downloads a file, stores it under its digest, and records where it came
/// from.
fn download(
    context: &Context,
    work: WorkId,
    role: RetrievalRole,
    supplement_key: Option<&str>,
    url: &str,
    media_type: Option<&str>,
    now: Timestamp,
) -> Result<(RetrievalId, Vec<u8>)> {
    let response = context.http.get_ok(url)?;
    let digest = context.blobs.put(&response.body)?;
    let retrieval = Retrieval {
        role,
        supplement_key: supplement_key.map(str::to_owned),
        url: url.to_owned(),
        media_type: media_type.map(str::to_owned).or(response.content_type),
        digest,
        bytes: u64::try_from(response.body.len()).unwrap_or(u64::MAX),
        retrieved_at: now,
    };
    let id = context.store.add_retrieval(work, &retrieval)?;
    Ok((id, response.body))
}

/// Downloads the newest version of a work from the PMC open-access store:
/// the JATS XML, the PDF, and the supplementary files. Then it makes the
/// canonical text. When the fetch fails, the queue records the reason.
pub fn fetch(context: &Context, work: WorkId, now: Timestamp) -> Result<Fetched> {
    fetch_files(context, work, now).or_else(|error| {
        context.store.enqueue(work, None, now)?;
        context
            .store
            .set_state(work, QueueState::Failed, Some(&error.to_string()), now)?;
        Err(error)
    })
}

/// Downloads the supplementary files, and adds the cells of each data file
/// to the document. Gives the number of files and the files that arity
/// could not read.
fn read_supplements(
    context: &Context,
    work: WorkId,
    entry: &pmc::PmcRecord,
    article: &arity_jats::Article,
    document: &mut arity_text::Document,
    now: Timestamp,
) -> Result<(usize, Vec<String>)> {
    let mut supplements = 0;
    let mut skipped = Vec::new();
    for supplement in &article.supplements {
        let file = supplement
            .href
            .rsplit('/')
            .next()
            .unwrap_or(&supplement.href);
        let Some(url) = entry.media_url(file) else {
            continue;
        };
        let (_, bytes) = download(
            context,
            work,
            RetrievalRole::Supplement,
            Some(&supplement.key),
            &url,
            supplement.media_type.as_deref(),
            now,
        )?;
        supplements += 1;
        let Some(format) = arity_sheets::format_of(file) else {
            continue;
        };
        let blocks = match arity_sheets::read(&bytes, format, &supplement.key) {
            Ok(blocks) => blocks,
            Err(error) => {
                skipped.push(format!("{file}: {error}"));
                continue;
            }
        };
        let attachment = Attachment {
            key: supplement.key.clone(),
            name: file.to_owned(),
            digest: Digest::of(&bytes),
            media_type: supplement.media_type.clone(),
            parser: SHEETS_PARSER.to_owned(),
        };
        document
            .attach(attachment, blocks)
            .map_err(|error| CliError::Usage(error.to_string()))?;
    }
    Ok((supplements, skipped))
}

fn fetch_files(context: &Context, work: WorkId, now: Timestamp) -> Result<Fetched> {
    let record = context.store.work(work)?;
    let pmcid = record.metadata.pmcid.clone().ok_or_else(|| {
        CliError::Usage("the work has no PMCID, so arity cannot find its full text".to_owned())
    })?;
    let entry = pmc::latest_record(context.http.as_ref(), &pmcid)?.ok_or_else(|| {
        CliError::Usage(format!(
            "the PMC open-access store has no files for {pmcid}"
        ))
    })?;
    let xml_url = entry
        .xml_url
        .as_deref()
        .and_then(pmc::https_url)
        .ok_or_else(|| CliError::Usage(format!("the PMC record of {pmcid} names no XML file")))?;
    let (retrieval, xml) = download(
        context,
        work,
        RetrievalRole::FullText,
        None,
        &xml_url,
        Some("application/xml"),
        now,
    )?;
    let article = arity_jats::parse(&xml)?;

    let mut metadata = article.metadata.clone();
    metadata.is_preprint |= record.metadata.is_preprint;
    if metadata.license.is_none() {
        metadata.license = entry.license();
    }
    if metadata.journal.is_none() {
        metadata.journal = record.metadata.journal;
    }
    if metadata.year.is_none() {
        metadata.year = record.metadata.year;
    }
    let work = context.store.upsert_work(&metadata, now)?;
    let mut document = article.document.clone();

    let pdf_url = entry.pdf_url.as_deref().and_then(pmc::https_url);
    if let Some(url) = &pdf_url {
        download(
            context,
            work,
            RetrievalRole::Pdf,
            None,
            url,
            Some("application/pdf"),
            now,
        )?;
    }
    let (supplements, skipped) =
        read_supplements(context, work, &entry, &article, &mut document, now)?;
    context.store.add_text(work, retrieval, &document, now)?;
    for notice in entry.notices(now) {
        context.store.add_notice(work, &notice)?;
    }
    context.store.enqueue(work, None, now)?;
    context
        .store
        .set_state(work, QueueState::Fetched, None, now)?;
    Ok(Fetched {
        work,
        version: entry.version_id(),
        blocks: document.blocks.len(),
        pdf: pdf_url.is_some(),
        supplements,
        skipped,
        license: metadata.license,
    })
}
