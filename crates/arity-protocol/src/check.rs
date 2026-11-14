use std::collections::{BTreeMap, HashMap, HashSet};
use std::ops::Range;

use arity_core::vocabulary::{AgentKind, UncertaintyKind, ValueType};
use arity_core::{Decimal, Digest, EntityKind};
use arity_text::{BlockKind, Document, contains_number, normalize};
use arity_units::Unit;

use crate::cues;
use crate::issue::{Issue, Severity};
use crate::properties::{Properties, PropertyDefinition};
use crate::submission::{
    ConditionInput, EffectReportInput, EvidenceInput, FigureReadingInput, MagnitudeInput,
    MethodInput, PropertyInput, ReportInput, SUBMISSION_FORMAT, Submission, UncertaintyInput,
    ValueInput, ValueReportInput,
};

const LONG_QUOTE: usize = 400;
const MAX_QUOTE: usize = 1_000;

/// A submission that passed the checks.
#[derive(Debug, Clone, PartialEq)]
pub struct Checked {
    pub submission: Submission,
    /// The digest of the submission file.
    pub digest: Digest,
    /// For each evidence item, by JSON Pointer: the range of the quote in
    /// the text of its block, in Unicode scalar values.
    pub positions: BTreeMap<String, Range<usize>>,
    /// The properties that the submission defines for the first time.
    pub new_properties: Vec<PropertyDefinition>,
    pub warnings: Vec<Issue>,
}

/// A submission that did not pass the checks.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("the submission has {} errors", self.errors().count())]
pub struct Rejected {
    pub issues: Vec<Issue>,
}

impl Rejected {
    pub fn errors(&self) -> impl Iterator<Item = &Issue> {
        self.issues
            .iter()
            .filter(|issue| issue.severity == Severity::Error)
    }
}

/// Checks a submission file against the canonical text of its source.
pub fn check(
    raw: &[u8],
    document: &Document,
    properties: &dyn Properties,
) -> Result<Checked, Rejected> {
    let submission: Submission = serde_json::from_slice(raw).map_err(|error| Rejected {
        issues: vec![Issue {
            severity: Severity::Error,
            path: String::new(),
            code: "invalid_json",
            message: format!("the file is not a valid submission: {error}"),
        }],
    })?;
    let mut checker = Checker {
        document,
        registry: properties,
        issues: Vec::new(),
        positions: BTreeMap::new(),
        entities: HashMap::new(),
        properties: HashMap::new(),
        new_properties: Vec::new(),
    };
    checker.submission(&submission);
    if checker
        .issues
        .iter()
        .any(|issue| issue.severity == Severity::Error)
    {
        return Err(Rejected {
            issues: checker.issues,
        });
    }
    Ok(Checked {
        submission,
        digest: Digest::of(raw),
        positions: checker.positions,
        new_properties: checker.new_properties,
        warnings: checker.issues,
    })
}

struct Resolved {
    definition: PropertyDefinition,
    unit: Option<Unit>,
}

struct Checker<'a> {
    document: &'a Document,
    registry: &'a dyn Properties,
    issues: Vec<Issue>,
    positions: BTreeMap<String, Range<usize>>,
    entities: HashMap<String, EntityKind>,
    properties: HashMap<String, Resolved>,
    new_properties: Vec<PropertyDefinition>,
}

/// The evidence of one report, after the checks.
struct Evidence {
    quotes: Vec<String>,
    blocks: Vec<String>,
}

impl Evidence {
    fn has_number(&self, number: Decimal) -> bool {
        self.quotes
            .iter()
            .any(|quote| contains_number(quote, number))
    }

    fn has_text(&self, text: &str, ignore_case: bool) -> bool {
        let fold = |value: &str| {
            let normalized = normalize(value).as_str().to_owned();
            if ignore_case {
                normalized.to_lowercase()
            } else {
                normalized
            }
        };
        let needle = fold(text);
        !needle.is_empty()
            && self
                .quotes
                .iter()
                .any(|quote| fold(quote).contains(&needle))
    }
}

