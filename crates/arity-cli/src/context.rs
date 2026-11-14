use std::fs;
use std::path::{Path, PathBuf};

use arity_sources::Http;
use arity_store::{Blobs, Store, WorkId};

use crate::config::Config;
use crate::error::{CliError, Result, io};

pub const DATABASE_FILE: &str = "arity.sqlite";
pub const BLOB_FOLDER: &str = "blobs";

/// A data folder: the database, the file store, and the settings.
pub struct Context {
    pub directory: PathBuf,
    pub store: Store,
    pub blobs: Blobs,
    pub http: Box<dyn Http>,
    pub config: Config,
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("directory", &self.directory)
            .finish_non_exhaustive()
    }
}

impl Context {
    pub fn open(directory: &Path, http: Box<dyn Http>) -> Result<Self> {
        fs::create_dir_all(directory).map_err(io(directory))?;
        Ok(Self {
            directory: directory.to_path_buf(),
            store: Store::open(&directory.join(DATABASE_FILE))?,
            blobs: Blobs::new(directory.join(BLOB_FOLDER)),
            http,
            config: Config::load(directory)?,
        })
    }

    /// Finds a work by a PMCID, a DOI, or a PMID.
    pub fn work(&self, identifier: &str) -> Result<WorkId> {
        self.store.find_work(identifier)?.ok_or_else(|| {
            CliError::Usage(format!(
                "the database has no work {identifier}; add it with `arity add {identifier}`"
            ))
        })
    }
}
