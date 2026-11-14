#![cfg(test)]

use std::collections::HashMap;

use arity_core::vocabulary::CheckState;
use arity_core::{Digest, WorkMetadata};
use arity_protocol::{Person, PropertyDefinition, check};
use arity_store::{
    CheckEvent, Retrieval, RetrievalRole, RunRecord, Store, StoreError, TextId, TextRecord, WorkId,
};
use arity_text::{Block, BlockKind, Document};
use jiff::Timestamp;
use serde_json::{Value, json};

fn at(text: &str) -> Timestamp {
    text.parse().unwrap()
}

fn document(text: &str) -> Document {
    Document::new(
        Digest::of(text.as_bytes()),
        "test 1",
        vec![
            Block::new("s1/p1", BlockKind::Paragraph, text),
            Block::new("t1/r1c2", BlockKind::TableCell, "IC50 (μM)"),
        ],
    )
    .unwrap()
}

fn setup(store: &Store, pmcid: &str, text: &str) -> (WorkId, TextId, Document) {
    let work = store
        .upsert_work(
            &WorkMetadata {
                title: format!("Work {pmcid}"),
                pmcid: Some(pmcid.parse().unwrap()),
                ..WorkMetadata::default()
            },
            at("2026-11-14T10:00:00Z"),
        )
        .unwrap();
    let document = document(text);
    let retrieval = store
        .add_retrieval(
            work,
            &Retrieval {
                role: RetrievalRole::FullText,
                supplement_key: None,
                url: format!("https://example.org/{pmcid}.xml"),
                media_type: Some("application/xml".to_owned()),
                digest: document.source.clone(),
                bytes: 1,
                software: arity_store::SOFTWARE.to_owned(),
                retrieved_at: at("2026-11-14T10:00:00Z"),
            },
        )
        .unwrap();
    let text_id = store
        .add_text(work, retrieval, &document, at("2026-11-14T10:00:00Z"))
        .unwrap();
    (work, text_id, document)
}

fn submission(document: &Document, number: &str, kind: &str) -> Value {
    json!({
        "format": "arity-submission/1",
        "work": "PMC1",
        "text": document.digest().unwrap(),
        "reader": { "kind": "software", "name": "reader", "version": "1" },
        "responsible": { "name": "A Person", "orcid": "0000-0002-1825-0097" },
        "entities": [{
            "id": "dox", "kind": kind, "label": "doxorubicin",
            "xrefs": ["CHEBI:28748"],
            "mentions": [{ "block": "s1/p1", "quote": "doxorubicin" }]
        }],
        "properties": [{
            "id": "ic50", "label": "half maximal inhibitory concentration",
            "value_type": "quantity", "unit": "umol/L"
        }],
        "reports": [{
            "type": "value", "id": "r1", "subject": ["dox"], "property": "ic50",
            "value": { "number": number, "unit": "umol/L", "unit_text": "μM" },
            "evidence": [
                { "block": "s1/p1", "quote": format!("IC50 of doxorubicin was {number} μM") },
                { "block": "t1/r1c2", "quote": "IC50 (μM)" }
            ]
        }]
    })
}

fn store_one(
    store: &Store,
    work: WorkId,
    text: TextId,
    document: &Document,
    value: &Value,
) -> arity_store::Result<arity_store::RunId> {
    let raw = serde_json::to_vec(value).unwrap();
    let registry: HashMap<String, PropertyDefinition> = store
        .properties()
        .unwrap()
        .into_iter()
        .map(|definition| (definition.key.clone(), definition))
        .collect();
    let checked = check(&raw, document, &registry)
        .unwrap_or_else(|rejected| panic!("{:#?}", rejected.issues));
    store.store_submission(work, text, &raw, &checked, at("2026-11-14T12:00:00Z"))
}

#[test]
fn stores_texts_once() {
    let store = Store::open_in_memory().unwrap();
    let (work, text, document) = setup(&store, "PMC1", "The IC50 of doxorubicin was 0.24 μM.");
    let (stored_work, stored) = store.text(text).unwrap();
    assert_eq!((stored_work, stored), (work, document.clone()));
    assert_eq!(store.latest_text(work).unwrap().unwrap().0, text);
    let digest = document.digest().unwrap();
    assert_eq!(store.text_by_digest(&digest).unwrap(), Some(text));
}

