//! The archive of a database: the files from which arity can make the same
//! database again. The archive holds no file of a source. For each file it
//! holds the address and the SHA-256 digest, so that a rebuild can download
//! the file again and make sure that it is the same file.
//!
//! The archive has one folder for each work, `works/<identifier>/`:
//!
//! - `work.json` holds the record of the work, its queue entry, the files
//!   that arity downloaded, the digests of its canonical texts, its notices,
//!   and its runs with the checks of their reports.
//! - `run-<sequence>.json` holds each submission, byte for byte.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use arity_core::vocabulary::CheckState;
use arity_core::{Digest, Notice, WorkMetadata};
use arity_protocol::{Issue, Person};
use arity_store::{
    CheckEvent, QueueEntry, QueueState, Retrieval, RetrievalId, RetrievalRole, RunRecord,
    TextRecord, WorkId, WorkRecord,
};
use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::context::Context;
use crate::error::{CliError, Result, io};
use crate::fetch::attach_sheet;
use crate::submit::{Outcome, submit_bytes};

/// The format of the work files of an archive.
pub const FORMAT: &str = "arity-archive/1";
const WORKS: &str = "works";
const WORK_FILE: &str = "work.json";

/// The record of one work in the archive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkFile {
    pub format: String,
    /// The place of the work in the catalogue. A rebuild adds the works in
    /// this order.
    pub position: i64,
    pub metadata: WorkMetadata,
    pub added_at: Timestamp,
    pub updated_at: Timestamp,
    pub queue: Option<QueueFile>,
    pub retrievals: Vec<RetrievalFile>,
    pub texts: Vec<TextFile>,
    pub notices: Vec<Notice>,
    pub runs: Vec<RunFile>,
}

/// The entry of a work in the reading queue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueueFile {
    pub state: QueueState,
    pub query: Option<String>,
    pub note: Option<String>,
    pub added_at: Timestamp,
    pub updated_at: Timestamp,
}

/// One downloaded file: where it came from, when, and its digest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetrievalFile {
    pub role: RetrievalRole,
    pub supplement_key: Option<String>,
    pub url: String,
    pub media_type: Option<String>,
    pub sha256: Digest,
    pub bytes: u64,
    pub software: String,
    pub retrieved_at: Timestamp,
}

/// A canonical text: its digest, its parser, and the digest of the file that
/// arity made it from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextFile {
    pub sha256: Digest,
    pub parser: String,
    pub source: Digest,
}

/// One submission and the checks of its reports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunFile {
    /// The place of the run among all runs, and the number of the run in the
    /// database. A rebuild stores the runs in this order. The file
    /// `run-<sequence>.json` holds the submission.
    pub sequence: i64,
    pub sha256: Digest,
    /// The software that checked the submission.
    pub software: String,
    pub submitted_at: Timestamp,
    pub checks: Vec<CheckFile>,
}

/// The check of one report by one person.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckFile {
    /// The identifier of the report in its submission.
    pub report: String,
    pub state: CheckState,
    pub person: Person,
    pub note: Option<String>,
    pub at: Timestamp,
}

impl From<QueueEntry> for QueueFile {
    fn from(entry: QueueEntry) -> Self {
        Self {
            state: entry.state,
            query: entry.query,
            note: entry.note,
            added_at: entry.added_at,
            updated_at: entry.updated_at,
        }
    }
}

impl From<&QueueFile> for QueueEntry {
    fn from(file: &QueueFile) -> Self {
        Self {
            state: file.state,
            query: file.query.clone(),
            note: file.note.clone(),
            added_at: file.added_at,
            updated_at: file.updated_at,
        }
    }
}

impl From<Retrieval> for RetrievalFile {
    fn from(retrieval: Retrieval) -> Self {
        Self {
            role: retrieval.role,
            supplement_key: retrieval.supplement_key,
            url: retrieval.url,
            media_type: retrieval.media_type,
            sha256: retrieval.digest,
            bytes: retrieval.bytes,
            software: retrieval.software,
            retrieved_at: retrieval.retrieved_at,
        }
    }
}

