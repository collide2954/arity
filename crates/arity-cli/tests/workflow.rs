#![cfg(test)]

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

use arity_cli::config::{Config, PersonConfig};
use arity_cli::submit::Outcome;
use arity_cli::{Context, archive, catalogue, checks, coverage, fetch, show, submit};
use arity_core::vocabulary::CheckState;
use arity_core::{CcElements, License, Notice, NoticeKind, NoticeSource};
use arity_sources::{Http, Response, SourceError, europepmc, pmc};
use jiff::Timestamp;
use serde_json::json;

const XML: &[u8] = include_bytes!("../../../testdata/jats/PMC6274071.xml");
const TOPIC: &str = "biphenylurea";

struct Fake(HashMap<String, Vec<u8>>);

impl Http for Fake {
    fn get(&self, url: &str) -> Result<Response, SourceError> {
        Ok(match self.0.get(url) {
            Some(body) => Response {
                status: 200,
                content_type: None,
                body: body.clone(),
            },
            None => Response {
                status: 404,
                content_type: None,
                body: Vec::new(),
            },
        })
    }
}

fn fake() -> Fake {
    let pmcid = "PMC6274071".parse().unwrap();
    let query = europepmc::open_access_query(TOPIC);
    let first_page = json!({
        "hitCount": 1,
        "nextCursorMark": "next",
        "resultList": { "result": [{
            "source": "MED",
            "pmcid": "PMC6274071",
            "doi": "10.3390/molecules21060762",
            "title": "Synthesis and Cytotoxic Activity of Biphenylurea Derivatives",
            "pubYear": "2016",
            "isOpenAccess": "Y",
            "inPMC": "Y",
            "license": "cc by"
        }]}
    });
    let last_page =
        json!({ "hitCount": 1, "nextCursorMark": "next", "resultList": { "result": [] } });
    let listing = r#"<?xml version="1.0" encoding="UTF-8"?><ListBucketResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/"><CommonPrefixes><Prefix>PMC6274071.1/</Prefix></CommonPrefixes></ListBucketResult>"#;
    let record = json!({
        "pmcid": "PMC6274071",
        "version": 1,
        "is_pmc_openaccess": true,
        "license_code": "CC BY",
        "xml_url": "s3://pmc-oa-opendata/PMC6274071.1/PMC6274071.1.xml?md5=00000000000000000000000000000000",
        "pdf_url": "s3://pmc-oa-opendata/PMC6274071.1/PMC6274071.1.pdf?md5=00000000000000000000000000000000",
        "media_urls": ["s3://pmc-oa-opendata/PMC6274071.1/molecules-21-00762-s001.pdf?md5=00000000000000000000000000000000"]
    });
    Fake(
        [
            (
                europepmc::search_url(&query, 5, "*"),
                serde_json::to_vec(&first_page).unwrap(),
            ),
            (
                europepmc::search_url(&query, 4, "next"),
                serde_json::to_vec(&last_page).unwrap(),
            ),
            (pmc::list_url(&pmcid), listing.as_bytes().to_vec()),
            (
                pmc::record_url(&pmcid, 1),
                serde_json::to_vec(&record).unwrap(),
            ),
            (
                format!("{}/PMC6274071.1/PMC6274071.1.xml", pmc::BUCKET),
                XML.to_vec(),
            ),
            (
                format!("{}/PMC6274071.1/PMC6274071.1.pdf", pmc::BUCKET),
                b"%PDF-1.4 article".to_vec(),
            ),
            (
                format!("{}/PMC6274071.1/molecules-21-00762-s001.pdf", pmc::BUCKET),
                b"%PDF-1.4 supplement".to_vec(),
            ),
        ]
        .into_iter()
        .collect(),
    )
}

fn now() -> Timestamp {
    "2026-11-14T10:00:00Z".parse().unwrap()
}

fn context(directory: &Path) -> Context {
    Config {
        person: Some(PersonConfig {
            name: "A Person".to_owned(),
            orcid: None,
        }),
        ..Config::default()
    }
    .save(directory)
    .unwrap();
    Context::open(directory, Box::new(fake())).unwrap()
}

