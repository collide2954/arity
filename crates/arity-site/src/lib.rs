//! Writes the arity database as static files for the website.
//!
//! The website reads `index.json`, `reports.json`, and one file for each
//! work. `reports.json` holds only the reports of current runs. The reports
//! of runs that a later run replaces go into `replaced.json`, so that links
//! to them continue to work.
//!
//! The export also writes a copy of the database, and a copy of each PDF
//! whose license lets others share it. For a work whose license does not let
//! others share the text, the export keeps the metadata and the evidence
//! quotes, and leaves out the full text and the PDF.

mod display;

use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use arity_core::License;
use arity_protocol::Coverage;
use arity_store::views::{EntityView, PropertyView, ReportView, RunView, WorkView};
use arity_store::{BlobError, Blobs, RetrievalRole, Store, StoreError, TextId, WorkId};
use arity_text::Document;
use jiff::Timestamp;
use serde::Serialize;

pub use display::{DisplayValue, display_value};

pub const SITE_FORMAT: &str = "arity-site/1";
pub const DATABASE_FILE: &str = "arity.sqlite";

#[derive(Debug, thiserror::Error)]
pub enum SiteError {
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    Blob(#[from] BlobError),
    #[error("cannot write {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot write JSON: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, SiteError>;

/// What the export wrote.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Exported {
    pub works: usize,
    pub reports: usize,
    pub pdfs: usize,
}

#[derive(Debug, Serialize)]
struct Index {
    format: &'static str,
    software: &'static str,
    generated_at: String,
    database: &'static str,
    works: Vec<WorkSummary>,
    properties: Vec<PropertySummary>,
    entities: Vec<EntitySummary>,
    /// The entities that only replaced runs name. The reports of the replaced
    /// runs use them, and links to them continue to work.
    former_entities: Vec<EntitySummary>,
}

#[derive(Debug, Serialize)]
struct WorkSummary {
    id: i64,
    title: String,
    citation: String,
    year: Option<i16>,
    journal: Option<String>,
    doi: Option<String>,
    pmcid: Option<String>,
    license: Option<String>,
    shareable: bool,
    is_preprint: bool,
    notices: Vec<String>,
    reports: u64,
    state: Option<String>,
}

#[derive(Debug, Serialize)]
struct PropertySummary {
    #[serde(flatten)]
    property: PropertyView,
    reports: usize,
}

#[derive(Debug, Serialize)]
struct EntitySummary {
    #[serde(flatten)]
    entity: EntityView,
    reports: usize,
}

#[derive(Debug, Serialize)]
struct ReportOut {
    #[serde(flatten)]
    report: ReportView,
    display: Option<DisplayValue>,
}

#[derive(Debug, Serialize)]
struct ReplacedOut {
    #[serde(flatten)]
    report: ReportOut,
    /// The report with the same local id in the current run of the work.
    successor: Option<i64>,
}

#[derive(Debug, Serialize)]
struct Attribution {
    citation: String,
    license: Option<String>,
    license_url: Option<String>,
    statement: Option<String>,
    copyright: Option<String>,
    doi_url: Option<String>,
}

#[derive(Debug, Serialize)]
struct WorkPage<'a> {
    work: &'a WorkView,
    attribution: Attribution,
    document: Option<Document>,
    /// The numbers of the text that the reports quote. The page lists the
    /// unquoted numbers only when it holds the text.
    coverage: Option<Coverage>,
    runs: Vec<&'a RunView>,
    reports: Vec<i64>,
    pdf: Option<String>,
}

