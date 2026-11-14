//! The commands of arity, as a library. The `arity` program calls these
//! functions, and the tests call them with a fake network.

pub mod archive;
pub mod catalogue;
pub mod config;
pub mod context;
pub mod coverage;
pub mod error;
pub mod fetch;
pub mod notices;
pub mod show;
pub mod submit;

pub use context::Context;
pub use error::{CliError, Result};