fn submission(context: &Context, number: &str) -> serde_json::Value {
    let work = context.work("PMC6274071").unwrap();
    let (_, document) = context.store.latest_text(work).unwrap().unwrap();
    let block = document
        .blocks
        .iter()
        .find(|block| {
            block
                .text
                .contains("reference drug, doxorubicin (IC₅₀ = 7.30")
        })
        .unwrap();
    json!({
        "format": "arity-submission/1",
        "work": "PMC6274071",
        "text": document.digest().unwrap(),
        "reader": { "kind": "person", "name": "A Reader" },
        "responsible": { "name": "A Person" },
        "entities": [
            {
                "id": "dox", "kind": "chemical", "label": "doxorubicin", "xrefs": ["CHEBI:28748"],
                "mentions": [{ "block": block.key, "quote": "doxorubicin" }]
            },
            {
                "id": "mcf7", "kind": "cell_line", "label": "MCF-7", "xrefs": ["CELLOSAURUS:CVCL_0031"],
                "mentions": [{ "block": block.key, "quote": "MCF-7", "occurrence": 1 }]
            }
        ],
        "properties": [{
            "id": "ic50", "label": "half maximal inhibitory concentration",
            "value_type": "quantity", "unit": "umol/L"
        }],
        "reports": [{
            "type": "value", "id": "r1", "subject": ["dox"], "property": "ic50",
            "value": {
                "number": number, "unit": "umol/L", "unit_text": "μM",
                "uncertainty": { "kind": "other", "number": "0.84" }
            },
            "conditions": [{ "role": "cell_line", "entity": "mcf7" }],
            "evidence": [{ "block": block.key, "quote": "doxorubicin (IC₅₀ = 7.30 ± 0.84 μM)" }]
        }]
    })
}

#[test]
fn catalogues_fetches_reads_submits_and_checks_a_work() {
    let directory = tempfile::tempdir().unwrap();
    let context = context(directory.path());

    let added = catalogue::search(&context, TOPIC, 5, now()).unwrap();
    assert_eq!(added.len(), 1);
    assert!(added[0].new);
    let work = added[0].work;

    let fetched = fetch::fetch(&context, work, now()).unwrap();
    assert_eq!(fetched.version, "PMC6274071.1");
    assert!(fetched.pdf);
    assert_eq!(fetched.supplements, 1);
    assert_eq!(
        fetched.license,
        Some(License::CreativeCommons {
            elements: CcElements::By,
            version: Some("4.0".to_owned())
        })
    );
    let record = context.store.work(work).unwrap();
    assert_eq!(record.metadata.authors[0].name, "Wagdy M Eldehna");
    assert_eq!(context.store.retrievals(work).unwrap().len(), 3);

    let (_, document) = context.store.latest_text(work).unwrap().unwrap();
    let digest = document.digest().unwrap();
    let text = show::render(&record.metadata, &digest, &document);
    assert!(text.contains("[t1/r3c5] 19.53 ± 1.05"));

    let path = directory.path().join("submission.json");
    fs::write(
        &path,
        serde_json::to_vec_pretty(&submission(&context, "7.30")).unwrap(),
    )
    .unwrap();
    let Outcome::Passed { .. } = submit::submit(&context, &path, true, now()).unwrap() else {
        panic!("the dry run did not pass");
    };
    let Outcome::Stored { reports, .. } = submit::submit(&context, &path, false, now()).unwrap()
    else {
        panic!("the submission was not stored");
    };
    assert_eq!(reports.len(), 1);
    let named = show::entities(&context.store.entities_of_work(work).unwrap());
    assert!(named.contains("  dox  chemical  doxorubicin"), "{named}");
    assert_eq!(
        context.store.work(work).unwrap().state,
        Some(arity_store::QueueState::Read)
    );

    submit::record_check(&context, reports[0].0, CheckState::Confirmed, None, now()).unwrap();
    assert_eq!(
        context.store.check_state(reports[0].0).unwrap(),
        CheckState::Confirmed
    );
}

