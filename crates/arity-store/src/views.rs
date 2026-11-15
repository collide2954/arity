//! Read-only views of the database, in the form that the website shows.

use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, BTreeSet, HashMap};

use arity_core::vocabulary::Direction;
use arity_core::{Notice, WorkMetadata};
use rusqlite::params;
use serde::Serialize;
use serde_json::Value;

use crate::store::{Result, Store, StoreError, WorkId};

/// The entity ids that a report body uses: its subjects, its agents, its
/// target, and the entities of its conditions.
fn used_entities(body: &Value) -> BTreeSet<&str> {
    let mut used: BTreeSet<&str> = body["subject"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    for list in ["agents", "conditions"] {
        used.extend(
            body[list]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|item| item["entity"].as_str()),
        );
    }
    used.extend(body["target"]["entity"].as_str());
    used
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EntityView {
    pub id: i64,
    pub kind: String,
    /// The label that the first run gave the entity. When no current run
    /// gives that label, the first label of the earliest current run.
    pub label: String,
    pub xrefs: Vec<String>,
    /// The labels that the current runs give the entity. For an entity that
    /// only replaced runs name, the labels of those runs.
    pub names: Vec<String>,
    /// Whether a current run names the entity.
    pub current: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PropertyView {
    pub key: String,
    pub label: String,
    pub value_type: String,
    pub unit: Option<String>,
    pub dimension: Option<String>,
    pub categories: Vec<String>,
    pub xrefs: Vec<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CheckView {
    pub state: String,
    pub person: String,
    pub orcid: Option<String>,
    pub note: Option<String>,
    pub at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EvidenceView {
    pub block: String,
    pub quote: String,
    pub start: i64,
    pub end: i64,
}

/// A value in the coherent SI unit of its dimension. arity calculates it
/// from the reported value; it is not a value that the source states.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CoherentView {
    pub value: Option<f64>,
    pub low: Option<f64>,
    pub high: Option<f64>,
    pub unit: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReportView {
    pub id: i64,
    pub work: i64,
    pub run: i64,
    pub local_id: String,
    pub kind: String,
    /// The report exactly as the reader submitted it.
    pub body: Value,
    /// The stored entity for each entity id that the body uses.
    pub entities: BTreeMap<String, i64>,
    pub property: Option<String>,
    pub coherent: CoherentView,
    pub direction: Option<String>,
    pub score: Option<i8>,
    pub check: String,
    pub checks: Vec<CheckView>,
    pub evidence: Vec<EvidenceView>,
    /// The warnings of the checks about this report.
    pub warnings: Vec<Value>,
    /// The run that corrects the run of this report, if there is one.
    pub replaced_by: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AgentView {
    pub kind: String,
    pub name: String,
    pub version: Option<String>,
    pub orcid: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RunView {
    pub id: i64,
    pub work: i64,
    pub reader: AgentView,
    pub responsible: AgentView,
    pub software: String,
    pub submitted_at: String,
    pub submission_sha256: String,
    pub text_sha256: String,
    pub parser: String,
    pub warnings: Value,
    /// The note of the reader on the whole run, for example what it corrects.
    pub note: Option<String>,
    /// The earlier run that this run corrects.
    pub replaces: Option<i64>,
    /// The later run that corrects this run.
    pub replaced_by: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RetrievalView {
    pub role: String,
    pub supplement_key: Option<String>,
    pub url: String,
    pub media_type: Option<String>,
    pub sha256: String,
    pub bytes: i64,
    pub software: String,
    pub retrieved_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WorkView {
    pub id: i64,
    pub metadata: WorkMetadata,
    pub state: Option<String>,
    pub notices: Vec<Notice>,
    pub retrievals: Vec<RetrievalView>,
    pub text_sha256: Option<String>,
    pub reports: u64,
}

fn json_list(text: &str) -> Result<Vec<String>> {
    Ok(serde_json::from_str(text)?)
}

impl Store {
    pub fn entity_views(&self) -> Result<Vec<EntityView>> {
        let mut xrefs: HashMap<i64, Vec<String>> = HashMap::new();
        let mut statement = self
            .connection
            .prepare("SELECT entity_id, curie FROM entity_xref ORDER BY curie")?;
        for row in statement.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))? {
            let (entity, curie): (i64, String) = row?;
            xrefs.entry(entity).or_default().push(curie);
        }
        let mut given: HashMap<i64, Vec<(String, bool)>> = HashMap::new();
        let mut statement = self.connection.prepare(
            "SELECT entity_id, name,
             NOT EXISTS (SELECT 1 FROM run WHERE run.replaces = entity_name.run_id)
             FROM entity_name ORDER BY run_id, name",
        )?;
        for row in statement.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))? {
            let (entity, name, from_current): (i64, String, bool) = row?;
            given.entry(entity).or_default().push((name, from_current));
        }
        let mut statement = self
            .connection
            .prepare("SELECT id, kind, label FROM entity ORDER BY id")?;
        let rows = statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
            .collect::<rusqlite::Result<Vec<(i64, String, String)>>>()?;
        Ok(rows
            .into_iter()
            .map(|(id, kind, label)| {
                let given = given.remove(&id).unwrap_or_default();
                let current: Vec<&str> = given
                    .iter()
                    .filter(|(_, from_current)| *from_current)
                    .map(|(name, _)| name.as_str())
                    .collect();
                let label = match current.first() {
                    Some(first) if !current.contains(&label.as_str()) => (*first).to_owned(),
                    _ => label,
                };
                let names: BTreeSet<&str> = given
                    .iter()
                    .filter(|(_, from_current)| *from_current || current.is_empty())
                    .map(|(name, _)| name.as_str())
                    .collect();
                EntityView {
                    id,
                    kind,
                    label,
                    xrefs: xrefs.remove(&id).unwrap_or_default(),
                    names: names.into_iter().map(str::to_owned).collect(),
                    current: !current.is_empty(),
                }
            })
            .collect())
    }

    pub fn property_views(&self) -> Result<Vec<PropertyView>> {
        let mut statement = self.connection.prepare(
            "SELECT key, label, value_type, unit, dimension, categories, xrefs, description
             FROM property ORDER BY key",
        )?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, Option<String>>(7)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        rows.into_iter()
            .map(
                |(key, label, value_type, unit, dimension, categories, xrefs, description)| {
                    Ok(PropertyView {
                        key,
                        label,
                        value_type,
                        unit,
                        dimension,
                        categories: json_list(&categories)?,
                        xrefs: json_list(&xrefs)?,
                        description,
                    })
                },
            )
            .collect()
    }

    fn agent_view(&self, id: i64) -> Result<AgentView> {
        Ok(self.connection.query_row(
            "SELECT kind, name, version, orcid FROM agent WHERE id = ?1",
            [id],
            |row| {
                Ok(AgentView {
                    kind: row.get(0)?,
                    name: row.get(1)?,
                    version: row.get(2)?,
                    orcid: row.get(3)?,
                })
            },
        )?)
    }

    pub fn run_views(&self) -> Result<Vec<RunView>> {
        let mut statement = self.connection.prepare(
            "SELECT run.id, run.work_id, run.reader_id, run.responsible_id, run.software,
             run.submitted_at, run.submission_sha256, text.sha256, text.parser, run.warnings,
             json_extract(run.submission, '$.note'), run.replaces, newer.id
             FROM run JOIN text ON text.id = run.text_id
             LEFT JOIN run AS newer ON newer.replaces = run.id
             ORDER BY run.id",
        )?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, String>(9)?,
                    row.get::<_, Option<String>>(10)?,
                    row.get::<_, Option<i64>>(11)?,
                    row.get::<_, Option<i64>>(12)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        rows.into_iter()
            .map(
                |(
                    id,
                    work,
                    reader,
                    responsible,
                    software,
                    at,
                    submission,
                    text,
                    parser,
                    warnings,
                    note,
                    replaces,
                    replaced_by,
                )| {
                    Ok(RunView {
                        id,
                        work,
                        reader: self.agent_view(reader)?,
                        responsible: self.agent_view(responsible)?,
                        software,
                        submitted_at: at,
                        submission_sha256: submission,
                        text_sha256: text,
                        parser,
                        warnings: serde_json::from_str(&warnings)?,
                        note,
                        replaces,
                        replaced_by,
                    })
                },
            )
            .collect()
    }

    fn checks_of(&self, report: i64) -> Result<Vec<CheckView>> {
        let mut statement = self.connection.prepare(
            "SELECT check_event.state, agent.name, agent.orcid, check_event.note, check_event.at
             FROM check_event JOIN agent ON agent.id = check_event.person_id
             WHERE check_event.report_id = ?1 ORDER BY check_event.at, check_event.id",
        )?;
        let rows = statement
            .query_map([report], |row| {
                Ok(CheckView {
                    state: row.get(0)?,
                    person: row.get(1)?,
                    orcid: row.get(2)?,
                    note: row.get(3)?,
                    at: row.get(4)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?;
        Ok(rows)
    }

    fn evidence_of(&self, report: i64) -> Result<Vec<EvidenceView>> {
        let mut statement = self.connection.prepare(
            "SELECT block_key, quote, start, \"end\" FROM evidence WHERE report_id = ?1 ORDER BY id",
        )?;
        let rows = statement
            .query_map([report], |row| {
                Ok(EvidenceView {
                    block: row.get(0)?,
                    quote: row.get(1)?,
                    start: row.get(2)?,
                    end: row.get(3)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?;
        Ok(rows)
    }

    fn run_entities(&self, run: i64) -> Result<BTreeMap<String, i64>> {
        let mut statement = self
            .connection
            .prepare("SELECT local_id, entity_id FROM run_entity WHERE run_id = ?1")?;
        let rows = statement
            .query_map([run], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<rusqlite::Result<_>>()?;
        Ok(rows)
    }

    pub fn report_views(&self) -> Result<Vec<ReportView>> {
        let mut statement = self.connection.prepare(
            "SELECT report.id, report.work_id, report.run_id, report.local_id, report.type,
             report.body, COALESCE(report.property_key, report.target_property_key),
             report.coherent, report.coherent_low, report.coherent_high, report.coherent_unit,
             report.direction, report_state.state, newer.id
             FROM report JOIN report_state ON report_state.report_id = report.id
             LEFT JOIN run AS newer ON newer.replaces = report.run_id
             ORDER BY report.id",
        )?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    CoherentView {
                        value: row.get(7)?,
                        low: row.get(8)?,
                        high: row.get(9)?,
                        unit: row.get(10)?,
                    },
                    row.get::<_, Option<String>>(11)?,
                    row.get::<_, String>(12)?,
                    row.get::<_, Option<i64>>(13)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let mut entities_by_run: HashMap<i64, BTreeMap<String, i64>> = HashMap::new();
        let mut warnings_by_run: HashMap<i64, HashMap<String, Vec<Value>>> = HashMap::new();
        rows.into_iter()
            .map(
                |(
                    id,
                    work,
                    run,
                    local_id,
                    kind,
                    body,
                    property,
                    coherent,
                    direction,
                    check,
                    replaced_by,
                )| {
                    let body: Value = serde_json::from_str(&body)?;
                    let of_run = match entities_by_run.entry(run) {
                        Entry::Occupied(entry) => entry.into_mut(),
                        Entry::Vacant(entry) => entry.insert(self.run_entities(run)?),
                    };
                    let used = used_entities(&body);
                    let entities = of_run
                        .iter()
                        .filter(|(local, _)| used.contains(local.as_str()))
                        .map(|(local, entity)| (local.clone(), *entity))
                        .collect();
                    let warnings = match warnings_by_run.entry(run) {
                        Entry::Occupied(entry) => entry.into_mut(),
                        Entry::Vacant(entry) => entry.insert(self.report_warnings(run)?),
                    }
                    .remove(&local_id)
                    .unwrap_or_default();
                    let score = direction
                        .as_deref()
                        .map(str::parse::<Direction>)
                        .transpose()
                        .map_err(|error| StoreError::Corrupt(error.to_string()))?
                        .and_then(Direction::score);
                    Ok(ReportView {
                        id,
                        work,
                        run,
                        local_id,
                        kind,
                        body,
                        entities,
                        property,
                        coherent,
                        direction,
                        score,
                        check,
                        checks: self.checks_of(id)?,
                        evidence: self.evidence_of(id)?,
                        warnings,
                        replaced_by,
                    })
                },
            )
            .collect()
    }

    /// The warnings of the checks of a run, for the local id of each report.
    /// A warning about a report has a path that starts with the place of the
    /// report in the submission, for example `/reports/3/`.
    fn report_warnings(&self, run: i64) -> Result<HashMap<String, Vec<Value>>> {
        let (submission, warnings): (String, String) = self.connection.query_row(
            "SELECT submission, warnings FROM run WHERE id = ?1",
            [run],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        let submission: Value = serde_json::from_str(&submission)?;
        let locals: Vec<&str> = submission["reports"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|report| report["id"].as_str().unwrap_or_default())
            .collect();
        let mut by_report: HashMap<String, Vec<Value>> = HashMap::new();
        for warning in serde_json::from_str::<Vec<Value>>(&warnings)? {
            let local = warning["path"]
                .as_str()
                .and_then(|path| path.strip_prefix("/reports/"))
                .and_then(|rest| rest.split('/').next())
                .and_then(|place| place.parse::<usize>().ok())
                .and_then(|place| locals.get(place));
            if let Some(local) = local {
                by_report
                    .entry((*local).to_owned())
                    .or_default()
                    .push(warning);
            }
        }
        Ok(by_report)
    }

    fn retrieval_views(&self, work: WorkId) -> Result<Vec<RetrievalView>> {
        let mut statement = self.connection.prepare(
            "SELECT role, supplement_key, url, media_type, sha256, bytes, software, retrieved_at
             FROM retrieval WHERE work_id = ?1 ORDER BY id",
        )?;
        let rows = statement
            .query_map([work.0], |row| {
                Ok(RetrievalView {
                    role: row.get(0)?,
                    supplement_key: row.get(1)?,
                    url: row.get(2)?,
                    media_type: row.get(3)?,
                    sha256: row.get(4)?,
                    bytes: row.get(5)?,
                    software: row.get(6)?,
                    retrieved_at: row.get(7)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?;
        Ok(rows)
    }

    pub fn work_views(&self) -> Result<Vec<WorkView>> {
        self.works()?
            .into_iter()
            .map(|record| {
                let text_sha256: Option<String> = self
                    .connection
                    .query_row(
                        "SELECT sha256 FROM text WHERE work_id = ?1 ORDER BY id DESC LIMIT 1",
                        [record.id.0],
                        |row| row.get(0),
                    )
                    .map(Some)
                    .or_else(|error| match error {
                        rusqlite::Error::QueryReturnedNoRows => Ok(None),
                        other => Err(other),
                    })?;
                let reports: i64 = self.connection.query_row(
                    "SELECT COUNT(*) FROM report WHERE work_id = ?1
                     AND NOT EXISTS (SELECT 1 FROM run WHERE run.replaces = report.run_id)",
                    params![record.id.0],
                    |row| row.get(0),
                )?;
                Ok(WorkView {
                    id: record.id.0,
                    notices: self.notices(record.id)?,
                    retrievals: self.retrieval_views(record.id)?,
                    state: record.state.map(|state| state.as_str().to_owned()),
                    metadata: record.metadata,
                    text_sha256,
                    reports: u64::try_from(reports).unwrap_or(0),
                })
            })
            .collect()
    }
}
