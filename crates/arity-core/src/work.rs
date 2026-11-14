use jiff::civil::Date;
use serde::{Deserialize, Serialize};

use crate::{Doi, License, Orcid, Pmcid, Pmid};

/// The bibliographic record of a work, as its source states it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkMetadata {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doi: Option<Doi>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pmid: Option<Pmid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pmcid: Option<Pmcid>,
    /// The type that the source gives, for example `research-article`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub article_type: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_preprint: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub journal: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published: Option<Date>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub year: Option<i16>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<Author>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<License>,
    /// The license text of the source, for attribution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license_statement: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub copyright: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Author {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orcid: Option<Orcid>,
}

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde passes the field by reference"
)]
fn is_false(value: &bool) -> bool {
    !value
}

impl WorkMetadata {
    /// A short citation for people, for example `Lafi et al. 2023, PLOS ONE`.
    pub fn short_citation(&self) -> String {
        let authors = self.authors.first().map(|author| {
            let surname = author.name.rsplit(' ').next().unwrap_or(&author.name);
            if self.authors.len() > 1 {
                format!("{surname} et al.")
            } else {
                surname.to_owned()
            }
        });
        let rest: Vec<String> = [self.year.map(|year| year.to_string()), self.journal.clone()]
            .into_iter()
            .flatten()
            .collect();
        match (authors, rest.is_empty()) {
            (Some(authors), true) => authors,
            (Some(authors), false) => format!("{authors} {}", rest.join(", ")),
            (None, false) => rest.join(", "),
            (None, true) => self.title.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn author(name: &str) -> Author {
        Author {
            name: name.to_owned(),
            orcid: None,
        }
    }

    #[test]
    fn short_citations_use_the_first_surname() {
        let mut work = WorkMetadata {
            title: "A title".to_owned(),
            year: Some(2023),
            journal: Some("PLOS ONE".to_owned()),
            authors: vec![author("Zainab Lafi"), author("Walhan Alshaer")],
            ..WorkMetadata::default()
        };
        assert_eq!(work.short_citation(), "Lafi et al. 2023, PLOS ONE");
        work.authors.truncate(1);
        assert_eq!(work.short_citation(), "Lafi 2023, PLOS ONE");
        work.authors.clear();
        work.journal = None;
        assert_eq!(work.short_citation(), "2023");
        work.year = None;
        assert_eq!(work.short_citation(), "A title");
    }

    #[test]
    fn serializes_without_empty_fields() {
        let work = WorkMetadata {
            title: "A title".to_owned(),
            pmcid: Some("PMC1".parse().unwrap()),
            ..WorkMetadata::default()
        };
        assert_eq!(
            serde_json::to_value(&work).unwrap(),
            serde_json::json!({ "title": "A title", "pmcid": "PMC1" })
        );
    }
}
