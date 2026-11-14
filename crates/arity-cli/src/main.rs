use std::fmt::Display;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use arity_cli::config::{CONFIG_FILE, Config, PersonConfig};
use arity_cli::submit::Outcome;
use arity_cli::{CliError, Context, Result, catalogue, fetch, notices, show, submit};
use arity_core::vocabulary::CheckState;
use arity_core::{EntityKind, License, NoticeKind, Orcid, PREFIXES, Scope};
use arity_sources::{Client, USER_AGENT};
use arity_store::{QueueState, ReportId, WorkId, WorkRecord};
use clap::{Parser, Subcommand, ValueEnum};
use jiff::Timestamp;

/// Collect the values and effects that open-access research articles report,
/// each with its source text and its provenance.
#[derive(Debug, Parser)]
#[command(name = "arity", version)]
struct Cli {
    /// The data folder.
    #[arg(long, env = "ARITY_DATA", default_value = "data", global = true)]
    data: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Make the data folder and record the person who works with it.
    Init {
        /// Your name. arity records it with each check that you make.
        #[arg(long)]
        name: String,
        #[arg(long)]
        orcid: Option<Orcid>,
    },
    /// Find open-access works for a topic and add them to the queue.
    Search {
        /// A Europe PMC query, for example `doxorubicin AND "MCF-7" AND IC50`.
        topic: String,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// Add works to the queue by PMCID, DOI, or PMID.
    Add {
        #[arg(required = true)]
        identifiers: Vec<String>,
    },
    /// Show the works and their state in the queue.
    Queue,
    /// Download the full text, the PDF, and the supplementary files of works.
    Fetch {
        identifiers: Vec<String>,
        /// Fetch this number of works from the front of the queue.
        #[arg(long)]
        next: Option<usize>,
    },
    /// Print the canonical text of a work, with the key of each block.
    Show {
        identifier: String,
        /// Print the canonical text as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Check a submission file, and store it when it passes.
    Submit {
        file: PathBuf,
        /// Do the checks, but do not store the submission.
        #[arg(long)]
        dry_run: bool,
    },
    /// Record that you compared a report with its source.
    Check {
        report: i64,
        #[arg(value_enum)]
        verdict: Verdict,
        #[arg(long)]
        note: Option<String>,
    },
    /// Read the notices about works again: retractions, corrections, and others.
    Notices { identifiers: Vec<String> },
    /// List the terms of a vocabulary.
    Vocab {
        #[arg(value_enum)]
        list: VocabList,
    },
    /// Print the JSON Schema of the submission format.
    Schema,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Verdict {
    /// The report agrees with its source.
    Confirm,
    /// The report does not agree with its source.
    Reject,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum VocabList {
    Kinds,
    Prefixes,
    Units,
    Properties,
    Notices,
}

fn emit(out: &mut impl Write, text: impl Display) -> Result<()> {
    writeln!(out, "{text}").map_err(|source| CliError::Io {
        path: PathBuf::from("<stdout>"),
        source,
    })
}

fn open(data: &Path) -> Result<Context> {
    let client = Client::new(USER_AGENT, Duration::from_millis(250));
    Context::open(data, Box::new(client))
}

fn identifier_of(work: &WorkRecord) -> String {
    work.metadata
        .pmcid
        .as_ref()
        .map(ToString::to_string)
        .or_else(|| work.metadata.doi.as_ref().map(ToString::to_string))
        .or_else(|| work.metadata.pmid.as_ref().map(ToString::to_string))
        .unwrap_or_else(|| format!("work:{}", work.id.0))
}

fn json_text(value: &impl serde::Serialize) -> Result<String> {
    serde_json::to_string_pretty(value).map_err(|error| CliError::Usage(error.to_string()))
}

fn run(cli: Cli, out: &mut impl Write) -> Result<ExitCode> {
    let now = Timestamp::now();
    let data = cli.data.as_path();
    match cli.command {
        Command::Init { name, orcid } => init(data, name, orcid, out),
        Command::Search { topic, limit } => search(data, &topic, limit, now, out),
        Command::Add { identifiers } => add(data, &identifiers, now, out),
        Command::Queue => queue(data, out),
        Command::Fetch { identifiers, next } => fetch_works(data, &identifiers, next, now, out),
        Command::Show { identifier, json } => show_work(data, &identifier, json, out),
        Command::Submit { file, dry_run } => submit_file(data, &file, dry_run, now, out),
        Command::Check {
            report,
            verdict,
            note,
        } => check(data, report, verdict, note.as_deref(), now, out),
        Command::Notices { identifiers } => refresh_notices(data, &identifiers, now, out),
        Command::Vocab { list } => vocab(list, data, out),
        Command::Schema => {
            emit(out, json_text(&arity_protocol::schema())?)?;
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn init(data: &Path, name: String, orcid: Option<Orcid>, out: &mut impl Write) -> Result<ExitCode> {
    let context = open(data)?;
    let config = Config {
        person: Some(PersonConfig { name, orcid }),
        ..context.config.clone()
    };
    config.save(data)?;
    emit(
        out,
        format!(
            "The data folder {} is ready. The person is in {CONFIG_FILE}.",
            data.display()
        ),
    )?;
    Ok(ExitCode::SUCCESS)
}

fn emit_added(out: &mut impl Write, added: &catalogue::Added, name: &str) -> Result<()> {
    let mark = if added.new { "+" } else { "=" };
    emit(out, format!("{mark} {name} {}", added.title))
}

fn search(
    data: &Path,
    topic: &str,
    limit: usize,
    now: Timestamp,
    out: &mut impl Write,
) -> Result<ExitCode> {
    let context = open(data)?;
    for added in catalogue::search(&context, topic, limit, now)? {
        let name = added
            .pmcid
            .as_ref()
            .map_or_else(|| format!("work:{}", added.work.0), ToString::to_string);
        emit_added(out, &added, &name)?;
    }
    Ok(ExitCode::SUCCESS)
}

fn add(
    data: &Path,
    identifiers: &[String],
    now: Timestamp,
    out: &mut impl Write,
) -> Result<ExitCode> {
    let context = open(data)?;
    for identifier in identifiers {
        let added = catalogue::add(&context, identifier, now)?;
        emit_added(out, &added, identifier)?;
    }
    Ok(ExitCode::SUCCESS)
}

fn queue(data: &Path, out: &mut impl Write) -> Result<ExitCode> {
    let context = open(data)?;
    for work in context.store.works()? {
        let state = work.state.map_or("-", QueueState::as_str);
        let year = work
            .metadata
            .year
            .map_or_else(|| "----".to_owned(), |year| year.to_string());
        let name = identifier_of(&work);
        emit(
            out,
            format!("{state:<8} {name:<14} {year} {}", work.metadata.title),
        )?;
    }
    Ok(ExitCode::SUCCESS)
}

fn fetch_works(
    data: &Path,
    identifiers: &[String],
    next: Option<usize>,
    now: Timestamp,
    out: &mut impl Write,
) -> Result<ExitCode> {
    let context = open(data)?;
    let mut works: Vec<WorkId> = identifiers
        .iter()
        .map(|identifier| context.work(identifier))
        .collect::<Result<_>>()?;
    if let Some(count) = next {
        works.extend(context.store.in_state(QueueState::Queued, count)?);
    }
    if works.is_empty() {
        return Err(CliError::Usage(
            "name the works to fetch, or give --next".to_owned(),
        ));
    }
    let mut code = ExitCode::SUCCESS;
    for work in works {
        match fetch::fetch(&context, work, now) {
            Ok(fetched) => {
                let license = fetched
                    .license
                    .as_ref()
                    .map_or_else(|| "no license".to_owned(), License::label);
                let pdf = if fetched.pdf { "yes" } else { "no" };
                emit(
                    out,
                    format!(
                        "{}: {} blocks, PDF {pdf}, {} supplementary files, {license}",
                        fetched.version, fetched.blocks, fetched.supplements
                    ),
                )?;
            }
            Err(error) => {
                code = ExitCode::FAILURE;
                emit(out, format!("work {}: {error}", work.0))?;
            }
        }
    }
    Ok(code)
}

fn show_work(data: &Path, identifier: &str, json: bool, out: &mut impl Write) -> Result<ExitCode> {
    let context = open(data)?;
    let work = context.work(identifier)?;
    let (_, document) = context.store.latest_text(work)?.ok_or_else(|| {
        CliError::Usage(format!(
            "{identifier} has no text yet; run `arity fetch {identifier}`"
        ))
    })?;
    if json {
        emit(out, json_text(&document)?)?;
    } else {
        let metadata = context.store.work(work)?.metadata;
        let digest = document
            .digest()
            .map_err(|error| CliError::Usage(error.to_string()))?;
        emit(out, show::render(&metadata, &digest, &document))?;
    }
    Ok(ExitCode::SUCCESS)
}

fn submit_file(
    data: &Path,
    file: &Path,
    dry_run: bool,
    now: Timestamp,
    out: &mut impl Write,
) -> Result<ExitCode> {
    let context = open(data)?;
    match submit::submit(&context, file, dry_run, now)? {
        Outcome::Stored {
            run,
            reports,
            warnings,
        } => {
            for warning in &warnings {
                emit(out, warning)?;
            }
            emit(
                out,
                format!("Stored as run {} with {} reports:", run.0, reports.len()),
            )?;
            for (report, local) in reports {
                emit(out, format!("  {local} -> report {}", report.0))?;
            }
            Ok(ExitCode::SUCCESS)
        }
        Outcome::Passed { warnings } => {
            for warning in &warnings {
                emit(out, warning)?;
            }
            emit(
                out,
                "The submission passed the checks. The dry run did not store it.",
            )?;
            Ok(ExitCode::SUCCESS)
        }
        Outcome::Rejected { issues } => {
            for issue in &issues {
                emit(out, issue)?;
            }
            emit(
                out,
                "The submission did not pass the checks. Correct the errors and submit again.",
            )?;
            Ok(ExitCode::FAILURE)
        }
    }
}

fn check(
    data: &Path,
    report: i64,
    verdict: Verdict,
    note: Option<&str>,
    now: Timestamp,
    out: &mut impl Write,
) -> Result<ExitCode> {
    let context = open(data)?;
    let state = match verdict {
        Verdict::Confirm => CheckState::Confirmed,
        Verdict::Reject => CheckState::Rejected,
    };
    submit::record_check(&context, ReportId(report), state, note, now)?;
    emit(out, format!("Report {report} is {state}."))?;
    Ok(ExitCode::SUCCESS)
}

fn refresh_notices(
    data: &Path,
    identifiers: &[String],
    now: Timestamp,
    out: &mut impl Write,
) -> Result<ExitCode> {
    let context = open(data)?;
    let works: Vec<WorkId> = if identifiers.is_empty() {
        context
            .store
            .works()?
            .into_iter()
            .map(|work| work.id)
            .collect()
    } else {
        identifiers
            .iter()
            .map(|identifier| context.work(identifier))
            .collect::<Result<_>>()?
    };
    for work in works {
        for notice in notices::refresh(&context, work, now)? {
            emit(
                out,
                format!(
                    "work {}: {} from {} ({})",
                    work.0,
                    notice.kind.label(),
                    notice.source.label(),
                    notice.reported_type
                ),
            )?;
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn vocab(list: VocabList, data: &Path, out: &mut impl Write) -> Result<ExitCode> {
    match list {
        VocabList::Kinds => {
            for kind in EntityKind::ALL {
                emit(out, format!("{:<18} {}", kind.as_str(), kind.description()))?;
            }
        }
        VocabList::Prefixes => {
            for prefix in PREFIXES {
                let scope = match prefix.scope {
                    Scope::Entities(kinds) => kinds
                        .iter()
                        .map(|kind| kind.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                    Scope::AnyEntity => "any entity".to_owned(),
                    Scope::Terms => "methods and properties".to_owned(),
                };
                emit(
                    out,
                    format!("{:<17} {} ({scope})", prefix.name, prefix.title),
                )?;
            }
        }
        VocabList::Units => {
            for symbol in arity_units::symbols() {
                let marks = [
                    symbol.is_metric().then_some("takes prefixes"),
                    symbol.is_arbitrary().then_some("arbitrary"),
                    symbol.is_extension().then_some("not in UCUM"),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(", ");
                let dimension = symbol.dimension().to_string();
                emit(
                    out,
                    format!(
                        "{:<10} {:<34} {dimension:<16} {marks}",
                        symbol.code(),
                        symbol.name()
                    ),
                )?;
            }
        }
        VocabList::Properties => {
            let context = open(data)?;
            for property in context.store.properties()? {
                let unit = property.unit.as_deref().unwrap_or("-");
                emit(
                    out,
                    format!(
                        "{:<24} {:<9} {unit:<12} {}",
                        property.key, property.value_type, property.label
                    ),
                )?;
            }
        }
        VocabList::Notices => {
            for kind in NoticeKind::ALL {
                emit(out, format!("{:<22} {}", kind.as_str(), kind.label()))?;
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let mut out = io::stdout().lock();
    match run(cli, &mut out) {
        Ok(code) => code,
        Err(error) => {
            let _ = writeln!(io::stderr(), "arity: {error}");
            ExitCode::FAILURE
        }
    }
}
