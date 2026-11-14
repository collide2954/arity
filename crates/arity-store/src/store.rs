use std::path::Path;

use arity_core::{Digest, Doi, Notice, Pmcid, Pmid, WorkMetadata};
use jiff::Timestamp;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

/// The name and version that each stored record names as its software.
pub const SOFTWARE: &str = concat!("arity ", env!("CARGO_PKG_VERSION"));

const MIGRATIONS: &[&str] = &[
    include_str!("../migrations/001_initial.sql"),
    include_str!("../migrations/002_run_entity.sql"),
    include_str!("../migrations/003_skip.sql"),
];

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(pub i64);
    };
}

id_type!(WorkId);
id_type!(RetrievalId);
id_type!(TextId);
id_type!(RunId);
id_type!(EntityId);
id_type!(ReportId);
id_type!(AgentId);

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("cannot read stored JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("the database holds a value that is not valid: {0}")]
    Corrupt(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Conflict(String),
}

pub type Result<T> = std::result::Result<T, StoreError>;

/// The state of a work in the reading queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueueState {
    Queued,
    Fetched,
    Read,
    Failed,
    Skipped,
}

impl QueueState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Fetched => "fetched",
            Self::Read => "read",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }

    pub(crate) fn parse(text: &str) -> Result<Self> {
        [
            Self::Queued,
            Self::Fetched,
            Self::Read,
            Self::Failed,
            Self::Skipped,
        ]
        .into_iter()
        .find(|state| state.as_str() == text)
        .ok_or_else(|| StoreError::Corrupt(format!("queue state {text:?}")))
    }
}

/// What a downloaded file is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalRole {
    Metadata,
    FullText,
    Pdf,
    Supplement,
}

impl RetrievalRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Metadata => "metadata",
            Self::FullText => "full_text",
            Self::Pdf => "pdf",
            Self::Supplement => "supplement",
        }
    }

    fn parse(text: &str) -> Result<Self> {
        [Self::Metadata, Self::FullText, Self::Pdf, Self::Supplement]
            .into_iter()
            .find(|role| role.as_str() == text)
            .ok_or_else(|| StoreError::Corrupt(format!("retrieval role {text:?}")))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkRecord {
    pub id: WorkId,
    pub metadata: WorkMetadata,
    pub state: Option<QueueState>,
    pub added_at: Timestamp,
    pub updated_at: Timestamp,
}

/// A record of one download: where the file came from, when, and its digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Retrieval {
    pub role: RetrievalRole,
    pub supplement_key: Option<String>,
    pub url: String,
    pub media_type: Option<String>,
    pub digest: Digest,
    pub bytes: u64,
    /// The software that downloaded the file.
    pub software: String,
    pub retrieved_at: Timestamp,
}

#[derive(Debug)]
pub struct Store {
    pub(crate) connection: Connection,
}

fn migrate(connection: &mut Connection) -> Result<()> {
    let version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    let applied = usize::try_from(version)
        .map_err(|_| StoreError::Corrupt(format!("schema version {version}")))?;
    if applied > MIGRATIONS.len() {
        return Err(StoreError::Conflict(format!(
            "the database has schema version {version}, but this arity knows only {}",
            MIGRATIONS.len()
        )));
    }
    for (index, migration) in (1_i64..).zip(MIGRATIONS).skip(applied) {
        let transaction = connection.transaction()?;
        transaction.execute_batch(migration)?;
        transaction.pragma_update(None, "user_version", index)?;
        transaction.commit()?;
    }
    Ok(())
}

