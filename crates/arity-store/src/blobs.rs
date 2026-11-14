use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use arity_core::Digest;

/// Files that arity downloads, stored by the SHA-256 digest of their
/// content. A file never changes after it is stored.
#[derive(Debug, Clone)]
pub struct Blobs {
    root: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum BlobError {
    #[error("cannot use the file store: {0}")]
    Io(#[from] io::Error),
    #[error("the stored file {expected} has the digest {actual}")]
    Corrupt { expected: Digest, actual: Digest },
}

impl Blobs {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn path(&self, digest: &Digest) -> PathBuf {
        let text = digest.as_str();
        self.root.join(&text[..2]).join(text)
    }

    pub fn put(&self, bytes: &[u8]) -> Result<Digest, BlobError> {
        let digest = Digest::of(bytes);
        let path = self.path(&digest);
        if path.is_file() {
            return Ok(digest);
        }
        let directory = path.parent().unwrap_or(&self.root);
        fs::create_dir_all(directory)?;
        let mut file = tempfile::NamedTempFile::new_in(directory)?;
        file.write_all(bytes)?;
        file.as_file().sync_all()?;
        file.persist(&path).map_err(|error| error.error)?;
        Ok(digest)
    }

    /// Reads a stored file and checks that its content has the digest.
    pub fn get(&self, digest: &Digest) -> Result<Vec<u8>, BlobError> {
        let bytes = fs::read(self.path(digest))?;
        let actual = Digest::of(&bytes);
        if actual == *digest {
            Ok(bytes)
        } else {
            Err(BlobError::Corrupt {
                expected: digest.clone(),
                actual,
            })
        }
    }

    pub fn contains(&self, digest: &Digest) -> bool {
        self.path(digest).is_file()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_and_reads_files_by_digest() {
        let directory = tempfile::tempdir().unwrap();
        let blobs = Blobs::new(directory.path());
        let digest = blobs.put(b"abc").unwrap();
        assert_eq!(digest, Digest::of(b"abc"));
        assert!(blobs.contains(&digest));
        assert_eq!(blobs.get(&digest).unwrap(), b"abc");
        assert!(
            blobs.path(&digest).ends_with(
                std::path::Path::new("ba")
                    .join("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
            )
        );
        assert_eq!(blobs.put(b"abc").unwrap(), digest);
    }

    #[test]
    fn finds_a_changed_file() {
        let directory = tempfile::tempdir().unwrap();
        let blobs = Blobs::new(directory.path());
        let digest = blobs.put(b"abc").unwrap();
        fs::write(blobs.path(&digest), b"abd").unwrap();
        assert!(matches!(blobs.get(&digest), Err(BlobError::Corrupt { .. })));
    }

    #[test]
    fn reports_a_missing_file() {
        let directory = tempfile::tempdir().unwrap();
        let blobs = Blobs::new(directory.path());
        assert!(matches!(
            blobs.get(&Digest::of(b"missing")),
            Err(BlobError::Io(_))
        ));
    }
}