#[test]
fn lists_the_numbers_that_no_report_quotes() {
    let directory = tempfile::tempdir().unwrap();
    let context = context(directory.path());
    let work = catalogue::search(&context, TOPIC, 5, now()).unwrap()[0].work;
    fetch::fetch(&context, work, now()).unwrap();
    let (_, before) = coverage::of_work(&context, work).unwrap();
    assert!(before.cells.numbers > 100, "{:?}", before.cells);
    assert_eq!((before.cells.quoted, before.text.quoted), (0, 0));

    let path = directory.path().join("submission.json");
    fs::write(
        &path,
        serde_json::to_vec(&submission(&context, "7.30")).unwrap(),
    )
    .unwrap();
    submit::submit(&context, &path, false, now()).unwrap();
    let (document, after) = coverage::of_work(&context, work).unwrap();
    assert_eq!(after.text.quoted, 2);
    assert_eq!(after.text.numbers, before.text.numbers);
    assert_eq!(after.unquoted.len(), before.unquoted.len() - 2);

    let text = coverage::render(&document, &after);
    assert!(text.starts_with("Table cells: "), "{text}");
    assert!(
        text.contains("Reports quote 0, readers skipped 0"),
        "{text}"
    );
    assert!(text.contains("[t1/r3c5] 19.53 ± 1.05"), "{text}");
}

#[test]
fn rejects_a_submission_with_a_number_that_the_source_does_not_state() {
    let directory = tempfile::tempdir().unwrap();
    let context = context(directory.path());
    let work = catalogue::search(&context, TOPIC, 5, now()).unwrap()[0].work;
    fetch::fetch(&context, work, now()).unwrap();
    let path = directory.path().join("submission.json");
    fs::write(
        &path,
        serde_json::to_vec(&submission(&context, "7.31")).unwrap(),
    )
    .unwrap();
    let Outcome::Rejected { issues } = submit::submit(&context, &path, false, now()).unwrap()
    else {
        panic!("the submission was not rejected");
    };
    let codes: Vec<&str> = issues.iter().map(|issue| issue.code.as_str()).collect();
    assert!(codes.contains(&"number_not_in_evidence"), "{codes:?}");
}

#[test]
fn adds_the_cells_of_supplementary_data_files_to_the_text() {
    let article = br#"<article xmlns:xlink="http://www.w3.org/1999/xlink" article-type="research-article"><front><article-meta><article-id pub-id-type="pmcid">PMC77</article-id><title-group><article-title>Data</article-title></title-group><permissions><license><ali:license_ref xmlns:ali="http://www.niso.org/schemas/ali/1.0/">https://creativecommons.org/licenses/by/4.0/</ali:license_ref></license></permissions></article-meta></front><body><p>See the data.</p><supplementary-material><label>Data S1</label><media xlink:href="values.csv" mimetype="text" mime-subtype="csv"/></supplementary-material><supplementary-material><label>Data S2</label><media xlink:href="broken.xlsx"/></supplementary-material></body></article>"#;
    let pmcid = "PMC77".parse().unwrap();
    let record = json!({
        "pmcid": "PMC77", "version": 2,
        "xml_url": "s3://pmc-oa-opendata/PMC77.2/PMC77.2.xml",
        "media_urls": [
            "s3://pmc-oa-opendata/PMC77.2/values.csv",
            "s3://pmc-oa-opendata/PMC77.2/broken.xlsx"
        ]
    });
    let fake = Fake(
        [
            (
                pmc::list_url(&pmcid),
                b"<ListBucketResult><CommonPrefixes><Prefix>PMC77.2/</Prefix></CommonPrefixes></ListBucketResult>".to_vec(),
            ),
            (pmc::record_url(&pmcid, 2), serde_json::to_vec(&record).unwrap()),
            (format!("{}/PMC77.2/PMC77.2.xml", pmc::BUCKET), article.to_vec()),
            (format!("{}/PMC77.2/values.csv", pmc::BUCKET), b"compound,IC50\nDOX,0.24\n".to_vec()),
            (format!("{}/PMC77.2/broken.xlsx", pmc::BUCKET), b"not a workbook".to_vec()),
        ]
        .into_iter()
        .collect(),
    );
    let directory = tempfile::tempdir().unwrap();
    let context = Context::open(directory.path(), Box::new(fake)).unwrap();
    let work = context
        .store
        .upsert_work(
            &arity_core::WorkMetadata {
                title: "Data".to_owned(),
                pmcid: Some("PMC77".parse().unwrap()),
                ..arity_core::WorkMetadata::default()
            },
            now(),
        )
        .unwrap();
    let fetched = fetch::fetch(&context, work, now()).unwrap();
    assert_eq!(fetched.supplements, 2);
    assert_eq!(fetched.skipped.len(), 1);
    assert!(fetched.skipped[0].starts_with("broken.xlsx"));
    let (_, document) = context.store.latest_text(work).unwrap().unwrap();
    assert_eq!(document.block("supp1/s1/r2c2").unwrap().text, "0.24");
    assert_eq!(document.attachments.len(), 1);
    assert_eq!(document.attachments[0].name, "values.csv");
}

