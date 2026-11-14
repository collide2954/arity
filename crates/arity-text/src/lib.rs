//! Source text for arity.
//!
//! This crate holds the rules that decide when a quote occurs in a source and
//! when a number occurs in a quote. The website uses the same rules, and the
//! shared test vectors in `testdata/text` keep the two implementations equal.

mod document;
mod normalize;
mod numbers;

pub use document::{Block, BlockKind, Cell, DOCUMENT_FORMAT, Document, DocumentError};
pub use normalize::{Normalized, normalize};
pub use numbers::{NumberMatch, contains_number, find_numbers};