fn is_key(text: &str) -> bool {
    text.starts_with(|c: char| c.is_ascii_lowercase())
        && text
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

impl Checker<'_> {
    fn error(&mut self, path: impl Into<String>, code: &'static str, message: impl Into<String>) {
        self.issue(Severity::Error, path.into(), code, message.into());
    }

    fn warning(&mut self, path: impl Into<String>, code: &'static str, message: impl Into<String>) {
        self.issue(Severity::Warning, path.into(), code, message.into());
    }

    fn issue(&mut self, severity: Severity, path: String, code: &'static str, message: String) {
        self.issues.push(Issue {
            severity,
            path,
            code,
            message,
        });
    }

    fn submission(&mut self, submission: &Submission) {
        if submission.format != SUBMISSION_FORMAT {
            self.error(
                "/format",
                "unknown_format",
                format!("the format must be {SUBMISSION_FORMAT}"),
            );
        }
        match self.document.digest() {
            Ok(digest) if digest == submission.text => {}
            Ok(digest) => self.error(
                "/text",
                "text_mismatch",
                format!(
                    "the submission names the text {}, but the current text is {digest}; read the text again",
                    submission.text
                ),
            ),
            Err(error) => self.error("/text", "text_unreadable", error.to_string()),
        }
        if submission.reader.name.trim().is_empty() {
            self.error("/reader/name", "empty_name", "give the name of the reader");
        }
        if submission.reader.kind == AgentKind::Software && submission.reader.version.is_none() {
            self.warning(
                "/reader/version",
                "no_version",
                "give the version of the software that read the source",
            );
        }
        if submission.responsible.name.trim().is_empty() {
            self.error(
                "/responsible/name",
                "empty_name",
                "give the name of the responsible person",
            );
        }
        for (index, property) in submission.properties.iter().enumerate() {
            self.property(&format!("/properties/{index}"), property);
        }
        for (index, entity) in submission.entities.iter().enumerate() {
            let path = format!("/entities/{index}");
            self.entity(&path, entity);
        }
        if submission.reports.is_empty() {
            self.error("/reports", "no_reports", "the submission has no reports");
        }
        let mut ids = HashSet::new();
        for (index, report) in submission.reports.iter().enumerate() {
            let path = format!("/reports/{index}");
            let id = match report {
                ReportInput::Value(value) => &value.id,
                ReportInput::Effect(effect) => &effect.id,
            };
            if !ids.insert(id.clone()) {
                self.error(
                    format!("{path}/id"),
                    "duplicate_id",
                    format!("more than one report has the id {id:?}"),
                );
            }
            match report {
                ReportInput::Value(value) => self.value_report(&path, value),
                ReportInput::Effect(effect) => self.effect_report(&path, effect),
            }
        }
    }

    fn property(&mut self, path: &str, input: &PropertyInput) {
        if !is_key(&input.id) {
            self.error(
                format!("{path}/id"),
                "invalid_key",
                "a property key has lower-case letters, digits, and underscores, and starts with a letter",
            );
            return;
        }
        if self.properties.contains_key(&input.id) {
            self.error(
                format!("{path}/id"),
                "duplicate_id",
                format!("the property {:?} has more than one definition", input.id),
            );
            return;
        }
        let definition = match self.registry.property(&input.id) {
            Some(existing) => {
                self.compare_property(path, input, &existing);
                existing
            }
            None => match self.new_property(path, input) {
                Some(definition) => {
                    self.new_properties.push(definition.clone());
                    definition
                }
                None => return,
            },
        };
        let unit = definition
            .unit
            .as_deref()
            .and_then(|code| Unit::parse(code).ok());
        self.properties
            .insert(input.id.clone(), Resolved { definition, unit });
    }

    fn compare_property(
        &mut self,
        path: &str,
        input: &PropertyInput,
        existing: &PropertyDefinition,
    ) {
        if input
            .value_type
            .is_some_and(|value_type| value_type != existing.value_type)
        {
            self.error(
                format!("{path}/value_type"),
                "property_conflict",
                format!(
                    "the property {:?} takes the value type {}",
                    existing.key, existing.value_type
                ),
            );
        }
        if let (Some(code), Some(stored)) = (&input.unit, &existing.unit) {
            let commensurable = Unit::parse(code)
                .ok()
                .zip(Unit::parse(stored).ok())
                .is_some_and(|(given, stored)| given.is_commensurable_with(&stored));
            if !commensurable {
                self.error(
                    format!("{path}/unit"),
                    "property_conflict",
                    format!(
                        "the property {:?} measures in units like {stored}",
                        existing.key
                    ),
                );
            }
        }
        if !input.categories.is_empty() && input.categories != existing.categories {
            self.error(
                format!("{path}/categories"),
                "property_conflict",
                format!(
                    "the property {:?} has the categories {:?}",
                    existing.key, existing.categories
                ),
            );
        }
        if input
            .label
            .as_ref()
            .is_some_and(|label| *label != existing.label)
        {
            self.warning(
                format!("{path}/label"),
                "label_differs",
                format!("arity keeps the stored label {:?}", existing.label),
            );
        }
    }

    fn new_property(&mut self, path: &str, input: &PropertyInput) -> Option<PropertyDefinition> {
        let label = input.label.as_deref().map_or_default(str::trim);
        if label.is_empty() {
            self.error(
                format!("{path}/label"),
                "missing_definition",
                format!("the property {:?} is new; give a label", input.id),
            );
        }
        let Some(value_type) = input.value_type else {
            self.error(
                format!("{path}/value_type"),
                "missing_definition",
                format!("the property {:?} is new; give a value type", input.id),
            );
            return None;
        };
        match value_type {
            ValueType::Quantity => match input.unit.as_deref().map(Unit::parse) {
                None => self.error(
                    format!("{path}/unit"),
                    "missing_definition",
                    "a quantity property needs a unit that sets its dimension",
                ),
                Some(Err(error)) => {
                    self.error(format!("{path}/unit"), "invalid_unit", error.to_string())
                }
                Some(Ok(_)) => {}
            },
            ValueType::Category => {
                let unique: HashSet<&String> = input.categories.iter().collect();
                if input.categories.is_empty() || unique.len() != input.categories.len() {
                    self.error(
                        format!("{path}/categories"),
                        "missing_definition",
                        "a category property needs a list of different categories",
                    );
                }
            }
            ValueType::Boolean => {}
        }
        if value_type != ValueType::Quantity && input.unit.is_some() {
            self.error(
                format!("{path}/unit"),
                "unexpected_field",
                "only a quantity property has a unit",
            );
        }
        (!label.is_empty()).then(|| PropertyDefinition {
            key: input.id.clone(),
            label: label.to_owned(),
            value_type,
            unit: input.unit.clone(),
            categories: input.categories.clone(),
            xrefs: input.xrefs.clone(),
            description: input.description.clone(),
        })
    }

    fn resolve_property(
        &mut self,
        path: &str,
        key: &str,
    ) -> Option<(PropertyDefinition, Option<Unit>)> {
        if let Some(resolved) = self.properties.get(key) {
            return Some((resolved.definition.clone(), resolved.unit.clone()));
        }
        if let Some(definition) = self.registry.property(key) {
            let unit = definition
                .unit
                .as_deref()
                .and_then(|code| Unit::parse(code).ok());
            self.properties.insert(
                key.to_owned(),
                Resolved {
                    definition: definition.clone(),
                    unit: unit.clone(),
                },
            );
            return Some((definition, unit));
        }
        self.error(
            path,
            "unknown_property",
            format!("the property {key:?} is not in the database; define it in /properties"),
        );
        None
    }

    fn entity(&mut self, path: &str, entity: &crate::submission::EntityInput) {
        if entity.id.trim().is_empty() || entity.id.contains(char::is_whitespace) {
            self.error(
                format!("{path}/id"),
                "invalid_id",
                "an entity id is not empty and has no spaces",
            );
        } else if self
            .entities
            .insert(entity.id.clone(), entity.kind)
            .is_some()
        {
            self.error(
                format!("{path}/id"),
                "duplicate_id",
                format!("more than one entity has the id {:?}", entity.id),
            );
        }
        if entity.label.trim().is_empty() {
            self.error(
                format!("{path}/label"),
                "empty_label",
                "give the name of the entity",
            );
        }
        for (index, xref) in entity.xrefs.iter().enumerate() {
            if !xref.fits_entity_kind(entity.kind) {
                self.error(
                    format!("{path}/xrefs/{index}"),
                    "xref_kind_mismatch",
                    format!(
                        "{} ({}) does not identify an entity of the kind {}",
                        xref,
                        xref.prefix().title,
                        entity.kind
                    ),
                );
            }
        }
        if entity.xrefs.is_empty() && entity.kind != EntityKind::Other {
            self.warning(
                format!("{path}/xrefs"),
                "no_xref",
                "the entity has no registry identifier, so arity cannot link it to other works",
            );
        }
        if entity.mentions.is_empty() {
            self.error(
                format!("{path}/mentions"),
                "no_mention",
                "quote at least one place where the source names the entity",
            );
        }
        for (index, mention) in entity.mentions.iter().enumerate() {
            self.evidence(&format!("{path}/mentions/{index}"), mention);
        }
    }

    fn evidence(&mut self, path: &str, evidence: &EvidenceInput) -> Option<String> {
        let Some(block) = self.document.block(&evidence.block) else {
            self.error(
                format!("{path}/block"),
                "unknown_block",
                format!("the text has no block {:?}", evidence.block),
            );
            return None;
        };
        let length = normalize(&evidence.quote).as_str().chars().count();
        if length == 0 {
            self.error(format!("{path}/quote"), "empty_quote", "the quote is empty");
            return None;
        }
        if length > MAX_QUOTE {
            self.error(
                format!("{path}/quote"),
                "quote_too_long",
                format!("a quote has at most {MAX_QUOTE} characters; quote only the part that supports the report"),
            );
            return None;
        }
        if length > LONG_QUOTE {
            self.warning(
                format!("{path}/quote"),
                "long_quote",
                format!("the quote has more than {LONG_QUOTE} characters"),
            );
        }
        let matches = normalize(&block.text).find(&evidence.quote);
        if matches.is_empty() {
            self.error(
                format!("{path}/quote"),
                "quote_not_found",
                format!("the quote does not occur in the block {:?}", evidence.block),
            );
            return None;
        }
        let occurrence = evidence.occurrence.unwrap_or(1);
        let Some(range) = usize::try_from(occurrence)
            .ok()
            .and_then(|occurrence| occurrence.checked_sub(1))
            .and_then(|index| matches.get(index))
        else {
            self.error(
                format!("{path}/occurrence"),
                "occurrence_out_of_range",
                format!("the quote occurs {} times in the block", matches.len()),
            );
            return None;
        };
        if matches.len() > 1 && evidence.occurrence.is_none() {
            self.warning(
                format!("{path}/quote"),
                "ambiguous_quote",
                format!(
                    "the quote occurs {} times in the block; arity uses the first, so give `occurrence` to choose another",
                    matches.len()
                ),
            );
        }
        self.positions.insert(path.to_owned(), range.clone());
        Some(evidence.quote.clone())
    }

    fn report_evidence(&mut self, path: &str, items: &[EvidenceInput]) -> Evidence {
        if items.is_empty() {
            self.error(
                format!("{path}/evidence"),
                "no_evidence",
                "a report needs at least one quote from the source",
            );
        }
        let mut evidence = Evidence {
            quotes: Vec::new(),
            blocks: Vec::new(),
        };
        for (index, item) in items.iter().enumerate() {
            if let Some(quote) = self.evidence(&format!("{path}/evidence/{index}"), item) {
                evidence.quotes.push(quote);
                evidence.blocks.push(item.block.clone());
            }
        }
        evidence
    }

    fn known_entity(&mut self, path: &str, id: &str) {
        if !self.entities.contains_key(id) {
            self.error(
                path,
                "unknown_entity",
                format!("no entity in /entities has the id {id:?}"),
            );
        }
    }

    fn value_report(&mut self, path: &str, report: &ValueReportInput) {
        let evidence = self.report_evidence(path, &report.evidence);
        if report.subject.is_empty() {
            self.error(
                format!("{path}/subject"),
                "no_subject",
                "name at least one entity",
            );
        }
        for (index, subject) in report.subject.iter().enumerate() {
            self.known_entity(&format!("{path}/subject/{index}"), subject);
        }
        if let Some((definition, unit)) =
            self.resolve_property(&format!("{path}/property"), &report.property)
        {
            self.value(
                &format!("{path}/value"),
                &report.value,
                &definition,
                unit.as_ref(),
                &evidence,
            );
        }
        self.conditions(path, &report.conditions, &evidence);
        self.method(path, report.method.as_ref(), &evidence);
    }

    fn effect_report(&mut self, path: &str, report: &EffectReportInput) {
        let evidence = self.report_evidence(path, &report.evidence);
        if report.agents.is_empty() {
            self.error(
                format!("{path}/agents"),
                "no_agents",
                "name at least one agent",
            );
        }
        for (index, agent) in report.agents.iter().enumerate() {
            let agent_path = format!("{path}/agents/{index}");
            self.known_entity(&format!("{agent_path}/entity"), &agent.entity);
            if let Some(dose) = &agent.dose {
                self.quantity(&format!("{agent_path}/dose"), dose, None, &evidence);
            }
        }
        self.known_entity(&format!("{path}/target/entity"), &report.target.entity);
        if let Some(property) = &report.target.property {
            self.resolve_property(&format!("{path}/target/property"), property);
        }
        if !cues::states_direction(report.direction, &evidence.quotes) {
            self.warning(
                format!("{path}/direction"),
                "direction_cue_missing",
                format!(
                    "no quote has a word that states the direction {}; a person must check it",
                    report.direction
                ),
            );
        }
        if let Some(magnitude) = &report.magnitude {
            self.magnitude(&format!("{path}/magnitude"), magnitude, &evidence);
        }
        if let Some(comparator) = &report.comparator
            && !evidence.has_text(comparator, true)
        {
            self.error(
                format!("{path}/comparator"),
                "text_not_in_evidence",
                format!("no quote contains the comparator {comparator:?}"),
            );
        }
        if let Some(significance) = &report.significance {
            let significance_path = format!("{path}/significance");
            if significance.statement.is_none() && significance.p.is_none() {
                self.error(
                    &significance_path,
                    "significance_shape",
                    "give a statement, a p-value, or both",
                );
            }
            if let Some(p) = significance.p {
                if p < Decimal::ZERO || p > Decimal::new(1, 0) {
                    self.error(
                        format!("{significance_path}/p"),
                        "invalid_p",
                        "a p-value is from 0 to 1",
                    );
                }
                self.number_in_evidence(&format!("{significance_path}/p"), p, &evidence);
            }
            if significance.comparison.is_some() && significance.p.is_none() {
                self.error(
                    format!("{significance_path}/comparison"),
                    "significance_shape",
                    "a comparison needs a p-value",
                );
            }
        }
        if let Some(n) = report.n {
            self.sample_size(&format!("{path}/n"), n, &evidence);
        }
        self.conditions(path, &report.conditions, &evidence);
        self.method(path, report.method.as_ref(), &evidence);
    }

    fn magnitude(&mut self, path: &str, magnitude: &MagnitudeInput, evidence: &Evidence) {
        self.number_in_evidence(&format!("{path}/number"), magnitude.number, evidence);
        if let Some(code) = &magnitude.unit {
            match Unit::parse(code) {
                Ok(unit) => self.unit_text(path, &unit, magnitude.unit_text.as_deref(), evidence),
                Err(error) => self.error(format!("{path}/unit"), "invalid_unit", error.to_string()),
            }
        }
        if let Some(interval) = &magnitude.interval {
            self.uncertainty(&format!("{path}/interval"), interval, evidence, true);
        }
    }

    fn conditions(&mut self, path: &str, conditions: &[ConditionInput], evidence: &Evidence) {
        for (index, condition) in conditions.iter().enumerate() {
            self.condition(&format!("{path}/conditions/{index}"), condition, evidence);
        }
    }

    fn condition(&mut self, path: &str, condition: &ConditionInput, evidence: &Evidence) {
        match condition {
            ConditionInput {
                role: Some(_),
                entity: Some(entity),
                text: None,
                property: None,
                value: None,
            } => self.known_entity(&format!("{path}/entity"), entity),
            ConditionInput {
                role: Some(_),
                entity: None,
                text: Some(text),
                property: None,
                value: None,
            } => {
                if !evidence.has_text(text, true) {
                    self.error(
                        format!("{path}/text"),
                        "text_not_in_evidence",
                        format!("no quote contains {text:?}"),
                    );
                }
            }
            ConditionInput {
                role: None,
                entity: None,
                text: None,
                property: Some(property),
                value: Some(value),
            } => {
                if let Some((definition, unit)) =
                    self.resolve_property(&format!("{path}/property"), property)
                {
                    self.value(
                        &format!("{path}/value"),
                        value,
                        &definition,
                        unit.as_ref(),
                        evidence,
                    );
                }
            }
            _ => self.error(
                path,
                "condition_shape",
                "give `role` with `entity` or `text`, or give `property` with `value`",
            ),
        }
    }

    fn method(&mut self, path: &str, method: Option<&MethodInput>, evidence: &Evidence) {
        let Some(method) = method else {
            return;
        };
        if method.label.trim().is_empty() {
            self.error(
                format!("{path}/method/label"),
                "empty_label",
                "give the name of the method",
            );
        } else if !evidence.has_text(&method.label, true) {
            self.warning(
                format!("{path}/method/label"),
                "method_not_in_evidence",
                "no quote names the method; add the quote that describes it",
            );
        }
    }

    fn value(
        &mut self,
        path: &str,
        value: &ValueInput,
        definition: &PropertyDefinition,
        unit: Option<&Unit>,
        evidence: &Evidence,
    ) {
        let shapes = [
            value.number.is_some(),
            value.low.is_some() || value.high.is_some(),
            value.boolean.is_some(),
            value.category.is_some(),
        ];
        if shapes.iter().filter(|shape| **shape).count() != 1 {
            self.error(
                path,
                "value_shape",
                "give exactly one of: `number`, `low` with `high`, `boolean`, or `category`",
            );
            return;
        }
        let quantity_fields = value.unit.is_some()
            || value.unit_text.is_some()
            || value.bound.is_some()
            || value.uncertainty.is_some()
            || value.statistic.is_some()
            || value.read_from_figure.is_some();
        match definition.value_type {
            ValueType::Quantity => {
                if value.boolean.is_some() || value.category.is_some() {
                    self.type_mismatch(path, definition);
                } else {
                    self.quantity(path, value, unit, evidence);
                }
            }
            ValueType::Boolean => {
                if value.boolean.is_none() {
                    self.type_mismatch(path, definition);
                } else if quantity_fields {
                    self.error(
                        path,
                        "unexpected_field",
                        "a boolean value has no unit, bound, or uncertainty",
                    );
                }
            }
            ValueType::Category => match &value.category {
                None => self.type_mismatch(path, definition),
                Some(_) if quantity_fields => {
                    self.error(
                        path,
                        "unexpected_field",
                        "a category value has no unit, bound, or uncertainty",
                    );
                }
                Some(category) => {
                    if !definition.categories.contains(category) {
                        self.error(
                            format!("{path}/category"),
                            "unknown_category",
                            format!(
                                "the property {:?} has the categories {:?}",
                                definition.key, definition.categories
                            ),
                        );
                    } else if !evidence.has_text(category, true) {
                        self.warning(
                            format!("{path}/category"),
                            "category_not_in_evidence",
                            format!("no quote contains {category:?}; a person must check it"),
                        );
                    }
                }
            },
        }
        if let Some(n) = value.n {
            self.sample_size(&format!("{path}/n"), n, evidence);
        }
    }

    fn type_mismatch(&mut self, path: &str, definition: &PropertyDefinition) {
        self.error(
            path,
            "value_type_mismatch",
            format!(
                "the property {:?} takes a value of the type {}",
                definition.key, definition.value_type
            ),
        );
    }

    fn quantity(
        &mut self,
        path: &str,
        value: &ValueInput,
        expected: Option<&Unit>,
        evidence: &Evidence,
    ) {
        let Some(code) = &value.unit else {
            self.error(
                format!("{path}/unit"),
                "unit_required",
                "a quantity needs a UCUM unit; use \"1\" for a pure number",
            );
            return;
        };
        let unit = match Unit::parse(code) {
            Ok(unit) => unit,
            Err(error) => {
                self.error(format!("{path}/unit"), "invalid_unit", error.to_string());
                return;
            }
        };
        if let Some(expected) = expected
            && !unit.is_commensurable_with(expected)
        {
            self.error(
                format!("{path}/unit"),
                "dimension_mismatch",
                format!(
                    "the unit {code} measures the dimension {}, but the property measures {}",
                    unit.dimension(),
                    expected.dimension()
                ),
            );
        }
        self.unit_text(path, &unit, value.unit_text.as_deref(), evidence);

        let stated = match &value.read_from_figure {
            Some(reading) => {
                self.figure_reading(&format!("{path}/read_from_figure"), reading, evidence);
                false
            }
            None => true,
        };
        if let Some(number) = value.number
            && stated
        {
            self.number_in_evidence(&format!("{path}/number"), number, evidence);
        }
        match (value.low, value.high) {
            (Some(low), Some(high)) => {
                if low > high {
                    self.error(path, "invalid_interval", "`low` is greater than `high`");
                }
                if stated {
                    self.number_in_evidence(&format!("{path}/low"), low, evidence);
                    self.number_in_evidence(&format!("{path}/high"), high, evidence);
                }
            }
            (None, None) => {}
            _ => self.error(path, "value_shape", "a range needs both `low` and `high`"),
        }
        if let Some(bound) = value.bound
            && !cues::states_bound(bound, &evidence.quotes)
        {
            self.warning(
                format!("{path}/bound"),
                "bound_cue_missing",
                format!(
                    "no quote has a sign or a word for the bound {bound}; a person must check it"
                ),
            );
        }
        if let Some(uncertainty) = &value.uncertainty {
            self.uncertainty(
                &format!("{path}/uncertainty"),
                uncertainty,
                evidence,
                stated,
            );
        }
    }

    fn unit_text(&mut self, path: &str, unit: &Unit, text: Option<&str>, evidence: &Evidence) {
        let Some(text) = text else {
            if !unit.dimension().is_none() || unit.is_arbitrary() {
                self.error(
                    format!("{path}/unit_text"),
                    "unit_text_required",
                    "give the unit exactly as the source writes it",
                );
            }
            return;
        };
        if !evidence.has_text(text, false) {
            self.error(
                format!("{path}/unit_text"),
                "unit_text_not_in_evidence",
                format!("no quote contains the unit text {text:?}"),
            );
        }
        match Unit::parse_reported(text) {
            Ok(reported) if !reported.is_equivalent_to(unit) => self.error(
                format!("{path}/unit_text"),
                "unit_text_mismatch",
                format!(
                    "the source writes {text:?}, which arity reads as {}, but the unit is {}",
                    reported.code(),
                    unit.code()
                ),
            ),
            Ok(_) => {}
            Err(_) => self.warning(
                format!("{path}/unit_text"),
                "unit_text_unreadable",
                format!("arity cannot read the unit text {text:?}; a person must check the unit"),
            ),
        }
    }

    fn uncertainty(
        &mut self,
        path: &str,
        uncertainty: &UncertaintyInput,
        evidence: &Evidence,
        stated: bool,
    ) {
        let interval_kind = matches!(
            uncertainty.kind,
            UncertaintyKind::ConfidenceInterval
                | UncertaintyKind::Range
                | UncertaintyKind::InterquartileRange
        );
        let has_bounds = uncertainty.low.is_some() && uncertainty.high.is_some();
        let has_half_width = uncertainty.number.is_some();
        let shape_ok = if interval_kind {
            has_bounds && !has_half_width
        } else {
            has_half_width != has_bounds && uncertainty.low.is_some() == uncertainty.high.is_some()
        };
        if !shape_ok {
            let message = if interval_kind {
                "an interval needs `low` and `high`"
            } else {
                "give `number`, or give `low` with `high`"
            };
            self.error(path, "uncertainty_shape", message);
            return;
        }
        if let Some(number) = uncertainty.number
            && number < Decimal::ZERO
        {
            self.error(
                format!("{path}/number"),
                "negative_uncertainty",
                "the uncertainty is less than zero",
            );
        }
        if let (Some(low), Some(high)) = (uncertainty.low, uncertainty.high)
            && low > high
        {
            self.error(path, "invalid_interval", "`low` is greater than `high`");
        }
        match (uncertainty.kind, uncertainty.level) {
            (UncertaintyKind::ConfidenceInterval, Some(level)) => {
                if level <= Decimal::ZERO || level >= Decimal::new(100, 0) {
                    self.error(
                        format!("{path}/level"),
                        "invalid_level",
                        "the level is a percent from 0 to 100",
                    );
                } else if stated {
                    self.number_in_evidence(&format!("{path}/level"), level, evidence);
                }
            }
            (_, Some(_)) => self.error(
                format!("{path}/level"),
                "unexpected_field",
                "only a confidence interval has a level",
            ),
            _ => {}
        }
        if stated {
            for (field, number) in [
                ("number", uncertainty.number),
                ("low", uncertainty.low),
                ("high", uncertainty.high),
            ] {
                if let Some(number) = number {
                    self.number_in_evidence(&format!("{path}/{field}"), number, evidence);
                }
            }
        }
    }

    fn figure_reading(&mut self, path: &str, reading: &FigureReadingInput, evidence: &Evidence) {
        let is_figure = self
            .document
            .block(&reading.figure)
            .is_some_and(|block| block.kind == BlockKind::FigureCaption);
        if !is_figure {
            self.error(
                format!("{path}/figure"),
                "figure_not_found",
                format!("the text has no figure caption {:?}", reading.figure),
            );
        } else if !evidence.blocks.contains(&reading.figure) {
            self.error(
                format!("{path}/figure"),
                "figure_not_in_evidence",
                "quote the caption of the figure in the evidence of the report",
            );
        }
        if reading.method.trim().is_empty() {
            self.error(
                format!("{path}/method"),
                "empty_method",
                "tell how the reader read the value",
            );
        }
        if reading.resolution <= Decimal::ZERO {
            self.error(
                format!("{path}/resolution"),
                "invalid_resolution",
                "the resolution is more than zero",
            );
        }
    }

    fn sample_size(&mut self, path: &str, n: u32, evidence: &Evidence) {
        if n == 0 {
            self.error(
                path,
                "invalid_sample_size",
                "the number of samples is at least 1",
            );
        } else {
            self.number_in_evidence(path, Decimal::from(n), evidence);
        }
    }

    fn number_in_evidence(&mut self, path: &str, number: Decimal, evidence: &Evidence) {
        if !evidence.has_number(number) {
            self.error(
                path,
                "number_not_in_evidence",
                format!("no quote of the report contains the number {number}"),
            );
        }
    }
}