fn files_in(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut files = BTreeMap::new();
    let mut folders = vec![root.to_path_buf()];
    while let Some(folder) = folders.pop() {
        for entry in fs::read_dir(&folder).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                folders.push(path);
            } else {
                let name = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                files.insert(name, fs::read(&path).unwrap());
            }
        }
    }
    files
}

/// A database with one stored submission, and the run of its report `r1`.
fn stored(directory: &Path) -> (Context, arity_store::RunId, arity_store::ReportId) {
    let context = context(directory);
    let work = catalogue::search(&context, TOPIC, 5, now()).unwrap()[0].work;
    fetch::fetch(&context, work, now()).unwrap();
    let path = directory.join("submission.json");
    fs::write(
        &path,
        serde_json::to_vec_pretty(&submission(&context, "7.30")).unwrap(),
    )
    .unwrap();
    let Outcome::Stored { run, reports, .. } =
        submit::submit(&context, &path, false, now()).unwrap()
    else {
        panic!("the submission was not stored");
    };
    (context, run, reports[0].0)
}

fn check_file(run: arity_store::RunId, report: &str, state: &str, at: &str) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "format": "arity-checks/1",
        "person": { "name": "A Checker", "orcid": "0000-0002-1825-0097" },
        "checks": [{
            "work": "PMC6274071", "run": run.0, "report": report, "state": state,
            "note": "The source states 7.30 ± 0.84 μM.", "at": at
        }]
    }))
    .unwrap()
}

#[test]
fn records_the_checks_of_a_check_file_with_the_person_of_the_file() {
    let directory = tempfile::tempdir().unwrap();
    let (context, run, report) = stored(directory.path());
    let file = check_file(run, "r1", "rejected", "2026-11-14T09:00:00Z");

    assert_eq!(checks::record(&context, &file, now()).unwrap(), 1);
    assert_eq!(
        context.store.check_state(report).unwrap(),
        CheckState::Rejected
    );
    let events = context.store.check_events().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].person.name, "A Checker");
    assert_eq!(
        events[0].note.as_deref(),
        Some("The source states 7.30 ± 0.84 μM.")
    );
    assert_eq!(events[0].at, "2026-11-14T09:00:00Z".parse().unwrap());

    assert_eq!(checks::record(&context, &file, now()).unwrap(), 0);
    assert_eq!(context.store.check_events().unwrap().len(), 1);
}

#[test]
fn refuses_a_check_file_with_a_problem_and_records_none_of_its_checks() {
    let directory = tempfile::tempdir().unwrap();
    let (context, run, _) = stored(directory.path());
    let at = "2026-11-14T09:00:00Z";
    for (file, problem) in [
        (check_file(run, "r9", "confirmed", at), "no report r9"),
        (check_file(run, "r1", "unchecked", at), "confirm or reject"),
        (
            check_file(run, "r1", "confirmed", "2026-11-15T09:00:00Z"),
            "later than now",
        ),
        (
            check_file(arity_store::RunId(run.0 + 1), "r1", "confirmed", at),
            "no run",
        ),
        (b"{\"format\": \"arity-checks/9\"}".to_vec(), "check file"),
    ] {
        let error = checks::record(&context, &file, now())
            .unwrap_err()
            .to_string();
        assert!(error.contains(problem), "{error}");
    }
    assert_eq!(context.store.check_events().unwrap(), Vec::new());
}

