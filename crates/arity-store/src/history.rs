use arity_core::vocabulary::CheckState;
use arity_core::{Digest, Orcid};
use arity_protocol::Person;
use jiff::Timestamp;
use rusqlite::{OptionalExtension, params};

use crate::store::{QueueState, Result, RunId, Store, StoreError, WorkId, timestamp};

/// The entry of a work in the reading queue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueEntry {
    pub state: QueueState,
    /// The query that found the work.
    pub query: Option<String>,
    /// The reason for the state, for example why a fetch failed.
    pub note: Option<String>,
    pub added_at: Timestamp,
    pub updated_at: Timestamp,
}

/// A canonical text without its blocks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextRecord {
    pub digest: Digest,
    pub parser: String,
    /// The digest of the file that arity made the text from.
    pub source: Digest,
}

/// A submission as the database holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRecord {
    pub id: RunId,
    pub work: WorkId,
    pub digest: Digest,
    /// The submission file, byte for byte.
    pub submission: String,
    /// The software that checked the submission.
    pub software: String,
    pub submitted_at: Timestamp,
}

/// The check of one report by one person.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckEvent {
    pub run: RunId,
    /// The identifier of the report in its submission.
    pub report: String,
    pub state: CheckState,
    pub person: Person,
    pub note: Option<String>,
    pub at: Timestamp,
}

fn digest(text: &str) -> Result<Digest> {
    text.parse()
        .map_err(|_| StoreError::Corrupt(format!("digest {text:?}")))
}

impl Store {
    pub fn queue_entry(&self, work: WorkId) -> Result<Option<QueueEntry>> {
        let row = self
            .connection
            .query_row(
                "SELECT state, query, note, added_at, updated_at FROM queue WHERE work_id = ?1",
                [work.0],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                    ))
                },
            )
            .optional()?;
        row.map(|(state, query, note, added_at, updated_at)| {
            Ok(QueueEntry {
                state: QueueState::parse(&state)?,
                query,
                note,
                added_at: timestamp(&added_at)?,
                updated_at: timestamp(&updated_at)?,
            })
        })
        .transpose()
    }

    /// Writes the queue entry of a work. It replaces the entry that the work
    /// has.
    pub fn restore_queue(&self, work: WorkId, entry: &QueueEntry) -> Result<()> {
        self.connection.execute(
            "INSERT OR REPLACE INTO queue (work_id, state, query, note, added_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                work.0,
                entry.state.as_str(),
                entry.query,
                entry.note,
                entry.added_at.to_string(),
                entry.updated_at.to_string()
            ],
        )?;
        Ok(())
    }

    /// Sets the times at which arity added the record of a work and last
    /// changed it.
    pub fn restore_times(
        &self,
        work: WorkId,
        added_at: Timestamp,
        updated_at: Timestamp,
    ) -> Result<()> {
        let changed = self.connection.execute(
            "UPDATE work SET added_at = ?2, updated_at = ?3 WHERE id = ?1",
            params![work.0, added_at.to_string(), updated_at.to_string()],
        )?;
        if changed == 0 {
            return Err(StoreError::NotFound(format!(
                "no work has the id {}",
                work.0
            )));
        }
        Ok(())
    }

    /// The canonical texts of a work, in the order that arity made them.
    pub fn texts(&self, work: WorkId) -> Result<Vec<TextRecord>> {
        let mut statement = self.connection.prepare(
            "SELECT text.sha256, text.parser, retrieval.sha256 FROM text
             JOIN retrieval ON retrieval.id = text.retrieval_id
             WHERE text.work_id = ?1 ORDER BY text.id",
        )?;
        let rows = statement
            .query_map([work.0], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        rows.into_iter()
            .map(|(text, parser, source)| {
                Ok(TextRecord {
                    digest: digest(&text)?,
                    parser,
                    source: digest(&source)?,
                })
            })
            .collect()
    }

    /// All runs, in the order that the database stored them.
    pub fn runs(&self) -> Result<Vec<RunRecord>> {
        let mut statement = self.connection.prepare(
            "SELECT id, work_id, submission_sha256, submission, software, submitted_at
             FROM run ORDER BY id",
        )?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        rows.into_iter()
            .map(|(id, work, sha256, submission, software, submitted_at)| {
                Ok(RunRecord {
                    id: RunId(id),
                    work: WorkId(work),
                    digest: digest(&sha256)?,
                    submission,
                    software,
                    submitted_at: timestamp(&submitted_at)?,
                })
            })
            .collect()
    }

    /// All checks of reports, in the order that the database stored them.
    pub fn check_events(&self) -> Result<Vec<CheckEvent>> {
        let mut statement = self.connection.prepare(
            "SELECT report.run_id, report.local_id, check_event.state, agent.name, agent.orcid,
             check_event.note, check_event.at
             FROM check_event
             JOIN report ON report.id = check_event.report_id
             JOIN agent ON agent.id = check_event.person_id
             ORDER BY check_event.id",
        )?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, String>(6)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        rows.into_iter()
            .map(|(run, report, state, name, orcid, note, at)| {
                Ok(CheckEvent {
                    run: RunId(run),
                    report,
                    state: state
                        .parse()
                        .map_err(|_| StoreError::Corrupt(format!("check state {state:?}")))?,
                    person: Person {
                        name,
                        orcid: orcid
                            .map(|text| {
                                text.parse::<Orcid>()
                                    .map_err(|_| StoreError::Corrupt(format!("ORCID {text:?}")))
                            })
                            .transpose()?,
                    },
                    note,
                    at: timestamp(&at)?,
                })
            })
            .collect()
    }
}
