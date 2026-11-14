#![cfg(test)]

use arity_core::{Digest, Notice, NoticeKind, NoticeSource, WorkMetadata};
use arity_store::{QueueEntry, QueueState, Retrieval, RetrievalRole, Store};
use jiff::Timestamp;

fn at(text: &str) -> Timestamp {
    text.parse().unwrap()
}

fn metadata() -> WorkMetadata {
    WorkMetadata {
        title: "A study".to_owned(),
        pmcid: Some("PMC123".parse().unwrap()),
        doi: Some("10.1000/a.1".parse().unwrap()),
        year: Some(2023),
        ..WorkMetadata::default()
    }
}

#[test]
fn migrates_a_new_database() {
    let store = Store::open_in_memory().unwrap();
    assert_eq!(store.schema_version().unwrap(), 3);
}

#[test]
fn reopens_a_database_file() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("arity.sqlite");
    let id = {
        let store = Store::open(&path).unwrap();
        store
            .upsert_work(&metadata(), at("2026-11-14T10:00:00Z"))
            .unwrap()
    };
    let store = Store::open(&path).unwrap();
    assert_eq!(store.work(id).unwrap().metadata.title, "A study");
}

#[test]
fn adds_and_finds_works() {
    let store = Store::open_in_memory().unwrap();
    let id = store
        .upsert_work(&metadata(), at("2026-11-14T10:00:00Z"))
        .unwrap();
    assert_eq!(store.find_work("PMC123").unwrap(), Some(id));
    assert_eq!(
        store.find_work("https://doi.org/10.1000/A.1").unwrap(),
        Some(id)
    );
    assert_eq!(store.find_work("PMC999").unwrap(), None);
    assert_eq!(store.find_work("not an identifier").unwrap(), None);
    let record = store.work(id).unwrap();
    assert_eq!(record.metadata, metadata());
    assert_eq!(record.state, None);
}

#[test]
fn replaces_a_record_and_keeps_missing_identifiers() {
    let store = Store::open_in_memory().unwrap();
    let id = store
        .upsert_work(&metadata(), at("2026-11-14T10:00:00Z"))
        .unwrap();
    let newer = WorkMetadata {
        title: "A study, from the full text".to_owned(),
        pmcid: Some("PMC123".parse().unwrap()),
        pmid: Some("42".parse().unwrap()),
        ..WorkMetadata::default()
    };
    assert_eq!(
        store
            .upsert_work(&newer, at("2026-11-14T11:00:00Z"))
            .unwrap(),
        id
    );
    let record = store.work(id).unwrap();
    assert_eq!(record.metadata.title, "A study, from the full text");
    assert_eq!(record.metadata.doi, metadata().doi);
    assert_eq!(record.metadata.pmid.unwrap().as_str(), "42");
    assert_eq!(record.added_at, at("2026-11-14T10:00:00Z"));
    assert_eq!(record.updated_at, at("2026-11-14T11:00:00Z"));
    assert_eq!(store.works().unwrap().len(), 1);
}

#[test]
fn keeps_a_queue_in_the_order_found() {
    let store = Store::open_in_memory().unwrap();
    let first = store
        .upsert_work(&metadata(), at("2026-11-14T10:00:00Z"))
        .unwrap();
    let second = store
        .upsert_work(
            &WorkMetadata {
                title: "Another".to_owned(),
                pmcid: Some("PMC456".parse().unwrap()),
                ..WorkMetadata::default()
            },
            at("2026-11-14T10:00:00Z"),
        )
        .unwrap();
    assert!(
        store
            .enqueue(second, Some("query b"), at("2026-11-14T10:01:00Z"))
            .unwrap()
    );
    assert!(
        store
            .enqueue(first, Some("query a"), at("2026-11-14T10:02:00Z"))
            .unwrap()
    );
    assert!(
        !store
            .enqueue(first, None, at("2026-11-14T10:03:00Z"))
            .unwrap()
    );
    assert_eq!(
        store.in_state(QueueState::Queued, 10).unwrap(),
        [second, first]
    );

    store
        .set_state(
            second,
            QueueState::Fetched,
            None,
            at("2026-11-14T10:04:00Z"),
        )
        .unwrap();
    assert_eq!(store.in_state(QueueState::Queued, 10).unwrap(), [first]);
    assert_eq!(store.work(second).unwrap().state, Some(QueueState::Fetched));
}

#[test]
fn records_retrievals() {
    let store = Store::open_in_memory().unwrap();
    let work = store
        .upsert_work(&metadata(), at("2026-11-14T10:00:00Z"))
        .unwrap();
    let retrieval = Retrieval {
        role: RetrievalRole::FullText,
        supplement_key: None,
        url: "https://example.org/a.xml".to_owned(),
        media_type: Some("application/xml".to_owned()),
        digest: Digest::of(b"<article/>"),
        bytes: 10,
        software: "arity 0.0.9".to_owned(),
        retrieved_at: at("2026-11-14T10:05:00Z"),
    };
    let id = store.add_retrieval(work, &retrieval).unwrap();
    assert_eq!(store.retrievals(work).unwrap(), [(id, retrieval)]);
}

#[test]
fn restores_the_queue_entry_and_the_times_of_a_work() {
    let store = Store::open_in_memory().unwrap();
    let work = store
        .upsert_work(&metadata(), at("2026-11-14T10:00:00Z"))
        .unwrap();
    assert_eq!(store.queue_entry(work).unwrap(), None);
    let entry = QueueEntry {
        state: QueueState::Failed,
        query: Some("doxorubicin".to_owned()),
        note: Some("no files".to_owned()),
        added_at: at("2026-11-14T10:01:00Z"),
        updated_at: at("2026-11-14T10:02:00Z"),
    };
    store.restore_queue(work, &entry).unwrap();
    assert_eq!(store.queue_entry(work).unwrap(), Some(entry.clone()));
    let replaced = QueueEntry {
        state: QueueState::Read,
        note: None,
        ..entry
    };
    store.restore_queue(work, &replaced).unwrap();
    assert_eq!(store.queue_entry(work).unwrap(), Some(replaced));

    store
        .restore_times(work, at("2026-11-13T09:00:00Z"), at("2026-11-14T11:00:00Z"))
        .unwrap();
    let record = store.work(work).unwrap();
    assert_eq!(
        (record.added_at, record.updated_at),
        (at("2026-11-13T09:00:00Z"), at("2026-11-14T11:00:00Z"))
    );
}

#[test]
fn stores_each_notice_once() {
    let store = Store::open_in_memory().unwrap();
    let work = store
        .upsert_work(&metadata(), at("2026-11-14T10:00:00Z"))
        .unwrap();
    let notice = Notice {
        kind: NoticeKind::Retraction,
        reported_type: "retraction".to_owned(),
        source: NoticeSource::Crossref,
        notice_doi: Some("10.1000/r.1".parse().unwrap()),
        notice_pmid: None,
        notice_citation: None,
        date: Some(jiff::civil::date(2025, 3, 4)),
        source_record: None,
        retrieved_at: at("2026-11-14T10:00:00Z"),
    };
    assert!(store.add_notice(work, &notice).unwrap());
    assert!(!store.add_notice(work, &notice).unwrap());
    assert_eq!(store.notices(work).unwrap(), [notice]);
}