/// A database with one work, a notice, one run, and one check, and the
/// archive of that database.
fn archived() -> (tempfile::TempDir, tempfile::TempDir) {
    let data = tempfile::tempdir().unwrap();
    let context = context(data.path());
    let work = catalogue::search(&context, TOPIC, 5, now()).unwrap()[0].work;
    fetch::fetch(&context, work, now()).unwrap();
    let notice = Notice {
        kind: NoticeKind::Correction,
        reported_type: "correction".to_owned(),
        source: NoticeSource::Crossref,
        notice_doi: Some("10.1000/notice.1".parse().unwrap()),
        notice_pmid: None,
        notice_citation: None,
        date: Some(jiff::civil::date(2026, 1, 5)),
        source_record: Some(r#"{"type":"correction"}"#.to_owned()),
        retrieved_at: now(),
    };
    context.store.add_notice(work, &notice).unwrap();
    let path = data.path().join("submission.json");
    fs::write(
        &path,
        serde_json::to_vec_pretty(&submission(&context, "7.30")).unwrap(),
    )
    .unwrap();
    let Outcome::Stored { reports, .. } = submit::submit(&context, &path, false, now()).unwrap()
    else {
        panic!("the submission was not stored");
    };
    let later = "2026-11-14T11:00:00Z".parse().unwrap();
    submit::record_check(
        &context,
        reports[0].0,
        CheckState::Confirmed,
        Some("The value agrees with Table 1."),
        later,
    )
    .unwrap();
    let folder = tempfile::tempdir().unwrap();
    let written = archive::write(&context, folder.path()).unwrap();
    assert_eq!((written.works, written.runs), (1, 1));
    (data, folder)
}

#[test]
fn rebuilds_the_same_database_from_its_archive() {
    let (data, folder) = archived();
    let files = files_in(folder.path());
    assert!(
        files.contains_key("works/PMC6274071/work.json"),
        "{:?}",
        files.keys()
    );
    assert!(files.contains_key("works/PMC6274071/run-1.json"));
    assert_eq!(
        files["works/PMC6274071/run-1.json"],
        fs::read(data.path().join("submission.json")).unwrap()
    );

    let rebuilt_data = tempfile::tempdir().unwrap();
    let rebuilt = Context::open(rebuilt_data.path(), Box::new(fake())).unwrap();
    assert_eq!(
        archive::rebuild(&rebuilt, folder.path()).unwrap(),
        archive::Rebuilt {
            works: 1,
            files: 3,
            runs: 1,
            checks: 1
        }
    );
    let work = rebuilt.work("PMC6274071").unwrap();
    assert_eq!(
        rebuilt.store.work(work).unwrap().state,
        Some(arity_store::QueueState::Read)
    );
    let run = rebuilt.store.runs().unwrap()[0].id;
    let report = rebuilt.store.reports_of_run(run).unwrap()[0].0;
    assert_eq!(
        rebuilt.store.check_state(report).unwrap(),
        CheckState::Confirmed
    );

    let again = tempfile::tempdir().unwrap();
    archive::write(&rebuilt, again.path()).unwrap();
    assert_eq!(files_in(again.path()), files);
}

#[test]
fn refuses_a_file_that_is_not_the_archived_file() {
    let (_data, folder) = archived();
    let mut changed = fake();
    changed.0.insert(
        format!("{}/PMC6274071.1/PMC6274071.1.pdf", pmc::BUCKET),
        b"%PDF-1.4 another article".to_vec(),
    );
    let rebuilt_data = tempfile::tempdir().unwrap();
    let rebuilt = Context::open(rebuilt_data.path(), Box::new(changed)).unwrap();
    let error = archive::rebuild(&rebuilt, folder.path()).unwrap_err();
    assert!(error.to_string().contains("PMC6274071.1.pdf"), "{error}");
}

#[test]
fn refuses_a_submission_that_is_not_the_archived_submission() {
    let (_data, folder) = archived();
    let run = folder.path().join("works/PMC6274071/run-1.json");
    let mut bytes = fs::read(&run).unwrap();
    bytes.push(b'\n');
    fs::write(&run, bytes).unwrap();
    let rebuilt_data = tempfile::tempdir().unwrap();
    let rebuilt = Context::open(rebuilt_data.path(), Box::new(fake())).unwrap();
    let error = archive::rebuild(&rebuilt, folder.path()).unwrap_err();
    assert!(error.to_string().contains("run-1.json"), "{error}");
}

#[test]
fn rebuilds_a_correction_with_the_numbers_of_its_runs() {
    let data = tempfile::tempdir().unwrap();
    let context = context(data.path());
    let work = catalogue::search(&context, TOPIC, 5, now()).unwrap()[0].work;
    fetch::fetch(&context, work, now()).unwrap();
    let mut correction = submission(&context, "7.30");
    correction["replaces"] = serde_json::json!(1);
    correction["note"] = serde_json::json!("The reader corrected the conditions.");
    for (name, value) in [
        ("first.json", submission(&context, "7.30")),
        ("second.json", correction),
    ] {
        let path = data.path().join(name);
        fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
        submit::submit(&context, &path, false, now()).unwrap();
    }
    let folder = tempfile::tempdir().unwrap();
    archive::write(&context, folder.path()).unwrap();

    let rebuilt_data = tempfile::tempdir().unwrap();
    let rebuilt = Context::open(rebuilt_data.path(), Box::new(fake())).unwrap();
    archive::rebuild(&rebuilt, folder.path()).unwrap();
    let runs = rebuilt.store.run_views().unwrap();
    let links: Vec<_> = runs
        .iter()
        .map(|run| (run.id, run.replaces, run.replaced_by))
        .collect();
    assert_eq!(links, [(1, None, Some(2)), (2, Some(1), None)]);
}

#[test]
fn refuses_an_archive_whose_run_numbers_have_a_gap() {
    let (_data, folder) = archived();
    let directory = folder.path().join("works/PMC6274071");
    fs::rename(directory.join("run-1.json"), directory.join("run-2.json")).unwrap();
    let path = directory.join("work.json");
    let mut file: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    file["runs"][0]["sequence"] = serde_json::json!(2);
    fs::write(&path, serde_json::to_vec_pretty(&file).unwrap()).unwrap();
    let rebuilt_data = tempfile::tempdir().unwrap();
    let rebuilt = Context::open(rebuilt_data.path(), Box::new(fake())).unwrap();
    let error = archive::rebuild(&rebuilt, folder.path()).unwrap_err();
    assert!(
        error.to_string().contains("run 2 of the archive"),
        "{error}"
    );
}

#[test]
fn refuses_an_archive_that_names_an_unknown_parser() {
    let (_data, folder) = archived();
    let path = folder.path().join("works/PMC6274071/work.json");
    let mut file: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    file["texts"][0]["parser"] = serde_json::json!("arity-jats 9.9.9");
    fs::write(&path, serde_json::to_vec_pretty(&file).unwrap()).unwrap();
    let rebuilt_data = tempfile::tempdir().unwrap();
    let rebuilt = Context::open(rebuilt_data.path(), Box::new(fake())).unwrap();
    let error = archive::rebuild(&rebuilt, folder.path()).unwrap_err();
    assert!(error.to_string().contains("arity-jats 9.9.9"), "{error}");
}

#[test]
fn rebuilds_only_into_an_empty_database() {
    let (data, folder) = archived();
    let context = Context::open(data.path(), Box::new(fake())).unwrap();
    let error = archive::rebuild(&context, folder.path()).unwrap_err();
    assert!(error.to_string().contains("empty"), "{error}");
}

#[test]
fn records_a_failed_fetch_in_the_queue() {
    let directory = tempfile::tempdir().unwrap();
    let context = Context::open(directory.path(), Box::new(Fake(HashMap::new()))).unwrap();
    let work = context
        .store
        .upsert_work(
            &arity_core::WorkMetadata {
                title: "No files".to_owned(),
                pmcid: Some("PMC1".parse().unwrap()),
                ..arity_core::WorkMetadata::default()
            },
            now(),
        )
        .unwrap();
    assert!(fetch::fetch(&context, work, now()).is_err());
    assert_eq!(
        context.store.work(work).unwrap().state,
        Some(arity_store::QueueState::Failed)
    );
}
