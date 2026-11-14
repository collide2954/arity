#![cfg(test)]

use std::collections::HashMap;

use arity_core::Digest;
use arity_core::vocabulary::ValueType;
use arity_protocol::{Checked, PropertyDefinition, Rejected, Severity, check};
use arity_text::{Block, BlockKind, Document};
use serde_json::{Value, json};

fn document() -> Document {
    let blocks = vec![
        Block::new(
            "abstract/p1",
            BlockKind::Abstract,
            "The results showed a reduction in the IC50 values of DOX in MCF-7 cells (from 0.24 μM to 0.012 μM) when treated with Dis (0.03 μM).",
        ),
        Block::new(
            "s2/p1",
            BlockKind::Paragraph,
            "MCF-7 cells were treated for 48 h. Values are mean ± SD of three experiments (n = 3).",
        ),
        Block::new(
            "s3/p1",
            BlockKind::Paragraph,
            "Hydralazine alone had no effect on cell viability (p = 0.34) compared with untreated cells.",
        ),
        Block::new("t1/r1c2", BlockKind::TableCell, "IC50 (μM)"),
        Block::new("t1/r2c1", BlockKind::TableCell, "Doxorubicin"),
        Block::new("t1/r2c2", BlockKind::TableCell, "0.24 ± 0.02"),
        Block::new(
            "f1/caption",
            BlockKind::FigureCaption,
            "Viability of MCF-7 cells after 48 h of treatment.",
        ),
        Block::new(
            "t1/caption",
            BlockKind::TableCaption,
            "Table 1: IC50 values.",
        ),
        Block::new(
            "t2/caption",
            BlockKind::TableCaption,
            "Table 2: IC50 values of the compounds in MCF-7 cells.",
        ),
    ];
    Document::new(Digest::of(b"source"), "test 1", blocks).unwrap()
}

fn submission() -> Value {
    json!({
        "format": "arity-submission/1",
        "work": "PMC1",
        "text": document().digest().unwrap(),
        "reader": { "kind": "software", "name": "reader", "version": "1" },
        "responsible": { "name": "A Person" },
        "entities": [
            {
                "id": "dox", "kind": "chemical", "label": "doxorubicin",
                "xrefs": ["CHEBI:28748"],
                "mentions": [{ "block": "t1/r2c1", "quote": "Doxorubicin" }]
            },
            {
                "id": "mcf7", "kind": "cell_line", "label": "MCF-7",
                "xrefs": ["CELLOSAURUS:CVCL_0031"],
                "mentions": [{ "block": "s2/p1", "quote": "MCF-7" }]
            },
            {
                "id": "hyd", "kind": "chemical", "label": "hydralazine",
                "xrefs": ["CHEBI:5775"],
                "mentions": [{ "block": "s3/p1", "quote": "Hydralazine" }]
            }
        ],
        "properties": [
            {
                "id": "ic50", "label": "half maximal inhibitory concentration",
                "value_type": "quantity", "unit": "umol/L"
            },
            { "id": "exposure_time", "label": "exposure time", "value_type": "quantity", "unit": "h" },
            { "id": "cell_viability", "label": "cell viability", "value_type": "quantity", "unit": "%" }
        ],
        "reports": [
            {
                "type": "value", "id": "r1", "subject": ["dox"], "property": "ic50",
                "value": {
                    "number": "0.24", "unit": "umol/L", "unit_text": "μM",
                    "uncertainty": { "kind": "sd", "number": "0.02" },
                    "statistic": "mean", "n": 3
                },
                "conditions": [
                    { "role": "cell_line", "entity": "mcf7" },
                    {
                        "property": "exposure_time",
                        "value": { "number": "48", "unit": "h", "unit_text": "h" }
                    }
                ],
                "evidence": [
                    { "block": "t1/r2c2", "quote": "0.24 ± 0.02" },
                    { "block": "t1/r1c2", "quote": "IC50 (μM)" },
                    {
                        "block": "s2/p1",
                        "quote": "treated for 48 h. Values are mean ± SD of three experiments (n = 3)"
                    }
                ]
            },
            {
                "type": "effect", "id": "e1",
                "agents": [{ "entity": "hyd" }],
                "target": { "entity": "mcf7", "property": "cell_viability" },
                "direction": "no_change",
                "comparator": "untreated cells",
                "significance": { "comparison": "eq", "p": "0.34" },
                "evidence": [{
                    "block": "s3/p1",
                    "quote": "Hydralazine alone had no effect on cell viability (p = 0.34) compared with untreated cells"
                }]
            }
        ]
    })
}

