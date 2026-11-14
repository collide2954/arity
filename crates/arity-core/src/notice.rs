use std::fmt;
use std::str::FromStr;

use jiff::Timestamp;
use jiff::civil::Date;
use serde::{Deserialize, Serialize};

use crate::{Doi, Pmid};

/// The kind of a notice that a publisher or a registry issues about a work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoticeKind {
    Retraction,
    PartialRetraction,
    Withdrawal,
    Removal,
    ExpressionOfConcern,
    Correction,
    Erratum,
    Corrigendum,
    Addendum,
    Clarification,
    NewVersion,
    NewEdition,
    Reinstatement,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0:?} is not a known notice kind")]
pub struct UnknownNoticeKind(pub String);

impl NoticeKind {
    pub const ALL: [Self; 14] = [
        Self::Retraction,
        Self::PartialRetraction,
        Self::Withdrawal,
        Self::Removal,
        Self::ExpressionOfConcern,
        Self::Correction,
        Self::Erratum,
        Self::Corrigendum,
        Self::Addendum,
        Self::Clarification,
        Self::NewVersion,
        Self::NewEdition,
        Self::Reinstatement,
        Self::Other,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Retraction => "retraction",
            Self::PartialRetraction => "partial_retraction",
            Self::Withdrawal => "withdrawal",
            Self::Removal => "removal",
            Self::ExpressionOfConcern => "expression_of_concern",
            Self::Correction => "correction",
            Self::Erratum => "erratum",
            Self::Corrigendum => "corrigendum",
            Self::Addendum => "addendum",
            Self::Clarification => "clarification",
            Self::NewVersion => "new_version",
            Self::NewEdition => "new_edition",
            Self::Reinstatement => "reinstatement",
            Self::Other => "other",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Retraction => "Retraction",
            Self::PartialRetraction => "Partial retraction",
            Self::Withdrawal => "Withdrawal",
            Self::Removal => "Removal",
            Self::ExpressionOfConcern => "Expression of concern",
            Self::Correction => "Correction",
            Self::Erratum => "Erratum",
            Self::Corrigendum => "Corrigendum",
            Self::Addendum => "Addendum",
            Self::Clarification => "Clarification",
            Self::NewVersion => "New version",
            Self::NewEdition => "New edition",
            Self::Reinstatement => "Reinstatement",
            Self::Other => "Other notice",
        }
    }
}

impl fmt::Display for NoticeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for NoticeKind {
    type Err = UnknownNoticeKind;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.as_str() == text)
            .ok_or_else(|| UnknownNoticeKind(text.to_owned()))
    }
}

/// The service that told arity about a notice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoticeSource {
    Crossref,
    EuropePmc,
    RetractionWatch,
    Publisher,
}

impl NoticeSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Crossref => "crossref",
            Self::EuropePmc => "europe_pmc",
            Self::RetractionWatch => "retraction_watch",
            Self::Publisher => "publisher",
        }
    }
}

/// A notice about a work, exactly as one source reports it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Notice {
    pub kind: NoticeKind,
    /// The type name that the source uses, without change.
    pub reported_type: String,
    pub source: NoticeSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notice_doi: Option<Doi>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notice_pmid: Option<Pmid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notice_citation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<Date>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_record: Option<String>,
    pub retrieved_at: Timestamp,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip() {
        for kind in NoticeKind::ALL {
            assert_eq!(kind.as_str().parse::<NoticeKind>(), Ok(kind));
            assert_eq!(
                serde_json::to_string(&kind).unwrap(),
                format!("\"{}\"", kind.as_str())
            );
        }
        assert!("Retraction".parse::<NoticeKind>().is_err());
    }

    #[test]
    fn labels_start_with_a_capital_letter() {
        for kind in NoticeKind::ALL {
            assert!(kind.label().starts_with(char::is_uppercase), "{kind}");
        }
    }

    #[test]
    fn a_notice_serializes_without_empty_fields() {
        let notice = Notice {
            kind: NoticeKind::Retraction,
            reported_type: "retraction".to_owned(),
            source: NoticeSource::Crossref,
            notice_doi: Some("10.1000/retraction.1".parse().unwrap()),
            notice_pmid: None,
            notice_citation: None,
            date: Some(jiff::civil::date(2025, 3, 4)),
            source_record: None,
            retrieved_at: "2026-11-13T10:00:00Z".parse().unwrap(),
        };
        let json = serde_json::to_value(&notice).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "kind": "retraction",
                "reported_type": "retraction",
                "source": "crossref",
                "notice_doi": "10.1000/retraction.1",
                "date": "2025-03-04",
                "retrieved_at": "2026-11-13T10:00:00Z",
            })
        );
        let back: Notice = serde_json::from_value(json).unwrap();
        assert_eq!(back, notice);
    }
}