#[test]
fn gives_back_the_texts_the_runs_and_the_checks() {
    let store = Store::open_in_memory().unwrap();
    let (work, text, document) = setup(&store, "PMC1", "The IC50 of doxorubicin was 0.24 μM.");
    let value = submission(&document, "0.24", "chemical");
    let run = store_one(&store, work, text, &document, &value).unwrap();
    let report = store.reports_of_run(run).unwrap()[0].0;
    let person = Person {
        name: "A Person".to_owned(),
        orcid: None,
    };
    store
        .record_check(
            report,
            CheckState::Rejected,
            &person,
            Some("The unit is not the unit of the source."),
            at("2026-11-14T13:00:00Z"),
        )
        .unwrap();

    assert_eq!(
        store.texts(work).unwrap(),
        [TextRecord {
            digest: document.digest().unwrap(),
            parser: "test 1".to_owned(),
            source: document.source.clone(),
        }]
    );
    let raw = serde_json::to_string(&value).unwrap();
    assert_eq!(
        store.runs().unwrap(),
        [RunRecord {
            id: run,
            work,
            digest: Digest::of(raw.as_bytes()),
            submission: raw,
            software: arity_store::SOFTWARE.to_owned(),
            submitted_at: at("2026-11-14T12:00:00Z"),
        }]
    );
    assert_eq!(
        store.check_events().unwrap(),
        [CheckEvent {
            run,
            report: "r1".to_owned(),
            state: CheckState::Rejected,
            person,
            note: Some("The unit is not the unit of the source.".to_owned()),
            at: at("2026-11-14T13:00:00Z"),
        }]
    );
}

#[test]
fn stores_a_checked_submission_with_its_provenance() {
    let store = Store::open_in_memory().unwrap();
    let (work, text, document) = setup(&store, "PMC1", "The IC50 of doxorubicin was 0.24 μM.");
    let value = submission(&document, "0.24", "chemical");
    let run = store_one(&store, work, text, &document, &value).unwrap();
    let reports = store.reports_of_run(run).unwrap();
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].1, "r1");
    assert_eq!(
        store.check_state(reports[0].0).unwrap(),
        CheckState::Unchecked
    );
    let properties = store.properties().unwrap();
    assert_eq!(properties.len(), 1);
    assert_eq!(properties[0].unit.as_deref(), Some("umol/L"));

    let again = store_one(&store, work, text, &document, &value).unwrap();
    assert_eq!(again, run);
}

#[test]
fn links_entities_across_works_by_identifier() {
    let store = Store::open_in_memory().unwrap();
    let (first_work, first_text, first) =
        setup(&store, "PMC1", "The IC50 of doxorubicin was 0.24 μM.");
    let (second_work, second_text, second) =
        setup(&store, "PMC2", "The IC50 of doxorubicin was 1.1 μM.");
    store_one(
        &store,
        first_work,
        first_text,
        &first,
        &submission(&first, "0.24", "chemical"),
    )
    .unwrap();
    let mut value = submission(&second, "1.1", "chemical");
    value["properties"] = json!([]);
    store_one(&store, second_work, second_text, &second, &value).unwrap();
    assert_eq!(store.entity_count().unwrap(), 1);
}

#[test]
fn refuses_an_identifier_that_belongs_to_an_entity_of_another_kind() {
    let store = Store::open_in_memory().unwrap();
    let (first_work, first_text, first) =
        setup(&store, "PMC1", "The IC50 of doxorubicin was 0.24 μM.");
    let (second_work, second_text, second) =
        setup(&store, "PMC2", "The IC50 of doxorubicin was 1.1 μM.");
    let mut value = submission(&first, "0.24", "chemical");
    value["entities"][0]["xrefs"] = json!(["CHEBI:28748", "MESH:D004317"]);
    store_one(&store, first_work, first_text, &first, &value).unwrap();
    let mut other_kind = submission(&second, "1.1", "other");
    other_kind["properties"] = json!([]);
    other_kind["entities"][0]["xrefs"] = json!(["MESH:D004317"]);
    assert!(matches!(
        store_one(&store, second_work, second_text, &second, &other_kind),
        Err(StoreError::Conflict(_))
    ));
}

#[test]
fn refuses_identifiers_that_belong_to_different_entities() {
    let store = Store::open_in_memory().unwrap();
    let (first_work, first_text, first) =
        setup(&store, "PMC1", "The IC50 of doxorubicin was 0.24 μM.");
    let (second_work, second_text, second) =
        setup(&store, "PMC2", "The IC50 of doxorubicin was 1.1 μM.");
    let (third_work, third_text, third) =
        setup(&store, "PMC3", "The IC50 of doxorubicin was 2.5 μM.");
    store_one(
        &store,
        first_work,
        first_text,
        &first,
        &submission(&first, "0.24", "chemical"),
    )
    .unwrap();
    let mut cisplatin = submission(&second, "1.1", "chemical");
    cisplatin["properties"] = json!([]);
    cisplatin["entities"][0]["xrefs"] = json!(["CHEBI:27899"]);
    store_one(&store, second_work, second_text, &second, &cisplatin).unwrap();
    let mut both = submission(&third, "2.5", "chemical");
    both["properties"] = json!([]);
    both["entities"][0]["xrefs"] = json!(["CHEBI:28748", "CHEBI:27899"]);
    assert!(matches!(
        store_one(&store, third_work, third_text, &third, &both),
        Err(StoreError::Conflict(_))
    ));
    assert_eq!(store.entity_count().unwrap(), 2);
}

