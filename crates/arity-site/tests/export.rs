#![cfg(test)]

use arity_core::{Digest, License, WorkMetadata};
use arity_protocol::check;
use arity_site::export;
use arity_store::{Blobs, Retrieval, RetrievalRole, Store, WorkId};
use arity_text::{Block, BlockKind, Document};
use jiff::Timestamp;
use serde_json::{Value, json};

fn at() -> Timestamp {
    "2026-11-14T10:00:00Z".parse().unwrap()
}

fn work(
    store: &Store,
    blobs: &Blobs,
    pmcid: &str,
    license: License,
    text: &str,
) -> (WorkId, Document) {
    let work = store
        .upsert_work(
            &WorkMetadata {
                title: format!("Work {pmcid}"),
                pmcid: Some(pmcid.parse().unwrap()),
                license: Some(license),
                year: Some(2024),
                ..WorkMetadata::default()
            },
            at(),
        )
        .unwrap();
    let document = Document::new(
        Digest::of(text.as_bytes()),
        "test 1",
        vec![Block::new("s1/p1", BlockKind::Paragraph, text)],
    )
    .unwrap();
    let xml = store
        .add_retrieval(
            work,
            &Retrieval {
                role: RetrievalRole::FullText,
                supplement_key: None,
                url: "https://example.org/a.xml".to_owned(),
                media_type: None,
                digest: document.source.clone(),
                bytes: 1,
                software: arity_store::SOFTWARE.to_owned(),
                retrieved_at: at(),
            },
        )
        .unwrap();
    let pdf = blobs.put(format!("%PDF {pmcid}").as_bytes()).unwrap();
    store
        .add_retrieval(
            work,
            &Retrieval {
                role: RetrievalRole::Pdf,
                supplement_key: None,
                url: "https://example.org/a.pdf".to_owned(),
                media_type: Some("application/pdf".to_owned()),
                digest: pdf,
                bytes: 9,
                software: arity_store::SOFTWARE.to_owned(),
                retrieved_at: at(),
            },
        )
        .unwrap();
    let text_id = store.add_text(work, xml, &document, at()).unwrap();
    let submission = json!({
        "format": "arity-submission/1",
        "work": pmcid,
        "text": document.digest().unwrap(),
        "reader": { "kind": "person", "name": "A Reader" },
        "responsible": { "name": "A Person" },
        "entities": [{
            "id": "dox", "kind": "chemical", "label": "doxorubicin", "xrefs": ["CHEBI:28748"],
            "mentions": [{ "block": "s1/p1", "quote": "doxorubicin" }]
        }],
        "properties": if store.properties().unwrap().is_empty() {
            json!([{ "id": "ic50", "label": "IC50", "value_type": "quantity", "unit": "umol/L" }])
        } else {
            json!([])
        },
        "reports": [{
            "type": "value", "id": "r1", "subject": ["dox"], "property": "ic50",
            "value": { "number": "240", "unit": "nmol/L", "unit_text": "nM", "uncertainty": { "kind": "sd", "number": "20" } },
            "evidence": [{ "block": "s1/p1", "quote": "240 ± 20 nM" }]
        }]
    });
    let raw = serde_json::to_vec(&submission).unwrap();
    let properties = store.properties().unwrap();
    let checked = check(&raw, &document, &properties).unwrap();
    store
        .store_submission(work, text_id, &raw, &checked, at())
        .unwrap();
    (work, document)
}

fn read(path: &std::path::Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

#[test]
fn writes_the_site_and_leaves_out_text_that_may_not_be_shared() {
    let folder = tempfile::tempdir().unwrap();
    let blobs = Blobs::new(folder.path().join("blobs"));
    let out = folder.path().join("site");
    let store = Store::open_in_memory().unwrap();
    let open = License::CreativeCommons {
        elements: arity_core::CcElements::By,
        version: Some("4.0".to_owned()),
    };
    let (shared, _) = work(
        &store,
        &blobs,
        "PMC1",
        open,
        "The IC50 of doxorubicin was 240 ± 20 nM.",
    );
    let (closed, _) = work(
        &store,
        &blobs,
        "PMC2",
        License::Unknown("free to read".to_owned()),
        "Here doxorubicin gave 240 ± 20 nM.",
    );

    let exported = export(&store, &blobs, &out, at()).unwrap();
    assert_eq!((exported.works, exported.reports, exported.pdfs), (2, 2, 1));

    let index = read(&out.join("index.json"));
    assert_eq!(index["format"], "arity-site/1");
    assert_eq!(index["works"][0]["shareable"], true);
    assert_eq!(index["works"][1]["shareable"], false);
    assert_eq!(index["properties"][0]["key"], "ic50");
    assert_eq!(index["properties"][0]["reports"], 2);
    assert_eq!(index["entities"][0]["reports"], 2);

    let reports = read(&out.join("reports.json"));
    let display = &reports[0]["display"];
    assert_eq!(display["unit"], "umol/L");
    assert!((display["value"].as_f64().unwrap() - 0.24).abs() < 1e-12);
    assert!((display["interval_high"].as_f64().unwrap() - 0.26).abs() < 1e-12);

    let shared_page = read(&out.join(format!("works/{}.json", shared.0)));
    assert_eq!(shared_page["pdf"], format!("works/{}.pdf", shared.0));
    assert_eq!(shared_page["document"]["blocks"][0]["key"], "s1/p1");
    assert_eq!(shared_page["attribution"]["license"], "CC BY 4.0");
    assert_eq!(shared_page["runs"][0]["responsible"]["name"], "A Person");
    assert!(out.join(format!("works/{}.pdf", shared.0)).is_file());

    let closed_page = read(&out.join(format!("works/{}.json", closed.0)));
    assert_eq!(closed_page["pdf"], Value::Null);
    assert_eq!(closed_page["document"], Value::Null);
    assert!(!out.join(format!("works/{}.pdf", closed.0)).exists());

    let copy = Store::open(&out.join("arity.sqlite")).unwrap();
    assert!(copy.latest_text(shared).unwrap().is_some());
    assert_eq!(copy.report_views().unwrap().len(), 2);
    let closed_text = copy.latest_text(closed);
    assert!(closed_text.is_err() || closed_text.unwrap().is_none());
}
