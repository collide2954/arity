use std::collections::{BTreeMap, HashMap};
use std::ops::Range;

use arity_core::vocabulary::{
    AgentKind, Bound, CheckState, Comparison, Role, SignificanceStatement, Statistic, ValueType,
};
use arity_core::{Curie, Decimal, Digest};
use arity_protocol::{
    Checked, ConditionInput, EffectReportInput, EntityInput, EvidenceInput, MethodInput, Person,
    PropertyDefinition, ReportInput, SkipInput, ValueInput, ValueReportInput,
};
use arity_units::Unit;
use jiff::Timestamp;
use rusqlite::{Connection, OptionalExtension, params};

use crate::store::{
    AgentId, EntityId, ReportId, Result, RunId, SOFTWARE, Store, StoreError, TextId, WorkId,
};

/// An entity that a run of a work names, with the id that the run gives it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkEntity {
    pub local_id: String,
    pub entity: EntityId,
    pub kind: String,
    pub label: String,
}

fn number_text(value: Option<Decimal>) -> Option<String> {
    value.map(|value| value.to_string())
}

fn index(value: usize) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

fn agent(
    connection: &Connection,
    kind: AgentKind,
    name: &str,
    version: Option<&str>,
    orcid: Option<&str>,
) -> Result<AgentId> {
    connection.execute(
        "INSERT OR IGNORE INTO agent (kind, name, version, orcid) VALUES (?1, ?2, ?3, ?4)",
        params![kind.as_str(), name, version, orcid],
    )?;
    let id = connection.query_row(
        "SELECT id FROM agent WHERE kind = ?1 AND name = ?2
         AND COALESCE(version, '') = COALESCE(?3, '') AND COALESCE(orcid, '') = COALESCE(?4, '')",
        params![kind.as_str(), name, version, orcid],
        |row| row.get(0),
    )?;
    Ok(AgentId(id))
}

/// The columns of a quantity, with each value also in the coherent SI unit.
struct Quantity {
    number: Option<String>,
    low: Option<String>,
    high: Option<String>,
    unit: Option<String>,
    unit_text: Option<String>,
    coherent: Option<f64>,
    coherent_low: Option<f64>,
    coherent_high: Option<f64>,
    coherent_unit: Option<String>,
}

impl Quantity {
    fn of(value: &ValueInput) -> Self {
        let unit = value
            .unit
            .as_deref()
            .and_then(|code| Unit::parse(code).ok());
        let convert = |number: Option<Decimal>| {
            number
                .zip(unit.as_ref())
                .map(|(number, unit)| unit.to_coherent(number.to_f64()))
        };
        Self {
            number: number_text(value.number),
            low: number_text(value.low),
            high: number_text(value.high),
            unit: value.unit.clone(),
            unit_text: value.unit_text.clone(),
            coherent: convert(value.number),
            coherent_low: convert(value.low),
            coherent_high: convert(value.high),
            coherent_unit: unit.as_ref().map(Unit::coherent_code),
        }
    }
}

struct Writer<'a> {
    connection: &'a Connection,
    run: RunId,
    work: WorkId,
    text: TextId,
    positions: &'a BTreeMap<String, Range<usize>>,
    entities: HashMap<String, EntityId>,
}