fn run_with(
    value: &Value,
    registry: &HashMap<String, PropertyDefinition>,
) -> Result<Checked, Rejected> {
    check(&serde_json::to_vec(value).unwrap(), &document(), registry)
}

fn run(value: &Value) -> Result<Checked, Rejected> {
    run_with(value, &HashMap::new())
}

fn changed(edit: impl FnOnce(&mut Value)) -> Value {
    let mut value = submission();
    edit(&mut value);
    value
}

fn errors(value: &Value) -> Vec<(String, &'static str)> {
    match run(value) {
        Ok(_) => Vec::new(),
        Err(rejected) => rejected
            .errors()
            .map(|issue| (issue.path.clone(), issue.code.as_str()))
            .collect(),
    }
}

fn warnings(value: &Value) -> Vec<(String, &'static str)> {
    run(value)
        .unwrap_or_else(|rejected| panic!("{:#?}", rejected.issues))
        .warnings
        .into_iter()
        .map(|issue| (issue.path, issue.code.as_str()))
        .collect()
}

fn one(path: &str, code: &'static str) -> Vec<(String, &'static str)> {
    vec![(path.to_owned(), code)]
}

#[test]
fn accepts_a_correct_submission() {
    let checked = run(&submission()).unwrap_or_else(|rejected| panic!("{:#?}", rejected.issues));
    assert!(checked.warnings.is_empty(), "{:#?}", checked.warnings);
    assert_eq!(checked.positions["/reports/0/evidence/0"], 0..11);
    assert_eq!(checked.positions["/entities/1/mentions/0"], 0..5);
    let keys: Vec<&str> = checked
        .new_properties
        .iter()
        .map(|definition| definition.key.as_str())
        .collect();
    assert_eq!(keys, ["ic50", "exposure_time", "cell_viability"]);
    assert_eq!(
        checked.digest,
        Digest::of(&serde_json::to_vec(&submission()).unwrap())
    );
}

#[test]
fn rejects_a_file_that_is_not_a_submission() {
    let result = check(b"{ not json", &document(), &HashMap::new());
    assert_eq!(result.unwrap_err().issues[0].code.as_str(), "invalid_json");
    let unknown_field = changed(|v| v["confidence"] = json!(0.9));
    assert_eq!(errors(&unknown_field), one("", "invalid_json"));
}

#[test]
fn rejects_a_wrong_format_or_text() {
    assert_eq!(
        errors(&changed(|v| v["format"] = json!("arity-submission/0"))),
        one("/format", "unknown_format")
    );
    assert_eq!(
        errors(&changed(|v| v["text"] = json!(Digest::of(b"other")))),
        one("/text", "text_mismatch")
    );
}

#[test]
fn rejects_quotes_that_do_not_occur() {
    assert_eq!(
        errors(&changed(
            |v| v["reports"][0]["evidence"][0]["quote"] = json!("0.24 ± 0.03")
        )),
        [
            ("/reports/0/evidence/0/quote".to_owned(), "quote_not_found"),
            (
                "/reports/0/value/number".to_owned(),
                "number_not_in_evidence"
            ),
            (
                "/reports/0/value/uncertainty/number".to_owned(),
                "number_not_in_evidence"
            ),
        ]
    );
    assert_eq!(
        errors(&changed(
            |v| v["entities"][0]["mentions"][0]["block"] = json!("t9/r1c1")
        )),
        one("/entities/0/mentions/0/block", "unknown_block")
    );
}

#[test]
fn rejects_numbers_that_no_quote_contains() {
    assert_eq!(
        errors(&changed(
            |v| v["reports"][0]["value"]["number"] = json!("0.25")
        )),
        one("/reports/0/value/number", "number_not_in_evidence")
    );
    assert_eq!(
        errors(&changed(
            |v| v["reports"][0]["value"]["number"] = json!("-0.24")
        )),
        one("/reports/0/value/number", "number_not_in_evidence")
    );
    assert_eq!(
        errors(&changed(|v| v["reports"][0]["value"]["n"] = json!(4))),
        one("/reports/0/value/n", "number_not_in_evidence")
    );
    assert_eq!(
        errors(&changed(
            |v| v["reports"][1]["significance"]["p"] = json!("0.05")
        )),
        one("/reports/1/significance/p", "number_not_in_evidence")
    );
}

