//! The SQLite database and the file store of arity.
//!
//! The database holds the catalogue of works, the canonical texts, the
//! notices about each work, and the reports with their full provenance. The
//! file store holds each downloaded file, under the SHA-256 digest of its
//! content.

mod blobs;
mod store;

pub use blobs::{BlobError, Blobs};
pub use store::{
    AgentId, EntityId, QueueState, ReportId, Result, Retrieval, RetrievalId, RetrievalRole, RunId,
    SOFTWARE, Store, StoreError, TextId, WorkId, WorkRecord,
};
