//! Source text for arity.
//!
//! This crate holds the rules that decide when a quote occurs in a source and
//! when a number occurs in a quote. The website uses the same rules, and the
//! shared test vectors in `testdata/text` keep the two implementations equal.

mod normalize;

pub use normalize::{Normalized, normalize};
