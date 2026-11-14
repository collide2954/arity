//! Core types that all arity crates use.

mod curie;
mod decimal;
mod entity;
mod id;
mod license;

pub use curie::{Curie, CurieError, PREFIXES, Prefix, Scope};
pub use decimal::{Decimal, DecimalError};
pub use entity::{EntityKind, UnknownEntityKind};
pub use id::{Doi, IdError, Orcid, Pmcid, Pmid};
pub use license::{CcElements, EmptyLicense, License};