fn coverage_of(store: &Store, text: TextId, document: &Document) -> Result<Coverage> {
    let quoted = store.quoted_ranges(text)?;
    let skipped = store.skipped_ranges(text)?;
    Ok(arity_protocol::coverage(document, &quoted, &skipped))
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let text = serde_json::to_vec(value)?;
    fs::write(path, text).map_err(|source| SiteError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn shareable(work: &WorkView) -> bool {
    work.metadata
        .license
        .as_ref()
        .is_some_and(License::allows_redistribution)
}

fn attribution(work: &WorkView) -> Attribution {
    let metadata = &work.metadata;
    Attribution {
        citation: metadata.short_citation(),
        license: metadata.license.as_ref().map(License::label),
        license_url: metadata.license.as_ref().and_then(License::url),
        statement: metadata.license_statement.clone(),
        copyright: metadata.copyright.clone(),
        doi_url: metadata.doi.as_ref().map(arity_core::Doi::url),
    }
}

fn summary(work: &WorkView) -> WorkSummary {
    let notices: BTreeSet<String> = work
        .notices
        .iter()
        .map(|notice| notice.kind.as_str().to_owned())
        .collect();
    WorkSummary {
        id: work.id,
        title: work.metadata.title.clone(),
        citation: work.metadata.short_citation(),
        year: work.metadata.year,
        journal: work.metadata.journal.clone(),
        doi: work.metadata.doi.as_ref().map(ToString::to_string),
        pmcid: work.metadata.pmcid.as_ref().map(ToString::to_string),
        license: work.metadata.license.as_ref().map(License::label),
        shareable: shareable(work),
        is_preprint: work.metadata.is_preprint,
        notices: notices.into_iter().collect(),
        reports: work.reports,
        state: work.state.clone(),
    }
}

/// Writes the static files of the website into a folder.
pub fn export(store: &Store, blobs: &Blobs, out: &Path, now: Timestamp) -> Result<Exported> {
    let works_folder = out.join("works");
    fs::create_dir_all(&works_folder).map_err(|source| SiteError::Io {
        path: works_folder.clone(),
        source,
    })?;
    let works = store.work_views()?;
    let properties = store.property_views()?;
    let runs = store.run_views()?;
    let (reports, replaced): (Vec<ReportOut>, Vec<ReportOut>) =
        reports_with_display(store, &properties)?
            .into_iter()
            .partition(|report| report.report.replaced_by.is_none());
    write_json(&out.join("reports.json"), &reports)?;
    write_json(
        &out.join("replaced.json"),
        &with_successors(replaced, &reports, &runs),
    )?;
    let pdfs = write_work_pages(store, blobs, out, &works, &runs, &reports)?;
    write_index(store, out, &works, properties, &reports, now)?;
    write_database(store, out, &works)?;
    Ok(Exported {
        works: works.len(),
        reports: reports.len(),
        pdfs,
    })
}

fn reports_with_display(store: &Store, properties: &[PropertyView]) -> Result<Vec<ReportOut>> {
    let units: HashMap<&str, &str> = properties
        .iter()
        .filter_map(|property| Some((property.key.as_str(), property.unit.as_deref()?)))
        .collect();
    Ok(store
        .report_views()?
        .into_iter()
        .map(|report| {
            let display = (report.kind == "value")
                .then(|| {
                    let unit = units.get(report.property.as_deref()?)?;
                    display_value(report.body.get("value")?, unit)
                })
                .flatten();
            ReportOut { report, display }
        })
        .collect())
}

/// Gives each replaced report the report with the same local id in the
/// current run of its work. A run can replace only an earlier run, so each
/// chain of replaced runs comes to an end.
fn with_successors(
    replaced: Vec<ReportOut>,
    current: &[ReportOut],
    runs: &[RunView],
) -> Vec<ReplacedOut> {
    let newer: HashMap<i64, i64> = runs
        .iter()
        .filter_map(|run| Some((run.id, run.replaced_by?)))
        .collect();
    let current: HashMap<(i64, &str), i64> = current
        .iter()
        .map(|out| {
            (
                (out.report.run, out.report.local_id.as_str()),
                out.report.id,
            )
        })
        .collect();
    replaced
        .into_iter()
        .map(|report| {
            let mut run = report.report.run;
            while let Some(next) = newer.get(&run) {
                run = *next;
            }
            let successor = current
                .get(&(run, report.report.local_id.as_str()))
                .copied();
            ReplacedOut { report, successor }
        })
        .collect()
}

fn copy_pdf(store: &Store, blobs: &Blobs, out: &Path, work: &WorkView) -> Result<Option<String>> {
    let latest = store
        .retrievals(WorkId(work.id))?
        .into_iter()
        .rev()
        .find(|(_, retrieval)| retrieval.role == RetrievalRole::Pdf);
    let Some((_, retrieval)) = latest else {
        return Ok(None);
    };
    if !shareable(work) || !blobs.contains(&retrieval.digest) {
        return Ok(None);
    }
    let name = format!("works/{}.pdf", work.id);
    let path = out.join(&name);
    fs::write(&path, blobs.get(&retrieval.digest)?)
        .map_err(|source| SiteError::Io { path, source })?;
    Ok(Some(name))
}

fn write_work_pages(
    store: &Store,
    blobs: &Blobs,
    out: &Path,
    works: &[WorkView],
    runs: &[RunView],
    reports: &[ReportOut],
) -> Result<usize> {
    let mut pdfs = 0;
    for work in works {
        let latest = store.latest_text(WorkId(work.id))?;
        let mut coverage = latest
            .as_ref()
            .map(|(text, document)| coverage_of(store, *text, document))
            .transpose()?;
        let document = if shareable(work) {
            latest.map(|(_, document)| document)
        } else {
            if let Some(coverage) = coverage.as_mut() {
                coverage.unquoted.clear();
            }
            None
        };
        let pdf = copy_pdf(store, blobs, out, work)?;
        pdfs += usize::from(pdf.is_some());
        let page = WorkPage {
            work,
            attribution: attribution(work),
            document,
            coverage,
            runs: runs.iter().filter(|run| run.work == work.id).collect(),
            reports: reports
                .iter()
                .filter(|report| report.report.work == work.id)
                .map(|report| report.report.id)
                .collect(),
            pdf,
        };
        write_json(&out.join("works").join(format!("{}.json", work.id)), &page)?;
    }
    Ok(pdfs)
}

fn write_index(
    store: &Store,
    out: &Path,
    works: &[WorkView],
    properties: Vec<PropertyView>,
    reports: &[ReportOut],
    now: Timestamp,
) -> Result<()> {
    let property_counts = count(
        reports
            .iter()
            .filter_map(|report| report.report.property.clone()),
    );
    let entity_counts = count(reports.iter().flat_map(|report| {
        report
            .report
            .entities
            .values()
            .copied()
            .collect::<BTreeSet<i64>>()
    }));
    let (entities, former_entities) = store
        .entity_views()?
        .into_iter()
        .map(|entity| EntitySummary {
            reports: entity_counts.get(&entity.id).copied().unwrap_or(0),
            entity,
        })
        .partition(|summary| summary.entity.current);
    let index = Index {
        format: SITE_FORMAT,
        software: arity_store::SOFTWARE,
        generated_at: now.to_string(),
        database: DATABASE_FILE,
        works: works.iter().map(summary).collect(),
        properties: properties
            .into_iter()
            .map(|property| PropertySummary {
                reports: property_counts.get(&property.key).copied().unwrap_or(0),
                property,
            })
            .collect(),
        entities,
        former_entities,
    };
    write_json(&out.join("index.json"), &index)
}

fn write_database(store: &Store, out: &Path, works: &[WorkView]) -> Result<()> {
    let database = out.join(DATABASE_FILE);
    if database.exists() {
        fs::remove_file(&database).map_err(|source| SiteError::Io {
            path: database.clone(),
            source,
        })?;
    }
    store.copy_to(&database)?;
    let closed: Vec<WorkId> = works
        .iter()
        .filter(|work| !shareable(work))
        .map(|work| WorkId(work.id))
        .collect();
    if !closed.is_empty() {
        Store::open(&database)?.remove_texts(&closed)?;
    }
    Ok(())
}

fn count<K: std::hash::Hash + Eq>(keys: impl Iterator<Item = K>) -> HashMap<K, usize> {
    let mut counts = HashMap::new();
    for key in keys {
        *counts.entry(key).or_insert(0) += 1;
    }
    counts
}