#[test]
fn checks_units_against_the_property_and_the_source() {
    assert_eq!(
        errors(&changed(|v| v["reports"][0]["value"]["unit"] = json!("mg"))),
        [
            ("/reports/0/value/unit".to_owned(), "dimension_mismatch"),
            (
                "/reports/0/value/unit_text".to_owned(),
                "unit_text_mismatch"
            ),
        ]
    );
    assert_eq!(
        errors(&changed(
            |v| v["reports"][0]["value"]["unit"] = json!("mmol/L")
        )),
        one("/reports/0/value/unit_text", "unit_text_mismatch")
    );
    assert_eq!(
        errors(&changed(
            |v| v["reports"][0]["value"]["unit"] = json!("furlong")
        )),
        one("/reports/0/value/unit", "invalid_unit")
    );
    let missing = changed(|v| {
        v["reports"][0]["value"]
            .as_object_mut()
            .unwrap()
            .remove("unit_text");
    });
    assert_eq!(
        errors(&missing),
        one("/reports/0/value/unit_text", "unit_text_required")
    );
    assert_eq!(
        errors(&changed(
            |v| v["reports"][0]["value"]["unit_text"] = json!("nM")
        )),
        [
            (
                "/reports/0/value/unit_text".to_owned(),
                "unit_text_not_in_evidence"
            ),
            (
                "/reports/0/value/unit_text".to_owned(),
                "unit_text_mismatch"
            ),
        ]
    );
}

#[test]
fn checks_entities() {
    assert_eq!(
        errors(&changed(
            |v| v["entities"][0]["xrefs"] = json!(["NCBITaxon:9606"])
        )),
        one("/entities/0/xrefs/0", "xref_kind_mismatch")
    );
    assert_eq!(
        errors(&changed(|v| v["entities"][0]["mentions"] = json!([]))),
        one("/entities/0/mentions", "no_mention")
    );
    assert_eq!(
        errors(&changed(
            |v| v["reports"][0]["subject"] = json!(["cisplatin"])
        )),
        one("/reports/0/subject/0", "unknown_entity")
    );
    let duplicate = changed(|v| {
        let copy = v["entities"][0].clone();
        v["entities"].as_array_mut().unwrap().push(copy);
    });
    assert_eq!(errors(&duplicate), one("/entities/3/id", "duplicate_id"));
    assert_eq!(
        warnings(&changed(|v| v["entities"][0]["xrefs"] = json!([]))),
        one("/entities/0/xrefs", "no_xref")
    );
}

#[test]
fn checks_properties_against_the_database() {
    let registry: HashMap<String, PropertyDefinition> = [(
        "ic50".to_owned(),
        PropertyDefinition {
            key: "ic50".to_owned(),
            label: "half maximal inhibitory concentration".to_owned(),
            value_type: ValueType::Quantity,
            unit: Some("mol/L".to_owned()),
            categories: Vec::new(),
            xrefs: Vec::new(),
            description: None,
        },
    )]
    .into_iter()
    .collect();
    let checked = run_with(&submission(), &registry).unwrap();
    assert!(checked.new_properties.iter().all(|p| p.key != "ic50"));

    let conflict = changed(|v| v["properties"][0]["unit"] = json!("g"));
    let codes: Vec<&str> = run_with(&conflict, &registry)
        .unwrap_err()
        .errors()
        .map(|issue| issue.code.as_str())
        .collect();
    assert_eq!(codes, ["property_conflict"]);

    let undeclared = changed(|v| {
        v["properties"].as_array_mut().unwrap().remove(0);
    });
    assert_eq!(
        errors(&undeclared),
        one("/reports/0/property", "unknown_property")
    );
    assert!(run_with(&undeclared, &registry).is_ok());

    assert_eq!(
        errors(&changed(|v| v["properties"][0]["id"] = json!("IC50"))),
        [
            ("/properties/0/id".to_owned(), "invalid_key"),
            ("/reports/0/property".to_owned(), "unknown_property"),
        ]
    );
}

