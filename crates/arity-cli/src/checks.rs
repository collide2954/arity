//! The check files that the website makes: the checks of reports by one
//! person. arity records each check with the person that the file names.

use std::fs;
use std::path::Path;

use arity_core::vocabulary::CheckState;
use arity_protocol::Person;
use arity_store::{ReportId, RunId};
use jiff::Timestamp;
use serde::Deserialize;

use crate::context::Context;
use crate::error::{CliError, Result, io};

/// The format of a check file.
pub const FORMAT: &str = "arity-checks/1";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckFile {
    format: String,
    person: Person,
    checks: Vec<FileCheck>,
}

/// The check of one report, and when the person made it.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileCheck {
    /// The PMCID, DOI, or PMID of the work of the report.
    work: String,
    /// The run of the report.
    run: i64,
    /// The identifier of the report in its submission.
    report: String,
    state: CheckState,
    note: Option<String>,
    at: Timestamp,
}

/// Records the checks of a check file with the person that the file names.
/// arity finds the report of each check before it records a check, so a
/// file with a problem adds no checks. A check that the database has already
/// is not recorded again. Gives the number of new checks.
pub fn record(context: &Context, raw: &[u8], now: Timestamp) -> Result<usize> {
    let file: CheckFile = serde_json::from_slice(raw)
        .map_err(|error| CliError::Usage(format!("this is not a check file: {error}")))?;
    if file.format != FORMAT {
        return Err(CliError::Usage(format!(
            "this is not a check file of the format {FORMAT}"
        )));
    }
    if file.person.name.trim().is_empty() {
        return Err(CliError::Usage(
            "a check file must give the name of the person".to_owned(),
        ));
    }
    let known = context.store.check_events()?;
    let mut new: Vec<(ReportId, &FileCheck)> = Vec::new();
    for (position, check) in file.checks.iter().enumerate() {
        let report = report_of(context, check, now)
            .map_err(|problem| CliError::Usage(format!("check {}: {problem}", position + 1)))?;
        let recorded = known.iter().any(|event| {
            event.run == RunId(check.run)
                && event.report == check.report
                && event.state == check.state
                && event.person == file.person
                && event.note == check.note
                && event.at == check.at
        });
        if !recorded {
            new.push((report, check));
        }
    }
    for (report, check) in &new {
        context.store.record_check(
            *report,
            check.state,
            &file.person,
            check.note.as_deref(),
            check.at,
        )?;
    }
    Ok(new.len())
}

/// Records the checks of the check file at a path. See [`record`].
pub fn record_file(context: &Context, path: &Path, now: Timestamp) -> Result<usize> {
    record(context, &fs::read(path).map_err(io(path))?, now)
}

/// The report of a check, or the problem of the check.
fn report_of(
    context: &Context,
    check: &FileCheck,
    now: Timestamp,
) -> std::result::Result<ReportId, String> {
    if check.state == CheckState::Unchecked {
        return Err("a check must confirm or reject its report".to_owned());
    }
    if check.at > now {
        return Err(format!("the time {} is later than now", check.at));
    }
    let store = &context.store;
    let work = store
        .find_work(&check.work)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("arity has no work {}", check.work))?;
    let run = RunId(check.run);
    if store.work_of_run(run).map_err(|error| error.to_string())? != Some(work) {
        return Err(format!("the work {} has no run {}", check.work, check.run));
    }
    store
        .reports_of_run(run)
        .map_err(|error| error.to_string())?
        .into_iter()
        .find(|(_, local)| *local == check.report)
        .map(|(id, _)| id)
        .ok_or_else(|| format!("run {} has no report {}", check.run, check.report))
}