#[test]
fn records_checks_by_people() {
    let store = Store::open_in_memory().unwrap();
    let (work, text, document) = setup(&store, "PMC1", "The IC50 of doxorubicin was 0.24 μM.");
    let run = store_one(
        &store,
        work,
        text,
        &document,
        &submission(&document, "0.24", "chemical"),
    )
    .unwrap();
    let report = store.reports_of_run(run).unwrap()[0].0;
    let person = Person {
        name: "A Person".to_owned(),
        orcid: None,
    };
    store
        .record_check(
            report,
            CheckState::Confirmed,
            &person,
            None,
            at("2026-11-14T13:00:00Z"),
        )
        .unwrap();
    assert_eq!(store.check_state(report).unwrap(), CheckState::Confirmed);
    store
        .record_check(
            report,
            CheckState::Rejected,
            &person,
            Some("the value is for a different cell line"),
            at("2026-11-14T14:00:00Z"),
        )
        .unwrap();
    assert_eq!(store.check_state(report).unwrap(), CheckState::Rejected);
    let nobody = Person {
        name: " ".to_owned(),
        orcid: None,
    };
    assert!(
        store
            .record_check(
                report,
                CheckState::Confirmed,
                &nobody,
                None,
                at("2026-11-14T15:00:00Z")
            )
            .is_err()
    );
}

#[test]
fn shows_reports_with_their_evidence_checks_and_provenance() {
    let store = Store::open_in_memory().unwrap();
    let (work, text, document) = setup(&store, "PMC1", "The IC50 of doxorubicin was 0.24 μM.");
    let run = store_one(
        &store,
        work,
        text,
        &document,
        &submission(&document, "0.24", "chemical"),
    )
    .unwrap();
    let report = store.reports_of_run(run).unwrap()[0].0;
    let person = Person {
        name: "A Person".to_owned(),
        orcid: None,
    };
    store
        .record_check(
            report,
            CheckState::Confirmed,
            &person,
            None,
            at("2026-11-14T13:00:00Z"),
        )
        .unwrap();

    let reports = store.report_views().unwrap();
    assert_eq!(reports.len(), 1);
    let view = &reports[0];
    assert_eq!(view.kind, "value");
    assert_eq!(view.property.as_deref(), Some("ic50"));
    assert_eq!(view.check, "confirmed");
    assert_eq!(view.checks[0].person, "A Person");
    assert_eq!(view.body["value"]["number"], "0.24");
    assert_eq!(view.coherent.unit.as_deref(), Some("m-3.mol"));
    assert!((view.coherent.value.unwrap() - 0.24e-3).abs() < 1e-12);
    let entity = view.entities["dox"];
    assert_eq!(view.evidence.len(), 2);
    assert_eq!(view.evidence[0].block, "s1/p1");
    assert_eq!(view.evidence[0].start, 4);

    let runs = store.run_views().unwrap();
    assert_eq!(runs[0].reader.kind, "software");
    assert_eq!(runs[0].responsible.name, "A Person");
    assert_eq!(
        runs[0].responsible.orcid.as_deref(),
        Some("0000-0002-1825-0097")
    );
    assert_eq!(runs[0].text_sha256, document.digest().unwrap().as_str());

    let entities = store.entity_views().unwrap();
    assert_eq!(entities[0].id, entity);
    assert_eq!(entities[0].xrefs, ["CHEBI:28748"]);
    assert_eq!(entities[0].names, ["doxorubicin"]);

    let properties = store.property_views().unwrap();
    assert_eq!(properties[0].dimension.as_deref(), Some("L-3 N"));

    let works = store.work_views().unwrap();
    assert_eq!(works[0].reports, 1);
    assert_eq!(works[0].retrievals[0].role, "full_text");
    assert_eq!(
        works[0].text_sha256.as_deref(),
        Some(document.digest().unwrap().as_str())
    );
}

#[test]
fn refuses_a_file_that_the_checks_did_not_read() {
    let store = Store::open_in_memory().unwrap();
    let (work, text, document) = setup(&store, "PMC1", "The IC50 of doxorubicin was 0.24 μM.");
    let value = submission(&document, "0.24", "chemical");
    let raw = serde_json::to_vec(&value).unwrap();
    let checked = check(&raw, &document, &HashMap::new()).unwrap();
    let other = serde_json::to_vec_pretty(&value).unwrap();
    assert!(matches!(
        store.store_submission(work, text, &other, &checked, at("2026-11-14T12:00:00Z")),
        Err(StoreError::Conflict(_))
    ));
}