#[test]
fn chooses_among_repeated_quotes() {
    let mention = |occurrence: Option<u32>| {
        changed(|v| {
            v["entities"][0]["mentions"]
                .as_array_mut()
                .unwrap()
                .push(json!({ "block": "abstract/p1", "quote": "μM", "occurrence": occurrence }));
        })
    };
    assert_eq!(
        warnings(&mention(None)),
        one("/entities/0/mentions/1/quote", "ambiguous_quote")
    );
    let second = run(&mention(Some(2))).unwrap();
    assert_eq!(second.warnings, []);
    assert_eq!(second.positions["/entities/0/mentions/1"], 95..97);
    assert_eq!(
        errors(&mention(Some(4))),
        one(
            "/entities/0/mentions/1/occurrence",
            "occurrence_out_of_range"
        )
    );
}

#[test]
fn accepts_values_read_from_a_figure_with_their_evidence() {
    let reading = |figure: &str, evidence_block: &str| {
        let figure = figure.to_owned();
        let evidence_block = evidence_block.to_owned();
        changed(move |v| {
            v["reports"][1] = json!({
                "type": "value", "id": "r2", "subject": ["mcf7"], "property": "cell_viability",
                "value": {
                    "number": "62", "unit": "%", "unit_text": "%",
                    "read_from_figure": {
                        "figure": figure, "method": "visual reading against the axis grid",
                        "resolution": "5"
                    }
                },
                "evidence": [
                    { "block": evidence_block, "quote": "Viability of MCF-7 cells" },
                    { "block": "abstract/p1", "quote": "IC50 values of DOX in MCF-7 cells (from 0.24 μM" }
                ]
            });
        })
    };
    let accepted = reading("f1/caption", "f1/caption");
    assert_eq!(errors(&accepted), Vec::<(String, &str)>::new());
    assert!(warnings(&accepted).contains(&(
        "/reports/1/value/unit_text".to_owned(),
        "unit_text_from_image"
    )));
    let with_percent = {
        let mut value = accepted;
        value["reports"][1]["value"]["unit"] = json!("1");
        value["reports"][1]["value"]
            .as_object_mut()
            .unwrap()
            .remove("unit_text");
        value["properties"][2]["unit"] = json!("1");
        value
    };
    assert!(run(&with_percent).is_ok());
    assert_eq!(
        errors(&reading("t1/r2c2", "f1/caption")),
        one(
            "/reports/1/value/read_from_figure/figure",
            "figure_not_found"
        )
    );
}

#[test]
fn warns_when_no_quote_states_the_direction() {
    let increase = changed(|v| v["reports"][1]["direction"] = json!("increase"));
    assert_eq!(
        warnings(&increase),
        one("/reports/1/direction", "direction_cue_missing")
    );
}

#[test]
fn checks_conditions_and_comparators() {
    assert_eq!(
        errors(&changed(
            |v| v["reports"][0]["conditions"][0]["text"] = json!("x")
        )),
        one("/reports/0/conditions/0", "condition_shape")
    );
    assert_eq!(
        errors(&changed(
            |v| v["reports"][1]["comparator"] = json!("vehicle")
        )),
        one("/reports/1/comparator", "text_not_in_evidence")
    );
    assert_eq!(
        errors(&changed(
            |v| v["reports"][0]["conditions"][1]["value"]["number"] = json!("24")
        )),
        one(
            "/reports/0/conditions/1/value/number",
            "number_not_in_evidence"
        )
    );
}

#[test]
fn checks_uncertainty_and_statistics() {
    assert_eq!(
        errors(&changed(
            |v| v["reports"][0]["value"]["uncertainty"] = json!({ "kind": "ci", "number": "0.02" })
        )),
        one("/reports/0/value/uncertainty", "uncertainty_shape")
    );
    assert_eq!(
        errors(&changed(
            |v| v["reports"][0]["value"]["uncertainty"]["number"] = json!("-0.02")
        )),
        [
            (
                "/reports/0/value/uncertainty/number".to_owned(),
                "negative_uncertainty"
            ),
            (
                "/reports/0/value/uncertainty/number".to_owned(),
                "number_not_in_evidence"
            ),
        ]
    );
    assert_eq!(
        errors(&changed(
            |v| v["reports"][1]["significance"]["p"] = json!("1.5")
        )),
        [
            ("/reports/1/significance/p".to_owned(), "invalid_p"),
            (
                "/reports/1/significance/p".to_owned(),
                "number_not_in_evidence"
            ),
        ]
    );
    assert_eq!(
        errors(&changed(|v| v["reports"][0]["value"]["n"] = json!(0))),
        one("/reports/0/value/n", "invalid_sample_size")
    );
}

