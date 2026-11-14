//! Core types that all arity crates use.

mod decimal;
mod id;

pub use decimal::{Decimal, DecimalError};
pub use id::{Doi, IdError, Orcid, Pmcid, Pmid};