impl From<&RetrievalFile> for Retrieval {
    fn from(file: &RetrievalFile) -> Self {
        Self {
            role: file.role,
            supplement_key: file.supplement_key.clone(),
            url: file.url.clone(),
            media_type: file.media_type.clone(),
            digest: file.sha256.clone(),
            bytes: file.bytes,
            software: file.software.clone(),
            retrieved_at: file.retrieved_at,
        }
    }
}

impl From<TextRecord> for TextFile {
    fn from(text: TextRecord) -> Self {
        Self {
            sha256: text.digest,
            parser: text.parser,
            source: text.source,
        }
    }
}

impl From<&CheckEvent> for CheckFile {
    fn from(event: &CheckEvent) -> Self {
        Self {
            report: event.report.clone(),
            state: event.state,
            person: event.person.clone(),
            note: event.note.clone(),
            at: event.at,
        }
    }
}

/// What [`write()`] wrote.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Written {
    pub works: usize,
    pub runs: usize,
}

/// What [`rebuild`] added to the database.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rebuilt {
    pub works: usize,
    pub files: usize,
    pub runs: usize,
    pub checks: usize,
}

/// The name of the folder of a work: its PMCID, DOI, or PMID.
fn folder_name(work: &WorkRecord) -> String {
    let metadata = &work.metadata;
    if let Some(pmcid) = &metadata.pmcid {
        return pmcid.to_string();
    }
    if let Some(doi) = &metadata.doi {
        let safe: String = doi
            .as_str()
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '.' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        return format!("doi-{safe}");
    }
    if let Some(pmid) = &metadata.pmid {
        return format!("pmid-{pmid}");
    }
    format!("work-{}", work.id.0)
}

fn run_file(sequence: i64) -> String {
    format!("run-{sequence}.json")
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let mut bytes =
        serde_json::to_vec_pretty(value).map_err(|error| CliError::Usage(error.to_string()))?;
    bytes.push(b'\n');
    fs::write(path, bytes).map_err(io(path))
}

