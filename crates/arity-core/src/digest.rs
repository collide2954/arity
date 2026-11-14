use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

/// A SHA-256 digest in lower-case hexadecimal form.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Digest(String);

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0:?} is not a SHA-256 digest of 64 hexadecimal digits")]
pub struct InvalidDigest(pub String);

impl Digest {
    pub fn of(bytes: &[u8]) -> Self {
        Self(hex::encode(Sha256::digest(bytes)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for Digest {
    type Err = InvalidDigest;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let lower = text.trim().to_ascii_lowercase();
        if lower.len() == 64 && lower.bytes().all(|b| b.is_ascii_hexdigit()) {
            Ok(Self(lower))
        } else {
            Err(InvalidDigest(text.to_owned()))
        }
    }
}

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for Digest {
    type Error = InvalidDigest;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        text.parse()
    }
}

impl From<Digest> for String {
    fn from(digest: Digest) -> Self {
        digest.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_bytes() {
        assert_eq!(
            Digest::of(b"abc").as_str(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn parses_and_normalizes_hexadecimal_text() {
        let upper = "BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD";
        assert_eq!(upper.parse::<Digest>().unwrap(), Digest::of(b"abc"));
        for text in ["", "abc", &"g".repeat(64), &"a".repeat(63)] {
            assert!(text.parse::<Digest>().is_err(), "{text}");
        }
    }

    #[test]
    fn serializes_as_a_string() {
        let json = serde_json::to_string(&Digest::of(b"")).unwrap();
        assert_eq!(
            json,
            "\"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\""
        );
    }
}
