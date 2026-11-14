//! Core types that all arity crates use.

mod curie;
mod decimal;
mod entity;
mod id;
mod license;
mod notice;

pub use curie::{Curie, CurieError, PREFIXES, Prefix, Scope};
pub use decimal::{Decimal, DecimalError};
pub use entity::{EntityKind, UnknownEntityKind};
pub use id::{Doi, IdError, Orcid, Pmcid, Pmid};
pub use license::{CcElements, EmptyLicense, License};
pub use notice::{Notice, NoticeKind, NoticeSource, UnknownNoticeKind};