impl Writer<'_> {
    fn skip(&self, position: usize, item: &SkipInput) -> Result<()> {
        let path = format!("/skipped/{position}");
        let range = self.positions.get(&path).ok_or_else(|| {
            StoreError::Corrupt(format!("the checks give no position for {path}"))
        })?;
        self.connection.execute(
            "INSERT INTO skip (run_id, text_id, path, block_key, quote, start, \"end\", reason,
             note) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                self.run.0,
                self.text.0,
                path,
                item.block,
                item.quote,
                index(range.start),
                index(range.end),
                item.reason.as_str(),
                item.note
            ],
        )?;
        Ok(())
    }

    fn evidence(
        &self,
        path: &str,
        item: &EvidenceInput,
        report: Option<ReportId>,
        entity: Option<EntityId>,
    ) -> Result<()> {
        let range = self.positions.get(path).ok_or_else(|| {
            StoreError::Corrupt(format!("the checks give no position for {path}"))
        })?;
        self.connection.execute(
            "INSERT INTO evidence (run_id, text_id, report_id, entity_id, path, block_key, quote,
             start, \"end\") VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                self.run.0,
                self.text.0,
                report.map(|report| report.0),
                entity.map(|entity| entity.0),
                path,
                item.block,
                item.quote,
                index(range.start),
                index(range.end)
            ],
        )?;
        Ok(())
    }

    fn property(&self, definition: &PropertyDefinition) -> Result<()> {
        let dimension = definition
            .unit
            .as_deref()
            .and_then(|code| Unit::parse(code).ok())
            .map(|unit| unit.dimension().to_string());
        self.connection.execute(
            "INSERT INTO property (key, label, value_type, unit, dimension, categories, xrefs,
             description, run_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                definition.key,
                definition.label,
                definition.value_type.as_str(),
                definition.unit,
                dimension,
                serde_json::to_string(&definition.categories)?,
                serde_json::to_string(&definition.xrefs)?,
                definition.description,
                self.run.0
            ],
        )?;
        Ok(())
    }

    /// The entity that an earlier run of the same work names with this id.
    fn earlier_entity(&self, local_id: &str) -> Result<Option<EntityId>> {
        Ok(self
            .connection
            .query_row(
                "SELECT run_entity.entity_id FROM run_entity
                 JOIN run ON run.id = run_entity.run_id
                 WHERE run.work_id = ?1 AND run_entity.local_id = ?2 AND run.id != ?3
                 ORDER BY run_entity.run_id LIMIT 1",
                params![self.work.0, local_id, self.run.0],
                |row| row.get(0),
            )
            .optional()?
            .map(EntityId))
    }

    fn entity(&mut self, position: usize, entity: &EntityInput) -> Result<()> {
        let mut found: Option<(EntityId, String)> = None;
        for xref in &entity.xrefs {
            let curie = xref.to_string();
            let existing: Option<i64> = self
                .connection
                .query_row(
                    "SELECT entity_id FROM entity_xref WHERE curie = ?1",
                    [&curie],
                    |row| row.get(0),
                )
                .optional()?;
            match (&found, existing) {
                (Some((id, other)), Some(existing)) if id.0 != existing => {
                    return Err(StoreError::Conflict(format!(
                        "{other} and {curie} belong to different entities in the database"
                    )));
                }
                (None, Some(existing)) => found = Some((EntityId(existing), curie)),
                _ => {}
            }
        }
        let found = match (found, self.earlier_entity(&entity.id)?) {
            (Some((id, curie)), Some(earlier)) if id != earlier => {
                return Err(StoreError::Conflict(format!(
                    "the id {} names entity {} in an earlier run of this work, but {curie} belongs to entity {}",
                    entity.id, earlier.0, id.0
                )));
            }
            (Some(found), _) => Some(found),
            (None, Some(earlier)) => Some((earlier, format!("the id {}", entity.id))),
            (None, None) => None,
        };
        let id = if let Some((id, curie)) = found {
            let kind: String = self.connection.query_row(
                "SELECT kind FROM entity WHERE id = ?1",
                [id.0],
                |row| row.get(0),
            )?;
            if kind != entity.kind.as_str() {
                return Err(StoreError::Conflict(format!(
                    "{curie} belongs to an entity of the kind {kind}, not {}",
                    entity.kind
                )));
            }
            id
        } else {
            self.connection.execute(
                "INSERT INTO entity (kind, label, run_id) VALUES (?1, ?2, ?3)",
                params![entity.kind.as_str(), entity.label, self.run.0],
            )?;
            EntityId(self.connection.last_insert_rowid())
        };
        for xref in &entity.xrefs {
            self.connection.execute(
                "INSERT OR IGNORE INTO entity_xref (curie, entity_id) VALUES (?1, ?2)",
                params![xref.to_string(), id.0],
            )?;
        }
        self.connection.execute(
            "INSERT OR IGNORE INTO entity_name (entity_id, name, run_id) VALUES (?1, ?2, ?3)",
            params![id.0, entity.label, self.run.0],
        )?;
        for (mention_index, mention) in entity.mentions.iter().enumerate() {
            let path = format!("/entities/{position}/mentions/{mention_index}");
            self.evidence(&path, mention, None, Some(id))?;
        }
        self.connection.execute(
            "INSERT INTO run_entity (run_id, local_id, entity_id) VALUES (?1, ?2, ?3)",
            params![self.run.0, entity.id, id.0],
        )?;
        self.entities.insert(entity.id.clone(), id);
        Ok(())
    }

    fn entity_id(&self, local: &str) -> Result<EntityId> {
        self.entities
            .get(local)
            .copied()
            .ok_or_else(|| StoreError::Corrupt(format!("the submission has no entity {local:?}")))
    }

    fn method_columns(method: Option<&MethodInput>) -> Result<(Option<String>, String)> {
        let label = method.map(|method| method.label.clone());
        let xrefs: &[Curie] = method.map_or(&[], |method| &method.xrefs);
        Ok((label, serde_json::to_string(xrefs)?))
    }

    fn value_report(
        &self,
        position: usize,
        report: &ValueReportInput,
        body: &str,
    ) -> Result<ReportId> {
        let value = &report.value;
        let quantity = Quantity::of(value);
        let uncertainty = value.uncertainty.as_ref();
        let figure = value.read_from_figure.as_ref();
        let (method_label, method_xrefs) = Self::method_columns(report.method.as_ref())?;
        self.connection.execute(
            "INSERT INTO report (run_id, work_id, local_id, type, body, property_key, number, low,
             high, unit, unit_text, coherent, coherent_low, coherent_high, coherent_unit, bound,
             statistic, n, boolean, category, uncertainty_kind, uncertainty_number,
             uncertainty_low, uncertainty_high, uncertainty_level, read_from_figure, figure_block,
             figure_method, figure_resolution, method_label, method_xrefs, note)
             VALUES (?1, ?2, ?3, 'value', ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
             ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31)",
            params![
                self.run.0,
                self.work.0,
                report.id,
                body,
                report.property,
                quantity.number,
                quantity.low,
                quantity.high,
                quantity.unit,
                quantity.unit_text,
                quantity.coherent,
                quantity.coherent_low,
                quantity.coherent_high,
                quantity.coherent_unit,
                value.bound.map(Bound::as_str),
                value.statistic.map(Statistic::as_str),
                value.n,
                value.boolean,
                value.category,
                uncertainty.map(|u| u.kind.as_str()),
                number_text(uncertainty.and_then(|u| u.number)),
                number_text(uncertainty.and_then(|u| u.low)),
                number_text(uncertainty.and_then(|u| u.high)),
                number_text(uncertainty.and_then(|u| u.level)),
                figure.is_some(),
                figure.map(|f| f.figure.clone()),
                figure.map(|f| f.method.clone()),
                number_text(figure.map(|f| f.resolution)),
                method_label,
                method_xrefs,
                report.note
            ],
        )?;
        let id = ReportId(self.connection.last_insert_rowid());
        for (subject_index, subject) in report.subject.iter().enumerate() {
            self.connection.execute(
                "INSERT INTO report_entity (report_id, role, position, entity_id)
                 VALUES (?1, 'subject', ?2, ?3)",
                params![id.0, index(subject_index), self.entity_id(subject)?.0],
            )?;
        }
        self.conditions(id, &report.conditions)?;
        for (evidence_index, item) in report.evidence.iter().enumerate() {
            let path = format!("/reports/{position}/evidence/{evidence_index}");
            self.evidence(&path, item, Some(id), None)?;
        }
        Ok(id)
    }

    fn effect_report(
        &self,
        position: usize,
        report: &EffectReportInput,
        body: &str,
    ) -> Result<ReportId> {
        let magnitude = report.magnitude.as_ref();
        let interval = magnitude.and_then(|m| m.interval.as_ref());
        let significance = report.significance.as_ref();
        let (method_label, method_xrefs) = Self::method_columns(report.method.as_ref())?;
        self.connection.execute(
            "INSERT INTO report (run_id, work_id, local_id, type, body, target_entity_id,
             target_property_key, direction, relation, magnitude_measure, magnitude_number,
             magnitude_unit, uncertainty_kind, uncertainty_number, uncertainty_low,
             uncertainty_high, uncertainty_level, comparator, significance, p_comparison, p_value,
             test, n, method_label, method_xrefs, note)
             VALUES (?1, ?2, ?3, 'effect', ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
             ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25)",
            params![
                self.run.0,
                self.work.0,
                report.id,
                body,
                self.entity_id(&report.target.entity)?.0,
                report.target.property,
                report.direction.as_str(),
                report.relation.as_str(),
                magnitude.map(|m| m.measure.as_str()),
                number_text(magnitude.map(|m| m.number)),
                magnitude.and_then(|m| m.unit.clone()),
                interval.map(|i| i.kind.as_str()),
                number_text(interval.and_then(|i| i.number)),
                number_text(interval.and_then(|i| i.low)),
                number_text(interval.and_then(|i| i.high)),
                number_text(interval.and_then(|i| i.level)),
                report.comparator,
                significance
                    .and_then(|s| s.statement)
                    .map(SignificanceStatement::as_str),
                significance
                    .and_then(|s| s.comparison)
                    .map(Comparison::as_str),
                number_text(significance.and_then(|s| s.p)),
                significance.and_then(|s| s.test.clone()),
                report.n,
                method_label,
                method_xrefs,
                report.note
            ],
        )?;
        let id = ReportId(self.connection.last_insert_rowid());
        for (agent_index, agent) in report.agents.iter().enumerate() {
            let dose = agent.dose.as_ref().map(Quantity::of);
            self.connection.execute(
                "INSERT INTO report_entity (report_id, role, position, entity_id, dose_number,
                 dose_unit, dose_unit_text, dose_coherent) VALUES (?1, 'agent', ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    id.0,
                    index(agent_index),
                    self.entity_id(&agent.entity)?.0,
                    dose.as_ref().and_then(|d| d.number.clone()),
                    dose.as_ref().and_then(|d| d.unit.clone()),
                    dose.as_ref().and_then(|d| d.unit_text.clone()),
                    dose.as_ref().and_then(|d| d.coherent)
                ],
            )?;
        }
        self.conditions(id, &report.conditions)?;
        for (evidence_index, item) in report.evidence.iter().enumerate() {
            let path = format!("/reports/{position}/evidence/{evidence_index}");
            self.evidence(&path, item, Some(id), None)?;
        }
        Ok(id)
    }

    fn conditions(&self, report: ReportId, conditions: &[ConditionInput]) -> Result<()> {
        for (position, condition) in conditions.iter().enumerate() {
            let entity = condition
                .entity
                .as_deref()
                .map(|local| self.entity_id(local))
                .transpose()?;
            let quantity = condition.value.as_ref().map(Quantity::of);
            let value = condition.value.as_ref();
            self.connection.execute(
                "INSERT INTO condition (report_id, position, role, entity_id, text, property_key,
                 number, low, high, unit, unit_text, coherent, boolean, category)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                params![
                    report.0,
                    index(position),
                    condition.role.map(Role::as_str),
                    entity.map(|entity| entity.0),
                    condition.text,
                    condition.property,
                    quantity.as_ref().and_then(|q| q.number.clone()),
                    quantity.as_ref().and_then(|q| q.low.clone()),
                    quantity.as_ref().and_then(|q| q.high.clone()),
                    quantity.as_ref().and_then(|q| q.unit.clone()),
                    quantity.as_ref().and_then(|q| q.unit_text.clone()),
                    quantity.as_ref().and_then(|q| q.coherent),
                    value.and_then(|v| v.boolean),
                    value.and_then(|v| v.category.clone())
                ],
            )?;
        }
        Ok(())
    }
}

