use std::fs;
use std::path::Path;

use arity_core::vocabulary::CheckState;
use arity_protocol::{Issue, Submission, check};
use arity_store::{ReportId, RunId};
use jiff::Timestamp;

use crate::context::Context;
use crate::error::{CliError, Result, io};

#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    /// The submission passed the checks and is in the database.
    Stored {
        run: RunId,
        reports: Vec<(ReportId, String)>,
        warnings: Vec<Issue>,
    },
    /// The submission passed the checks. A dry run does not store it.
    Passed {
        warnings: Vec<Issue>,
    },
    Rejected {
        issues: Vec<Issue>,
    },
}

/// Checks a submission file against the text that it names, and stores it
/// when it passes and the run is not a dry run.
pub fn submit(context: &Context, path: &Path, dry_run: bool, now: Timestamp) -> Result<Outcome> {
    let raw = fs::read(path).map_err(io(path))?;
    let header: Submission = match serde_json::from_slice(&raw) {
        Ok(submission) => submission,
        Err(error) => {
            return Ok(Outcome::Rejected {
                issues: vec![Issue {
                    severity: arity_protocol::Severity::Error,
                    path: String::new(),
                    code: "invalid_json",
                    message: format!("the file is not a valid submission: {error}"),
                }],
            });
        }
    };
    let work = context.work(&header.work)?;
    let named = context
        .store
        .text_by_digest(&header.text)?
        .map(|id| {
            context
                .store
                .text(id)
                .map(|(owner, document)| (id, owner, document))
        })
        .transpose()?
        .filter(|(_, owner, _)| *owner == work);
    let (text, document) = match named {
        Some((id, _, document)) => (id, document),
        None => context.store.latest_text(work)?.ok_or_else(|| {
            CliError::Usage(format!(
                "the work {} has no text yet; run `arity fetch {}` first",
                header.work, header.work
            ))
        })?,
    };
    let properties = context.store.properties()?;
    let checked = match check(&raw, &document, &properties) {
        Ok(checked) => checked,
        Err(rejected) => {
            return Ok(Outcome::Rejected {
                issues: rejected.issues,
            });
        }
    };
    if dry_run {
        return Ok(Outcome::Passed {
            warnings: checked.warnings,
        });
    }
    let run = context
        .store
        .store_submission(work, text, &raw, &checked, now)?;
    context
        .store
        .set_state(work, arity_store::QueueState::Read, None, now)
        .or_else(|error| match error {
            arity_store::StoreError::NotFound(_) => Ok(()),
            other => Err(other),
        })?;
    Ok(Outcome::Stored {
        run,
        reports: context.store.reports_of_run(run)?,
        warnings: checked.warnings,
    })
}

/// Records the check of a report by the person that the configuration
/// names.
pub fn record_check(
    context: &Context,
    report: ReportId,
    state: CheckState,
    note: Option<&str>,
    now: Timestamp,
) -> Result<()> {
    let person = context.config.person()?;
    context
        .store
        .record_check(report, state, &person, note, now)?;
    Ok(())
}
