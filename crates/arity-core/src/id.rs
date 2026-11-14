use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IdError {
    #[error("{kind} {text:?} is not valid")]
    Invalid { kind: &'static str, text: String },
    #[error("ORCID {0:?} has an incorrect check digit")]
    OrcidChecksum(String),
}

fn invalid(kind: &'static str, text: &str) -> IdError {
    IdError::Invalid {
        kind,
        text: text.to_owned(),
    }
}

macro_rules! string_id {
    ($name:ident) => {
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl $name {
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<String> for $name {
            type Error = IdError;

            fn try_from(text: String) -> Result<Self, Self::Error> {
                text.parse()
            }
        }

        impl From<$name> for String {
            fn from(id: $name) -> Self {
                id.0
            }
        }
    };
}

/// A Digital Object Identifier in lower case, without a resolver prefix.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Doi(String);

string_id!(Doi);

impl Doi {
    pub fn url(&self) -> String {
        format!("https://doi.org/{}", self.0)
    }
}

impl FromStr for Doi {
    type Err = IdError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let trimmed = text.trim();
        let lower = trimmed.to_lowercase();
        let bare = [
            "https://doi.org/",
            "http://doi.org/",
            "https://dx.doi.org/",
            "http://dx.doi.org/",
            "doi:",
        ]
        .iter()
        .find_map(|prefix| lower.strip_prefix(prefix))
        .unwrap_or(&lower)
        .trim_start();

        let Some((registrant, suffix)) = bare.split_once('/') else {
            return Err(invalid("DOI", text));
        };
        let Some(code) = registrant.strip_prefix("10.") else {
            return Err(invalid("DOI", text));
        };
        let mut parts = code.split('.');
        let main_ok = parts
            .next()
            .is_some_and(|main| (4..=9).contains(&main.len()) && is_digits(main));
        let sub_ok = parts.all(|part| !part.is_empty() && is_digits(part));
        let suffix_ok = !suffix.is_empty() && !suffix.chars().any(char::is_whitespace);
        if main_ok && sub_ok && suffix_ok {
            Ok(Self(bare.to_owned()))
        } else {
            Err(invalid("DOI", text))
        }
    }
}

/// A PubMed Central identifier, for example `PMC1234567`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Pmcid(String);

string_id!(Pmcid);

impl FromStr for Pmcid {
    type Err = IdError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let trimmed = text.trim();
        let digits = trimmed
            .get(..3)
            .filter(|head| head.eq_ignore_ascii_case("pmc"))
            .and_then(|_| trimmed.get(3..))
            .filter(|digits| is_positive_integer(digits, 10))
            .ok_or_else(|| invalid("PMCID", text))?;
        Ok(Self(format!("PMC{digits}")))
    }
}

/// A PubMed identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Pmid(String);

string_id!(Pmid);

impl FromStr for Pmid {
    type Err = IdError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let trimmed = text.trim();
        if is_positive_integer(trimmed, 10) {
            Ok(Self(trimmed.to_owned()))
        } else {
            Err(invalid("PMID", text))
        }
    }
}

/// An ORCID identifier for a person, for example `0000-0002-1825-0097`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Orcid(String);

string_id!(Orcid);

impl Orcid {
    pub fn url(&self) -> String {
        format!("https://orcid.org/{}", self.0)
    }
}

impl FromStr for Orcid {
    type Err = IdError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let trimmed = text.trim();
        let bare = ["https://orcid.org/", "http://orcid.org/"]
            .iter()
            .find_map(|prefix| trimmed.strip_prefix(prefix))
            .unwrap_or(trimmed);
        let groups: Vec<&str> = bare.split('-').collect();
        let shape_ok = groups.len() == 4 && groups.iter().all(|group| group.len() == 4);
        let compact: String = groups.concat();
        let (body, check) = compact.split_at(compact.len().saturating_sub(1));
        let chars_ok = is_digits(body) && (is_digits(check) || check == "X");
        if !(shape_ok && chars_ok) {
            return Err(invalid("ORCID", text));
        }
        if orcid_check_character(body) == check.chars().next() {
            Ok(Self(bare.to_owned()))
        } else {
            Err(IdError::OrcidChecksum(text.to_owned()))
        }
    }
}

