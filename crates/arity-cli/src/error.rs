use std::path::PathBuf;

use arity_jats::JatsError;
use arity_sources::SourceError;
use arity_store::{BlobError, StoreError};

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    Blob(#[from] BlobError),
    #[error(transparent)]
    Source(#[from] SourceError),
    #[error("cannot read the article: {0}")]
    Jats(#[from] JatsError),
    #[error("cannot read or write {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot use the configuration: {0}")]
    Config(String),
    #[error("{0}")]
    Usage(String),
    #[error("the submission did not pass the checks")]
    Rejected,
}

pub type Result<T> = std::result::Result<T, CliError>;

pub(crate) fn io(path: impl Into<PathBuf>) -> impl FnOnce(std::io::Error) -> CliError {
    let path = path.into();
    move |source| CliError::Io { path, source }
}
