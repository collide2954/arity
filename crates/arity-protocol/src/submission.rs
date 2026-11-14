use arity_core::vocabulary::{
    AgentKind, Bound, Comparison, Direction, EffectMeasure, Relation, Role, SignificanceStatement,
    Statistic, UncertaintyKind, ValueType,
};
use arity_core::{Curie, Decimal, Digest, EntityKind, Orcid};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const SUBMISSION_FORMAT: &str = "arity-submission/1";

/// The reports that a reader takes from one source, with the evidence for
/// each report. arity checks each submission against the canonical text of
/// the source before it stores the reports.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Submission {
    /// Always "arity-submission/1".
    pub format: String,
    /// The work that the reader read: a PMCID, a DOI, or a PMID.
    pub work: String,
    /// The digest of the canonical text that `arity show` printed.
    pub text: Digest,
    /// The person or software that read the source.
    pub reader: Reader,
    /// The person who is responsible for the submission.
    pub responsible: Person,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entities: Vec<EntityInput>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub properties: Vec<PropertyInput>,
    pub reports: Vec<ReportInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Reader {
    pub kind: AgentKind,
    /// The name of the person, or the name of the software.
    pub name: String,
    /// The version of the software.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orcid: Option<Orcid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Person {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orcid: Option<Orcid>,
}

/// A place in the canonical text that supports a report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceInput {
    /// The key of a block, as `arity show` prints it, for example "t1/r3c5".
    pub block: String,
    /// The exact text from the block.
    pub quote: String,
    /// When the quote occurs more than once in the block: the occurrence to
    /// use. 1 is the first.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occurrence: Option<u32>,
}

/// A thing that the reports are about.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EntityInput {
    /// An identifier for the entity in this submission, for example "dox".
    pub id: String,
    pub kind: EntityKind,
    /// The name of the entity.
    pub label: String,
    /// Identifiers in public registries. arity links entities across works
    /// through these identifiers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub xrefs: Vec<Curie>,
    /// Places where the source names the entity.
    pub mentions: Vec<EvidenceInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// A property that values report, for example the half maximal inhibitory
/// concentration. arity keeps one definition for each key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PropertyInput {
    /// The key: lower-case letters, digits, and underscores, for example
    /// "ic50".
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_type: Option<ValueType>,
    /// For a quantity: a UCUM unit that sets the dimension. arity shows the
    /// values in this unit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// For a category: the values that the property can take.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub xrefs: Vec<Curie>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// One report: a value or an effect that the source states.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ReportInput {
    Value(ValueReportInput),
    Effect(EffectReportInput),
}

/// The value of a property of one entity, or of a combination of entities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ValueReportInput {
    pub id: String,
    /// The entity, or the entities of a combination.
    pub subject: Vec<String>,
    /// The key of the property.
    pub property: String,
    pub value: ValueInput,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conditions: Vec<ConditionInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<MethodInput>,
    pub evidence: Vec<EvidenceInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// A value as the source states it. Give one of: `number`, `low` with
/// `high`, `boolean`, or `category`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ValueInput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<Decimal>,
    /// The lower end of a value that the source states as a range.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub low: Option<Decimal>,
    /// The upper end of a value that the source states as a range.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub high: Option<Decimal>,
    /// The UCUM code of the unit, for example "umol/L".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// The unit exactly as the source writes it, for example "μM".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bound: Option<Bound>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uncertainty: Option<UncertaintyInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub statistic: Option<Statistic>,
    /// The number of samples, as the source states it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boolean: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Give this only when the reader reads the value from a plot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_from_figure: Option<FigureReadingInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UncertaintyInput {
    pub kind: UncertaintyKind,
    /// The half width, for example 0.4 in "3.2 ± 0.4".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<Decimal>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub low: Option<Decimal>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub high: Option<Decimal>,
    /// For a confidence interval: the level in percent, for example 95.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level: Option<Decimal>,
}