impl Store {
    pub fn run_by_digest(&self, digest: &Digest) -> Result<Option<RunId>> {
        Ok(self
            .connection
            .query_row(
                "SELECT id FROM run WHERE submission_sha256 = ?1",
                [digest.as_str()],
                |row| row.get(0),
            )
            .optional()?
            .map(RunId))
    }

    /// Stores a submission that passed the checks, with all its reports and
    /// their provenance, in one transaction. A submission that is already in
    /// the database gives its stored run.
    pub fn store_submission(
        &self,
        work: WorkId,
        text: TextId,
        raw: &[u8],
        checked: &Checked,
        now: Timestamp,
    ) -> Result<RunId> {
        if Digest::of(raw) != checked.digest {
            return Err(StoreError::Conflict(
                "the file is not the file that the checks read".to_owned(),
            ));
        }
        if let Some(run) = self.run_by_digest(&checked.digest)? {
            return Ok(run);
        }
        let (text_work, document) = self.text(text)?;
        if text_work != work || document.digest()? != checked.submission.text {
            return Err(StoreError::Conflict(
                "the submission does not name this text of this work".to_owned(),
            ));
        }
        let raw_text = std::str::from_utf8(raw)
            .map_err(|_| StoreError::Corrupt("the submission is not UTF-8".to_owned()))?;
        let submission = &checked.submission;
        if let Some(replaced) = submission.replaces {
            self.check_replaceable(work, RunId(replaced))?;
        }
        let transaction = self.connection.unchecked_transaction()?;
        let reader = agent(
            &transaction,
            submission.reader.kind,
            &submission.reader.name,
            submission.reader.version.as_deref(),
            submission
                .reader
                .orcid
                .as_ref()
                .map(arity_core::Orcid::as_str),
        )?;
        let responsible = agent(
            &transaction,
            AgentKind::Person,
            &submission.responsible.name,
            None,
            submission
                .responsible
                .orcid
                .as_ref()
                .map(arity_core::Orcid::as_str),
        )?;
        transaction.execute(
            "INSERT INTO run (work_id, text_id, submission_sha256, submission, reader_id,
             responsible_id, software, warnings, submitted_at, replaces)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                work.0,
                text.0,
                checked.digest.as_str(),
                raw_text,
                reader.0,
                responsible.0,
                SOFTWARE,
                serde_json::to_string(&checked.warnings)?,
                now.to_string(),
                submission.replaces
            ],
        )?;
        let mut writer = Writer {
            connection: &transaction,
            run: RunId(transaction.last_insert_rowid()),
            work,
            text,
            positions: &checked.positions,
            entities: HashMap::new(),
        };
        for definition in &checked.new_properties {
            writer.property(definition)?;
        }
        for (position, entity) in submission.entities.iter().enumerate() {
            writer.entity(position, entity)?;
        }
        for (position, report) in submission.reports.iter().enumerate() {
            let body = serde_json::to_string(report)?;
            match report {
                ReportInput::Value(value) => writer.value_report(position, value, &body)?,
                ReportInput::Effect(effect) => writer.effect_report(position, effect, &body)?,
            };
        }
        for (position, skip) in submission.skipped.iter().enumerate() {
            writer.skip(position, skip)?;
        }
        let run = writer.run;
        transaction.commit()?;
        Ok(run)
    }

    /// Records the result of a person's check of a report against its
    /// source.
    pub fn record_check(
        &self,
        report: ReportId,
        state: CheckState,
        person: &Person,
        note: Option<&str>,
        now: Timestamp,
    ) -> Result<()> {
        if person.name.trim().is_empty() {
            return Err(StoreError::Conflict(
                "a check needs the name of the person".to_owned(),
            ));
        }
        let person = agent(
            &self.connection,
            AgentKind::Person,
            &person.name,
            None,
            person.orcid.as_ref().map(arity_core::Orcid::as_str),
        )?;
        self.connection.execute(
            "INSERT INTO check_event (report_id, state, person_id, note, at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![report.0, state.as_str(), person.0, note, now.to_string()],
        )?;
        Ok(())
    }

    pub fn check_state(&self, report: ReportId) -> Result<CheckState> {
        let state: String = self
            .connection
            .query_row(
                "SELECT state FROM report_state WHERE report_id = ?1",
                [report.0],
                |row| row.get(0),
            )
            .optional()?
            .ok_or_else(|| StoreError::NotFound(format!("no report has the id {}", report.0)))?;
        state
            .parse()
            .map_err(|_| StoreError::Corrupt(format!("check state {state:?}")))
    }

    pub fn entity_count(&self) -> Result<u64> {
        let count: i64 = self
            .connection
            .query_row("SELECT COUNT(*) FROM entity", [], |row| row.get(0))?;
        Ok(u64::try_from(count).unwrap_or(0))
    }

    /// The entities that the runs of a work name, each with the id of its
    /// first run, in the order of the runs.
    pub fn entities_of_work(&self, work: WorkId) -> Result<Vec<WorkEntity>> {
        let mut statement = self.connection.prepare(
            "SELECT run_entity.local_id, entity.id, entity.kind, entity.label FROM run_entity
             JOIN run ON run.id = run_entity.run_id
             JOIN entity ON entity.id = run_entity.entity_id
             WHERE run.work_id = ?1 ORDER BY run_entity.run_id, run_entity.rowid",
        )?;
        let rows = statement
            .query_map([work.0], |row| {
                Ok(WorkEntity {
                    local_id: row.get(0)?,
                    entity: EntityId(row.get(1)?),
                    kind: row.get(2)?,
                    label: row.get(3)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let mut seen = std::collections::HashSet::new();
        Ok(rows
            .into_iter()
            .filter(|row| seen.insert(row.local_id.clone()))
            .collect())
    }

    /// Makes sure that a new run of a work can replace a run: the run exists,
    /// it is a run of the same work, and no other run replaces it already.
    fn check_replaceable(&self, work: WorkId, run: RunId) -> Result<()> {
        match self.work_of_run(run)? {
            None => Err(StoreError::Conflict(format!(
                "there is no run {} to replace",
                run.0
            ))),
            Some(owner) if owner != work => Err(StoreError::Conflict(format!(
                "run {} is a run of another work",
                run.0
            ))),
            Some(_) => {
                let newer: Option<i64> = self
                    .connection
                    .query_row("SELECT id FROM run WHERE replaces = ?1", [run.0], |row| {
                        row.get(0)
                    })
                    .optional()?;
                match newer {
                    Some(newer) => Err(StoreError::Conflict(format!(
                        "run {} is already replaced by run {newer}",
                        run.0
                    ))),
                    None => Ok(()),
                }
            }
        }
    }

    /// The work of a run, or nothing when there is no such run.
    pub fn work_of_run(&self, run: RunId) -> Result<Option<WorkId>> {
        Ok(self
            .connection
            .query_row("SELECT work_id FROM run WHERE id = ?1", [run.0], |row| {
                row.get(0).map(WorkId)
            })
            .optional()?)
    }

    pub fn reports_of_run(&self, run: RunId) -> Result<Vec<(ReportId, String)>> {
        let mut statement = self
            .connection
            .prepare("SELECT id, local_id FROM report WHERE run_id = ?1 ORDER BY id")?;
        let rows = statement
            .query_map([run.0], |row| Ok((ReportId(row.get(0)?), row.get(1)?)))?
            .collect::<rusqlite::Result<_>>()?;
        Ok(rows)
    }

    /// All property definitions, to give to the checks.
    pub fn properties(&self) -> Result<Vec<PropertyDefinition>> {
        let mut statement = self.connection.prepare(
            "SELECT key, label, value_type, unit, categories, xrefs, description
             FROM property ORDER BY key",
        )?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Option<String>>(6)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        rows.into_iter()
            .map(
                |(key, label, value_type, unit, categories, xrefs, description)| {
                    Ok(PropertyDefinition {
                        key,
                        label,
                        value_type: value_type
                            .parse::<ValueType>()
                            .map_err(|error| StoreError::Corrupt(error.to_string()))?,
                        unit,
                        categories: serde_json::from_str(&categories)?,
                        xrefs: serde_json::from_str(&xrefs)?,
                        description,
                    })
                },
            )
            .collect()
    }
}
