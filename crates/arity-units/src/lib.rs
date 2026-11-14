//! Units of measurement for arity.
//!
//! The parser reads UCUM codes (the Unified Code for Units of Measure). Each
//! unit has a dimension and a scale to the coherent SI unit. arity uses the
//! dimensions to check that a value agrees with its property, and it uses the
//! scales to convert values for comparison.

mod dimension;
mod table;
mod unit;

pub use dimension::{Dimension, DimensionError};
pub use table::{Atom, Prefix, prefixes, symbols};
pub use unit::{Unit, UnitError};
