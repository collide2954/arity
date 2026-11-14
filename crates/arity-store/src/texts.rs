use std::collections::HashMap;
use std::ops::Range;

use arity_core::Digest;
use arity_text::Document;
use jiff::Timestamp;
use rusqlite::{OptionalExtension, params};

use crate::store::{Result, RetrievalId, Store, StoreError, TextId, WorkId};

impl Store {
    /// Stores the canonical text of a work, with one row for each block. A
    /// text with the same digest is stored only once.
    pub fn add_text(
        &self,
        work: WorkId,
        retrieval: RetrievalId,
        document: &Document,
        now: Timestamp,
    ) -> Result<TextId> {
        let json = document.to_canonical_json()?;
        let digest = Digest::of(&json);
        if let Some(id) = self.text_by_digest(&digest)? {
            return Ok(id);
        }
        let text = String::from_utf8(json)
            .map_err(|_| StoreError::Corrupt("canonical JSON is not UTF-8".to_owned()))?;
        let transaction = self.connection.unchecked_transaction()?;
        transaction.execute(
            "INSERT INTO text (work_id, retrieval_id, sha256, parser, document, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                work.0,
                retrieval.0,
                digest.as_str(),
                document.parser,
                text,
                now.to_string()
            ],
        )?;
        let id = TextId(transaction.last_insert_rowid());
        {
            let mut insert = transaction.prepare(
                "INSERT INTO block (text_id, key, position, kind, label, section, content)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            )?;
            for (position, block) in (0_i64..).zip(&document.blocks) {
                let section = (!block.section.is_empty()).then(|| block.section.join(" › "));
                insert.execute(params![
                    id.0,
                    block.key,
                    position,
                    block.kind.as_str(),
                    block.label,
                    section,
                    block.text
                ])?;
            }
        }
        transaction.commit()?;
        Ok(id)
    }

    pub fn text_by_digest(&self, digest: &Digest) -> Result<Option<TextId>> {
        Ok(self
            .connection
            .query_row(
                "SELECT id FROM text WHERE sha256 = ?1",
                [digest.as_str()],
                |row| row.get(0),
            )
            .optional()?
            .map(TextId))
    }

    pub fn text(&self, id: TextId) -> Result<(WorkId, Document)> {
        let (work, json): (i64, String) = self
            .connection
            .query_row(
                "SELECT work_id, document FROM text WHERE id = ?1",
                [id.0],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?
            .ok_or_else(|| StoreError::NotFound(format!("no text has the id {}", id.0)))?;
        Ok((WorkId(work), serde_json::from_str(&json)?))
    }

    /// The quoted ranges of each block of a text, from the evidence of the
    /// reports that no person rejected. Each range counts Unicode scalar
    /// values of the text of its block.
    pub fn quoted_ranges(&self, text: TextId) -> Result<HashMap<String, Vec<Range<usize>>>> {
        self.ranges(
            "SELECT evidence.block_key, evidence.start, evidence.\"end\"
             FROM evidence
             JOIN report_state ON report_state.report_id = evidence.report_id
             WHERE evidence.text_id = ?1 AND report_state.state != 'rejected'
             ORDER BY evidence.id",
            text,
        )
    }

    /// The ranges of each block of a text that readers skipped. Each range
    /// counts Unicode scalar values of the text of its block.
    pub fn skipped_ranges(&self, text: TextId) -> Result<HashMap<String, Vec<Range<usize>>>> {
        self.ranges(
            "SELECT block_key, start, \"end\" FROM skip WHERE text_id = ?1 ORDER BY id",
            text,
        )
    }

    fn ranges(&self, sql: &str, text: TextId) -> Result<HashMap<String, Vec<Range<usize>>>> {
        let mut statement = self.connection.prepare(sql)?;
        let rows = statement
            .query_map([text.0], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let mut ranges: HashMap<String, Vec<Range<usize>>> = HashMap::new();
        for (block, start, end) in rows {
            let position = |value: i64| {
                usize::try_from(value)
                    .map_err(|_| StoreError::Corrupt(format!("quote position {value}")))
            };
            ranges
                .entry(block)
                .or_default()
                .push(position(start)?..position(end)?);
        }
        Ok(ranges)
    }

    /// The newest canonical text of a work.
    pub fn latest_text(&self, work: WorkId) -> Result<Option<(TextId, Document)>> {
        let id: Option<i64> = self
            .connection
            .query_row(
                "SELECT id FROM text WHERE work_id = ?1 ORDER BY id DESC LIMIT 1",
                [work.0],
                |row| row.get(0),
            )
            .optional()?;
        id.map(|id| Ok((TextId(id), self.text(TextId(id))?.1)))
            .transpose()
    }
}
