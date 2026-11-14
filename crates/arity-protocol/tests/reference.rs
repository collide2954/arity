#![cfg(test)]

use std::fmt::Write;
use std::path::PathBuf;

use arity_core::{EntityKind, NoticeKind, PREFIXES, Scope};
use arity_protocol::IssueCode;

const NOTE: &str = "<!-- The tests of arity-protocol write this file. After a change, run them with ARITY_UPDATE_DOCS=1. -->";

fn page(title: &str, lede: &str, header: &str, rows: impl Iterator<Item = String>) -> String {
    let columns = header.matches('|').count() - 1;
    let mut text = format!("# {title}\n\n{NOTE}\n\n{lede}\n\n{header}\n|");
    text.push_str(&" --- |".repeat(columns));
    text.push('\n');
    for row in rows {
        writeln!(text, "{row}").unwrap();
    }
    text
}

fn issues() -> String {
    page(
        "Issue codes",
        "The checks give each problem one of these codes. An error stops the submission. A warning lets the submission go into the database, and a person must look at the part that the warning names. `arity vocab issues` prints this list.",
        "| Code | Severity | Meaning |",
        IssueCode::ALL.iter().map(|code| {
            format!(
                "| `{}` | {} | {} |",
                code.as_str(),
                code.severity().as_str(),
                code.description()
            )
        }),
    )
}

fn kinds() -> String {
    page(
        "Entity kinds",
        "Each entity has one of these kinds. `arity vocab kinds` prints this list.",
        "| Kind | Meaning |",
        EntityKind::ALL
            .iter()
            .map(|kind| format!("| `{}` | {} |", kind.as_str(), kind.description())),
    )
}

fn registries() -> String {
    page(
        "Registries",
        "An identifier of an entity, a method, or a property has the form `PREFIX:ID`, for example `CHEBI:28748`. arity accepts these prefixes. `arity vocab prefixes` prints this list.",
        "| Prefix | Registry | Identifies |",
        PREFIXES.iter().map(|prefix| {
            let scope = match prefix.scope {
                Scope::Entities(kinds) => kinds
                    .iter()
                    .map(|kind| format!("`{}`", kind.as_str()))
                    .collect::<Vec<_>>()
                    .join(", "),
                Scope::AnyEntity => "entities of any kind".to_owned(),
                Scope::Terms => "methods and properties".to_owned(),
            };
            format!("| `{}` | {} | {scope} |", prefix.name, prefix.title)
        }),
    )
}

fn notices() -> String {
    page(
        "Notice kinds",
        "arity gives each notice one of these kinds, and it keeps the type name that the source uses. `arity vocab notices` prints this list.",
        "| Kind | Name |",
        NoticeKind::ALL
            .iter()
            .map(|kind| format!("| `{}` | {} |", kind.as_str(), kind.label())),
    )
}

fn check(name: &str, current: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/src/reference")
        .join(name);
    if std::env::var_os("ARITY_UPDATE_DOCS").is_some() {
        std::fs::write(&path, current).unwrap();
    }
    let committed = std::fs::read_to_string(&path)
        .unwrap_or_default()
        .replace("\r\n", "\n");
    assert!(
        committed == current,
        "docs/src/reference/{name} is out of date; run the tests with ARITY_UPDATE_DOCS=1"
    );
}

#[test]
fn the_reference_of_the_issue_codes_is_current() {
    check("issues.md", &issues());
}

#[test]
fn the_reference_of_the_entity_kinds_is_current() {
    check("kinds.md", &kinds());
}

#[test]
fn the_reference_of_the_registries_is_current() {
    check("registries.md", &registries());
}

#[test]
fn the_reference_of_the_notice_kinds_is_current() {
    check("notices.md", &notices());
}