fn orcid_check_character(body: &str) -> Option<char> {
    let total = body
        .chars()
        .filter_map(|c| c.to_digit(10))
        .fold(0, |total, digit| (total + digit) * 2);
    let result = (12 - total % 11) % 11;
    if result == 10 {
        Some('X')
    } else {
        char::from_digit(result, 10)
    }
}

fn is_digits(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit())
}

fn is_positive_integer(text: &str, max_len: usize) -> bool {
    is_digits(text) && text.len() <= max_len && !text.starts_with('0')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doi_strips_resolver_prefixes_and_folds_case() {
        let expected = "10.1371/journal.pone.0123456";
        for text in [
            "10.1371/journal.pone.0123456",
            "10.1371/JOURNAL.PONE.0123456",
            "https://doi.org/10.1371/journal.pone.0123456",
            "http://dx.doi.org/10.1371/journal.pone.0123456",
            "doi:10.1371/journal.pone.0123456",
            "DOI: 10.1371/journal.pone.0123456",
        ] {
            assert_eq!(text.parse::<Doi>().unwrap().as_str(), expected, "{text}");
        }
    }

    #[test]
    fn doi_accepts_registrant_subdivisions() {
        assert!("10.1000.10/abc".parse::<Doi>().is_ok());
    }

    #[test]
    fn doi_rejects_malformed_text() {
        for text in [
            "",
            "10.1371",
            "11.1371/x",
            "10.13/x",
            "10.1371/",
            "10.1371/a b",
            "10.ab12/x",
        ] {
            assert!(text.parse::<Doi>().is_err(), "{text}");
        }
    }

    #[test]
    fn doi_has_a_resolver_url() {
        let doi: Doi = "10.1038/nature12373".parse().unwrap();
        assert_eq!(doi.url(), "https://doi.org/10.1038/nature12373");
    }

    #[test]
    fn pmcid_normalizes_the_prefix() {
        assert_eq!("pmc123".parse::<Pmcid>().unwrap().as_str(), "PMC123");
        assert_eq!(
            "PMC7654321".parse::<Pmcid>().unwrap().as_str(),
            "PMC7654321"
        );
    }

    #[test]
    fn pmcid_rejects_malformed_text() {
        for text in ["", "PMC", "123", "PMC0123", "PMC12a", "PMCX"] {
            assert!(text.parse::<Pmcid>().is_err(), "{text}");
        }
    }

    #[test]
    fn pmid_requires_a_positive_integer() {
        assert_eq!("31452104".parse::<Pmid>().unwrap().as_str(), "31452104");
        for text in ["", "0", "012", "12.5", "PMC12", "12345678901"] {
            assert!(text.parse::<Pmid>().is_err(), "{text}");
        }
    }

    #[test]
    fn orcid_accepts_valid_identifiers() {
        for text in [
            "0000-0002-1825-0097",
            "https://orcid.org/0000-0002-1825-0097",
            "0000-0002-1694-233X",
        ] {
            assert!(text.parse::<Orcid>().is_ok(), "{text}");
        }
        let orcid: Orcid = "https://orcid.org/0000-0002-1825-0097".parse().unwrap();
        assert_eq!(orcid.as_str(), "0000-0002-1825-0097");
        assert_eq!(orcid.url(), "https://orcid.org/0000-0002-1825-0097");
    }

    #[test]
    fn orcid_rejects_an_incorrect_check_digit() {
        assert_eq!(
            "0000-0002-1825-0098".parse::<Orcid>(),
            Err(IdError::OrcidChecksum("0000-0002-1825-0098".to_owned()))
        );
    }

    #[test]
    fn orcid_rejects_malformed_text() {
        for text in [
            "",
            "0000-0002-1825",
            "0000000218250097",
            "0000-0002-1825-009Y",
        ] {
            assert!(matches!(
                text.parse::<Orcid>(),
                Err(IdError::Invalid { .. })
            ));
        }
    }

    #[test]
    fn identifiers_serialize_as_strings() {
        let doi: Doi = serde_json::from_str("\"10.1038/NATURE12373\"").unwrap();
        assert_eq!(
            serde_json::to_string(&doi).unwrap(),
            "\"10.1038/nature12373\""
        );
        assert!(serde_json::from_str::<Pmcid>("\"12\"").is_err());
    }
}