/// Writes the file of a run. A run does not change, so a file with other
/// bytes at the same place is an error.
fn write_run(path: &Path, submission: &[u8]) -> Result<()> {
    match fs::read(path) {
        Ok(existing) if existing == submission => Ok(()),
        Ok(_) => Err(CliError::Usage(format!(
            "the archive has another file at {}, but a run does not change",
            path.display()
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::write(path, submission).map_err(io(path))
        }
        Err(error) => Err(io(path)(error)),
    }
}

/// Writes the archive of the database into a folder.
pub fn write(context: &Context, folder: &Path) -> Result<Written> {
    let runs = context.store.runs()?;
    let checks = context.store.check_events()?;
    let works = context.store.works()?;
    let mut names = HashSet::new();
    for work in &works {
        let name = folder_name(work);
        if !names.insert(name.clone()) {
            return Err(CliError::Usage(format!(
                "two works have the folder name {name}"
            )));
        }
        let directory = folder.join(WORKS).join(&name);
        fs::create_dir_all(&directory).map_err(io(&directory))?;
        let own: Vec<&RunRecord> = runs.iter().filter(|run| run.work == work.id).collect();
        for run in &own {
            write_run(
                &directory.join(run_file(run.id.0)),
                run.submission.as_bytes(),
            )?;
        }
        let file = WorkFile {
            format: FORMAT.to_owned(),
            position: work.id.0,
            metadata: work.metadata.clone(),
            added_at: work.added_at,
            updated_at: work.updated_at,
            queue: context.store.queue_entry(work.id)?.map(QueueFile::from),
            retrievals: context
                .store
                .retrievals(work.id)?
                .into_iter()
                .map(|(_, retrieval)| RetrievalFile::from(retrieval))
                .collect(),
            texts: context
                .store
                .texts(work.id)?
                .into_iter()
                .map(TextFile::from)
                .collect(),
            notices: context.store.notices(work.id)?,
            runs: own
                .iter()
                .map(|run| RunFile {
                    sequence: run.id.0,
                    sha256: run.digest.clone(),
                    software: run.software.clone(),
                    submitted_at: run.submitted_at,
                    checks: checks
                        .iter()
                        .filter(|check| check.run == run.id)
                        .map(CheckFile::from)
                        .collect(),
                })
                .collect(),
        };
        write_json(&directory.join(WORK_FILE), &file)?;
    }
    Ok(Written {
        works: works.len(),
        runs: runs.len(),
    })
}

/// The work files of an archive, in the order of their positions.
fn read_works(folder: &Path) -> Result<Vec<(PathBuf, WorkFile)>> {
    let root = folder.join(WORKS);
    let mut works = Vec::new();
    for entry in fs::read_dir(&root).map_err(io(&root))? {
        let directory = entry.map_err(io(&root))?.path();
        if !directory.is_dir() {
            continue;
        }
        let path = directory.join(WORK_FILE);
        let bytes = fs::read(&path).map_err(io(&path))?;
        let file: WorkFile = serde_json::from_slice(&bytes).map_err(|error| {
            CliError::Usage(format!(
                "{} is not a work file of an archive: {error}",
                path.display()
            ))
        })?;
        if file.format != FORMAT {
            return Err(CliError::Usage(format!(
                "{} has the format {}, but this arity reads {FORMAT}",
                path.display(),
                file.format
            )));
        }
        works.push((directory, file));
    }
    works.sort_by_key(|(_, file)| file.position);
    if let Some(pair) = works
        .windows(2)
        .find(|pair| pair[0].1.position == pair[1].1.position)
    {
        return Err(CliError::Usage(format!(
            "{} and {} have the same position",
            pair[0].0.display(),
            pair[1].0.display()
        )));
    }
    Ok(works)
}

/// Downloads the files of a work again, and makes sure that each file is
/// the archived file. Then it makes the canonical texts again, and makes
/// sure that each text has the archived digest. Gives the number of files.
fn restore_files(context: &Context, work: WorkId, file: &WorkFile) -> Result<usize> {
    let mut restored: Vec<(RetrievalId, &RetrievalFile)> = Vec::new();
    for archived in &file.retrievals {
        let response = context.http.get_ok(&archived.url)?;
        let digest = Digest::of(&response.body);
        if digest != archived.sha256 {
            return Err(CliError::Usage(format!(
                "the file at {} is not the archived file: its SHA-256 digest is {digest}, not {}",
                archived.url, archived.sha256
            )));
        }
        context.blobs.put(&response.body)?;
        let id = context
            .store
            .add_retrieval(work, &Retrieval::from(archived))?;
        restored.push((id, archived));
    }
    for text in &file.texts {
        let (source, archived) = restored
            .iter()
            .find(|(_, retrieval)| {
                retrieval.role == RetrievalRole::FullText && retrieval.sha256 == text.source
            })
            .copied()
            .ok_or_else(|| {
                CliError::Usage(format!(
                    "the archive has no full text with the digest {}",
                    text.source
                ))
            })?;
        let article = arity_jats::parse(&context.blobs.get(&archived.sha256)?)?;
        let mut document = article.document;
        for supplement in &article.supplements {
            let data = restored.iter().find(|(_, retrieval)| {
                retrieval.role == RetrievalRole::Supplement
                    && retrieval.supplement_key.as_deref() == Some(supplement.key.as_str())
                    && retrieval.retrieved_at == archived.retrieved_at
            });
            if let Some((_, data)) = data {
                attach_sheet(&mut document, supplement, &context.blobs.get(&data.sha256)?)?;
            }
        }
        let digest = document
            .digest()
            .map_err(|error| CliError::Usage(error.to_string()))?;
        if digest != text.sha256 {
            return Err(CliError::Usage(format!(
                "the text that arity makes from {} has the digest {digest}, not {}; the archive names the parser {}, and this arity has {}",
                archived.url, text.sha256, text.parser, document.parser
            )));
        }
        context
            .store
            .add_text(work, source, &document, archived.retrieved_at)?;
    }
    Ok(restored.len())
}

fn not_stored(path: &Path, issues: &[Issue]) -> CliError {
    let issues: Vec<String> = issues.iter().map(ToString::to_string).collect();
    CliError::Usage(format!(
        "{} does not pass the checks of this arity:\n{}",
        path.display(),
        issues.join("\n")
    ))
}

/// Stores an archived run again, and records the checks of its reports.
/// Gives the number of checks.
fn replay(context: &Context, directory: &Path, run: &RunFile) -> Result<usize> {
    let path = directory.join(run_file(run.sequence));
    let raw = fs::read(&path).map_err(io(&path))?;
    let digest = Digest::of(&raw);
    if digest != run.sha256 {
        return Err(CliError::Usage(format!(
            "{} is not the archived submission: its SHA-256 digest is {digest}, not {}",
            path.display(),
            run.sha256
        )));
    }
    let (stored, reports) = match submit_bytes(context, &raw, false, run.submitted_at)? {
        Outcome::Stored { run, reports, .. } => (run, reports),
        Outcome::Rejected { issues } => return Err(not_stored(&path, &issues)),
        Outcome::Passed { .. } => return Err(not_stored(&path, &[])),
    };
    if stored.0 != run.sequence {
        return Err(CliError::Usage(format!(
            "{} is run {} of the archive, but the database stored it as run {}. A correction \
             names the run that it replaces by its number, so the numbers must go up from 1 \
             with no gap",
            path.display(),
            run.sequence,
            stored.0
        )));
    }
    for check in &run.checks {
        let report = reports
            .iter()
            .find(|(_, local)| *local == check.report)
            .map(|(id, _)| *id)
            .ok_or_else(|| {
                CliError::Usage(format!("{} has no report {}", path.display(), check.report))
            })?;
        context.store.record_check(
            report,
            check.state,
            &check.person,
            check.note.as_deref(),
            check.at,
        )?;
    }
    Ok(run.checks.len())
}

/// Makes the database again from an archive. It downloads each file again
/// and makes sure that the file did not change, makes the canonical texts
/// again, and stores the runs and the checks in their first order. The
/// database must have no works.
pub fn rebuild(context: &Context, folder: &Path) -> Result<Rebuilt> {
    if !context.store.works()?.is_empty() {
        return Err(CliError::Usage(
            "the database has works; rebuild into an empty data folder".to_owned(),
        ));
    }
    let works = read_works(folder)?;
    let mut rebuilt = Rebuilt::default();
    let mut ids = Vec::with_capacity(works.len());
    for (_, file) in &works {
        let work = context.store.upsert_work(&file.metadata, file.added_at)?;
        rebuilt.files += restore_files(context, work, file)?;
        for notice in &file.notices {
            context.store.add_notice(work, notice)?;
        }
        ids.push(work);
        rebuilt.works += 1;
    }
    let mut runs: Vec<(&Path, &RunFile)> = works
        .iter()
        .flat_map(|(directory, file)| file.runs.iter().map(move |run| (directory.as_path(), run)))
        .collect();
    runs.sort_by_key(|(_, run)| run.sequence);
    for (directory, run) in runs {
        rebuilt.checks += replay(context, directory, run)?;
        rebuilt.runs += 1;
    }
    for (work, (_, file)) in ids.into_iter().zip(&works) {
        if let Some(queue) = &file.queue {
            context
                .store
                .restore_queue(work, &QueueEntry::from(queue))?;
        }
        context
            .store
            .restore_times(work, file.added_at, file.updated_at)?;
    }
    Ok(rebuilt)
}