pub(crate) fn timestamp(text: &str) -> Result<Timestamp> {
    text.parse()
        .map_err(|_| StoreError::Corrupt(format!("timestamp {text:?}")))
}

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        Self::with_connection(Connection::open(path)?)
    }

    pub fn open_in_memory() -> Result<Self> {
        Self::with_connection(Connection::open_in_memory()?)
    }

    fn with_connection(mut connection: Connection) -> Result<Self> {
        connection.pragma_update(None, "foreign_keys", true)?;
        connection.pragma_update_and_check(None, "journal_mode", "WAL", |_| Ok(()))?;
        migrate(&mut connection)?;
        Ok(Self { connection })
    }

    /// Removes the full text of works from this database. The export calls
    /// it on its copy for each work whose license does not let others share
    /// the text. The evidence quotes stay.
    pub fn remove_texts(&self, works: &[WorkId]) -> Result<()> {
        let transaction = self.connection.unchecked_transaction()?;
        for work in works {
            transaction.execute(
                "DELETE FROM block WHERE text_id IN (SELECT id FROM text WHERE work_id = ?1)",
                [work.0],
            )?;
            transaction.execute("UPDATE text SET document = '' WHERE work_id = ?1", [work.0])?;
        }
        transaction.commit()?;
        Ok(())
    }

    /// Writes a clean copy of the whole database to one file.
    pub fn copy_to(&self, path: &Path) -> Result<()> {
        let target = path.to_str().ok_or_else(|| {
            StoreError::Conflict(format!("the path {} is not UTF-8", path.display()))
        })?;
        self.connection.execute("VACUUM INTO ?1", [target])?;
        Ok(())
    }

    pub fn schema_version(&self) -> Result<i64> {
        Ok(self
            .connection
            .pragma_query_value(None, "user_version", |row| row.get(0))?)
    }

    fn find_by(&self, column: &str, value: &str) -> Result<Option<WorkId>> {
        let sql = format!("SELECT id FROM work WHERE {column} = ?1");
        Ok(self
            .connection
            .query_row(&sql, [value], |row| row.get(0))
            .optional()?
            .map(WorkId))
    }

    /// Finds a work by a PMCID, a DOI, or a PMID.
    pub fn find_work(&self, identifier: &str) -> Result<Option<WorkId>> {
        if let Ok(pmcid) = identifier.parse::<Pmcid>() {
            return self.find_by("pmcid", pmcid.as_str());
        }
        if let Ok(doi) = identifier.parse::<Doi>() {
            return self.find_by("doi", doi.as_str());
        }
        if let Ok(pmid) = identifier.parse::<Pmid>() {
            return self.find_by("pmid", pmid.as_str());
        }
        Ok(None)
    }

    fn find_matching(&self, metadata: &WorkMetadata) -> Result<Option<WorkId>> {
        if let Some(pmcid) = &metadata.pmcid
            && let Some(id) = self.find_by("pmcid", pmcid.as_str())?
        {
            return Ok(Some(id));
        }
        if let Some(doi) = &metadata.doi
            && let Some(id) = self.find_by("doi", doi.as_str())?
        {
            return Ok(Some(id));
        }
        if let Some(pmid) = &metadata.pmid {
            return self.find_by("pmid", pmid.as_str());
        }
        Ok(None)
    }

    /// Adds a work, or replaces the record of the same work. An identifier
    /// that the new record does not have stays in the database.
    pub fn upsert_work(&self, metadata: &WorkMetadata, now: Timestamp) -> Result<WorkId> {
        let existing = self.find_matching(metadata)?;
        let mut merged = metadata.clone();
        if let Some(id) = existing {
            let old = self.work(id)?.metadata;
            merged.doi = merged.doi.or(old.doi);
            merged.pmid = merged.pmid.or(old.pmid);
            merged.pmcid = merged.pmcid.or(old.pmcid);
        }
        let json = serde_json::to_string(&merged)?;
        let doi = merged.doi.as_ref().map(Doi::as_str);
        let pubmed_id = merged.pmid.as_ref().map(Pmid::as_str);
        let central_id = merged.pmcid.as_ref().map(Pmcid::as_str);
        let license = merged.license.as_ref().map(ToString::to_string);
        let now = now.to_string();
        if let Some(id) = existing {
            self.connection.execute(
                "UPDATE work SET doi = ?1, pmid = ?2, pmcid = ?3, title = ?4, year = ?5,
                 license = ?6, is_preprint = ?7, metadata = ?8, updated_at = ?9 WHERE id = ?10",
                params![
                    doi,
                    pubmed_id,
                    central_id,
                    merged.title,
                    merged.year,
                    license,
                    merged.is_preprint,
                    json,
                    now,
                    id.0
                ],
            )?;
            return Ok(id);
        }
        self.connection.execute(
            "INSERT INTO work (doi, pmid, pmcid, title, year, license, is_preprint, metadata,
             added_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
            params![
                doi,
                pubmed_id,
                central_id,
                merged.title,
                merged.year,
                license,
                merged.is_preprint,
                json,
                now
            ],
        )?;
        Ok(WorkId(self.connection.last_insert_rowid()))
    }

    pub fn work(&self, id: WorkId) -> Result<WorkRecord> {
        let row = self
            .connection
            .query_row(
                "SELECT work.metadata, work.added_at, work.updated_at, queue.state
                 FROM work LEFT JOIN queue ON queue.work_id = work.id WHERE work.id = ?1",
                [id.0],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<String>>(3)?,
                    ))
                },
            )
            .optional()?
            .ok_or_else(|| StoreError::NotFound(format!("no work has the id {}", id.0)))?;
        Ok(WorkRecord {
            id,
            metadata: serde_json::from_str(&row.0)?,
            added_at: timestamp(&row.1)?,
            updated_at: timestamp(&row.2)?,
            state: row.3.as_deref().map(QueueState::parse).transpose()?,
        })
    }

    pub fn works(&self) -> Result<Vec<WorkRecord>> {
        let mut statement = self.connection.prepare("SELECT id FROM work ORDER BY id")?;
        let ids: Vec<i64> = statement
            .query_map([], |row| row.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        ids.into_iter().map(|id| self.work(WorkId(id))).collect()
    }

    /// Adds a work to the reading queue. A work that is already in the
    /// queue keeps its state.
    pub fn enqueue(&self, work: WorkId, query: Option<&str>, now: Timestamp) -> Result<bool> {
        let added = self.connection.execute(
            "INSERT OR IGNORE INTO queue (work_id, state, query, added_at, updated_at)
             VALUES (?1, 'queued', ?2, ?3, ?3)",
            params![work.0, query, now.to_string()],
        )?;
        Ok(added == 1)
    }

    pub fn set_state(
        &self,
        work: WorkId,
        state: QueueState,
        note: Option<&str>,
        now: Timestamp,
    ) -> Result<()> {
        let changed = self.connection.execute(
            "UPDATE queue SET state = ?2, note = ?3, updated_at = ?4 WHERE work_id = ?1",
            params![work.0, state.as_str(), note, now.to_string()],
        )?;
        if changed == 0 {
            return Err(StoreError::NotFound(format!(
                "the work {} is not in the queue",
                work.0
            )));
        }
        Ok(())
    }

    /// The works in a state, in the order that they entered the queue.
    pub fn in_state(&self, state: QueueState, limit: usize) -> Result<Vec<WorkId>> {
        let mut statement = self.connection.prepare(
            "SELECT work_id FROM queue WHERE state = ?1 ORDER BY added_at, work_id LIMIT ?2",
        )?;
        let limit = i64::try_from(limit).unwrap_or(i64::MAX);
        let ids = statement
            .query_map(params![state.as_str(), limit], |row| row.get(0))?
            .map(|id| id.map(WorkId))
            .collect::<rusqlite::Result<_>>()?;
        Ok(ids)
    }

    pub fn add_retrieval(&self, work: WorkId, retrieval: &Retrieval) -> Result<RetrievalId> {
        self.connection.execute(
            "INSERT INTO retrieval (work_id, role, supplement_key, url, media_type, sha256, bytes,
             software, retrieved_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                work.0,
                retrieval.role.as_str(),
                retrieval.supplement_key,
                retrieval.url,
                retrieval.media_type,
                retrieval.digest.as_str(),
                i64::try_from(retrieval.bytes).unwrap_or(i64::MAX),
                retrieval.software,
                retrieval.retrieved_at.to_string(),
            ],
        )?;
        Ok(RetrievalId(self.connection.last_insert_rowid()))
    }

    pub fn retrievals(&self, work: WorkId) -> Result<Vec<(RetrievalId, Retrieval)>> {
        let mut statement = self.connection.prepare(
            "SELECT id, role, supplement_key, url, media_type, sha256, bytes, software,
             retrieved_at FROM retrieval WHERE work_id = ?1 ORDER BY id",
        )?;
        let rows = statement
            .query_map([work.0], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        rows.into_iter()
            .map(
                |(id, role, supplement_key, url, media_type, digest, bytes, software, at)| {
                    Ok((
                        RetrievalId(id),
                        Retrieval {
                            role: RetrievalRole::parse(&role)?,
                            supplement_key,
                            url,
                            media_type,
                            digest: digest
                                .parse()
                                .map_err(|_| StoreError::Corrupt(format!("digest {digest:?}")))?,
                            bytes: u64::try_from(bytes).unwrap_or(0),
                            software,
                            retrieved_at: timestamp(&at)?,
                        },
                    ))
                },
            )
            .collect()
    }

    /// Adds a notice. A notice that is already in the database does not
    /// change.
    pub fn add_notice(&self, work: WorkId, notice: &Notice) -> Result<bool> {
        let added = self.connection.execute(
            "INSERT OR IGNORE INTO notice (work_id, kind, reported_type, source, notice_doi,
             notice_pmid, notice_citation, date, source_record, retrieved_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                work.0,
                notice.kind.as_str(),
                notice.reported_type,
                notice.source.as_str(),
                notice.notice_doi.as_ref().map(Doi::as_str),
                notice.notice_pmid.as_ref().map(Pmid::as_str),
                notice.notice_citation,
                notice.date.map(|date| date.to_string()),
                notice.source_record,
                notice.retrieved_at.to_string(),
            ],
        )?;
        Ok(added == 1)
    }

    pub fn notices(&self, work: WorkId) -> Result<Vec<Notice>> {
        let mut statement = self.connection.prepare(
            "SELECT kind, reported_type, source, notice_doi, notice_pmid, notice_citation, date,
             source_record, retrieved_at FROM notice WHERE work_id = ?1 ORDER BY date, id",
        )?;
        let rows = statement
            .query_map([work.0], |row| {
                Ok(serde_json::json!({
                    "kind": row.get::<_, String>(0)?,
                    "reported_type": row.get::<_, String>(1)?,
                    "source": row.get::<_, String>(2)?,
                    "notice_doi": row.get::<_, Option<String>>(3)?,
                    "notice_pmid": row.get::<_, Option<String>>(4)?,
                    "notice_citation": row.get::<_, Option<String>>(5)?,
                    "date": row.get::<_, Option<String>>(6)?,
                    "source_record": row.get::<_, Option<String>>(7)?,
                    "retrieved_at": row.get::<_, String>(8)?,
                }))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        rows.into_iter()
            .map(|row| Ok(serde_json::from_value(row)?))
            .collect()
    }
}