/// How the reader read a value from a plot. The value is the reader's
/// measurement, and the website shows it with a "read from figure" label.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FigureReadingInput {
    /// The key of the caption block of the figure, for example "f3/caption".
    pub figure: String,
    /// How the reader read the value, for example "visual reading against
    /// the axis grid".
    pub method: String,
    /// The smallest difference that the reading can show, in the unit of
    /// the value.
    pub resolution: Decimal,
}

/// A condition of a report. Give `role` with `entity` or `text`, or give
/// `property` with `value`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConditionInput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity: Option<String>,
    /// A condition that no entity describes, exactly as the source writes it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub property: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<ValueInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MethodInput {
    /// The name of the method, for example "MTT assay".
    pub label: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub xrefs: Vec<Curie>,
}

/// An effect of one or more agents on a target, or an association between
/// them, as the source states it. A statement of no effect has the
/// direction "`no_change`".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EffectReportInput {
    pub id: String,
    pub agents: Vec<AgentInput>,
    pub target: TargetInput,
    pub direction: Direction,
    #[serde(default = "effect", skip_serializing_if = "is_effect")]
    pub relation: Relation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub magnitude: Option<MagnitudeInput>,
    /// What the source compares with, exactly as it writes it, for example
    /// "untreated cells".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comparator: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub significance: Option<SignificanceInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conditions: Vec<ConditionInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<MethodInput>,
    pub evidence: Vec<EvidenceInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

fn effect() -> Relation {
    Relation::Effect
}

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde passes the field by reference"
)]
fn is_effect(relation: &Relation) -> bool {
    *relation == Relation::Effect
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AgentInput {
    pub entity: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dose: Option<ValueInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TargetInput {
    pub entity: String,
    /// The key of the property of the target that changes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub property: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MagnitudeInput {
    pub measure: EffectMeasure,
    pub number: Decimal,
    /// The UCUM code of the unit, for a measure that has a unit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interval: Option<UncertaintyInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SignificanceInput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub statement: Option<SignificanceStatement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comparison: Option<Comparison>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p: Option<Decimal>,
    /// The name of the statistical test, as the source writes it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test: Option<String>,
}

/// The JSON Schema of the submission format.
pub fn schema() -> serde_json::Value {
    serde_json::to_value(schemars::schema_for!(Submission)).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCHEMA_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../schemas/submission.schema.json"
    );

    #[test]
    fn the_committed_schema_is_current() {
        let current = format!("{}\n", serde_json::to_string_pretty(&schema()).unwrap());
        if std::env::var_os("ARITY_UPDATE_SCHEMAS").is_some() {
            std::fs::write(SCHEMA_PATH, &current).unwrap();
        }
        let committed = std::fs::read_to_string(SCHEMA_PATH)
            .unwrap_or_default()
            .replace("\r\n", "\n");
        assert!(
            committed == current,
            "schemas/submission.schema.json is out of date; run the tests with ARITY_UPDATE_SCHEMAS=1"
        );
    }

    #[test]
    fn rejects_unknown_fields() {
        let json = serde_json::json!({
            "entity": "dox",
            "confidence": 0.9
        });
        let error = serde_json::from_value::<AgentInput>(json).unwrap_err();
        assert!(error.to_string().contains("unknown field `confidence`"));
    }

    #[test]
    fn effects_default_to_the_effect_relation() {
        let json = serde_json::json!({
            "type": "effect",
            "id": "e1",
            "agents": [{ "entity": "a" }],
            "target": { "entity": "b" },
            "direction": "no_change",
            "evidence": [{ "block": "s1/p1", "quote": "no effect" }]
        });
        let ReportInput::Effect(report) = serde_json::from_value(json.clone()).unwrap() else {
            panic!("not an effect");
        };
        assert_eq!(report.relation, Relation::Effect);
        assert_eq!(report.direction, Direction::NoChange);
        assert_eq!(
            serde_json::to_value(ReportInput::Effect(report)).unwrap(),
            json
        );
    }
}
