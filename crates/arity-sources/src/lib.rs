//! Clients for the services that arity reads.
//!
//! - Europe PMC: the catalogue search, and the links to notices.
//! - The PMC open-access store: the full text, the PDF, and the media files.
//! - Crossref: the notices that publishers and Retraction Watch record.
//!
//! The clients only report what each service states. Each notice keeps the
//! name of its source and the type name that the source uses.

pub mod crossref;
pub mod europepmc;
mod http;
pub mod pmc;

use regex::Regex;

pub use http::{Client, Http, Response, SourceError};

/// The user agent that arity sends.
pub const USER_AGENT: &str = concat!(
    "arity/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/collide2954/arity)"
);

#[expect(
    clippy::expect_used,
    reason = "each pattern is a constant, and the tests compile all of them"
)]
fn compile(pattern: &str) -> Regex {
    Regex::new(pattern).expect("a constant pattern compiles")
}