#[test]
fn rejects_empty_and_duplicate_reports() {
    assert_eq!(
        errors(&changed(|v| v["reports"] = json!([]))),
        one("/reports", "no_reports")
    );
    assert_eq!(
        errors(&changed(|v| v["reports"][1]["id"] = json!("r1"))),
        one("/reports/1/id", "duplicate_id")
    );
    assert_eq!(
        errors(&changed(|v| v["reports"][1]["evidence"] = json!([]))),
        [
            ("/reports/1/evidence".to_owned(), "no_evidence"),
            ("/reports/1/comparator".to_owned(), "text_not_in_evidence"),
            (
                "/reports/1/significance/p".to_owned(),
                "number_not_in_evidence"
            ),
        ]
    );
}

#[test]
fn issues_have_a_severity_and_a_message() {
    let rejected = run(&changed(|v| {
        v["reports"][0]["value"]["number"] = json!("0.25");
    }))
    .unwrap_err();
    let issue = rejected.errors().next().unwrap();
    assert_eq!(issue.severity, Severity::Error);
    assert_eq!(
        issue.to_string(),
        "error at /reports/0/value/number: no quote of the report contains the number 0.25 [number_not_in_evidence]"
    );
    assert_eq!(rejected.to_string(), "the submission has 1 errors");
}

#[test]
fn accepts_a_skipped_quote_with_a_number_and_records_its_place() {
    let value = changed(|value| {
        value["skipped"] = json!([
            { "block": "abstract/p1", "quote": "from 0.24 μM to 0.012 μM", "reason": "cited" }
        ]);
    });
    let checked = run(&value).unwrap();
    assert!(checked.positions.contains_key("/skipped/0"));
}

#[test]
fn refuses_a_skipped_quote_without_a_number_or_without_the_note_of_another_reason() {
    let value = changed(|value| {
        value["skipped"] = json!([
            { "block": "s3/p1", "quote": "Hydralazine alone", "reason": "cited" },
            { "block": "s2/p1", "quote": "treated for 48 h", "reason": "other" },
            { "block": "s2/p1", "quote": "treated for 3 h", "reason": "cited" }
        ]);
    });
    assert_eq!(
        errors(&value),
        [
            ("/skipped/0/quote".to_owned(), "skip_without_number"),
            ("/skipped/1/note".to_owned(), "skip_note_required"),
            ("/skipped/2/quote".to_owned(), "quote_not_found"),
        ]
    );
}

#[test]
fn accepts_a_submission_that_only_skips_quotes() {
    let value = changed(|value| {
        value["reports"] = json!([]);
        value["skipped"] = json!([
            { "block": "s2/p1", "quote": "48 h", "reason": "other", "note": "A test." }
        ]);
    });
    assert!(run(&value).is_ok());
    let empty = changed(|value| value["reports"] = json!([]));
    assert_eq!(errors(&empty), one("/reports", "no_reports"));
}

fn read_from_table(table: &str) -> Value {
    changed(|value| {
        let report = &mut value["reports"][0];
        report["value"]["number"] = json!("0.31");
        report["value"]["read_from_figure"] = json!({
            "figure": format!("{table}/caption"),
            "method": "transcribed from the image of the table",
            "resolution": "0.01"
        });
        report["evidence"] = json!([
            { "block": format!("{table}/caption"), "quote": "IC50 values" },
            {
                "block": "s2/p1",
                "quote": "treated for 48 h. Values are mean ± SD of three experiments (n = 3)"
            }
        ]);
    })
}

#[test]
fn accepts_a_value_read_from_the_image_of_a_table_and_warns_about_its_unit() {
    let found = warnings(&read_from_table("t2"));
    assert!(
        found.contains(&(
            "/reports/0/value/unit_text".to_owned(),
            "unit_text_from_image"
        )),
        "{found:?}"
    );
}

#[test]
fn refuses_to_read_the_image_of_a_table_whose_cells_are_in_the_text() {
    assert_eq!(
        errors(&read_from_table("t1")),
        one(
            "/reports/0/value/read_from_figure/figure",
            "table_has_cells"
        )
    );
}
