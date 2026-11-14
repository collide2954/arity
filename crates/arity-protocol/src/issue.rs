use std::fmt;

use serde::{Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// The submission cannot go into the database.
    Error,
    /// The submission can go into the database, and a person must look at
    /// the part that the warning names.
    Warning,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
        }
    }
}

macro_rules! issue_codes {
    ($($variant:ident => $text:literal, $severity:ident, $description:literal;)+) => {
        /// The kind of a problem that the checks find. The name of each kind
        /// is stable, so that tools can use it.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum IssueCode {
            $(#[doc = $description] $variant,)+
        }

        impl IssueCode {
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            pub fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $text,)+
                }
            }

            /// The severity of each issue of this kind.
            pub fn severity(self) -> Severity {
                match self {
                    $(Self::$variant => Severity::$severity,)+
                }
            }

            pub fn description(self) -> &'static str {
                match self {
                    $(Self::$variant => $description,)+
                }
            }
        }
    };
}

issue_codes! {
    AmbiguousQuote => "ambiguous_quote", Warning,
        "The quote occurs more than one time in the block, and the submission does not give the occurrence. arity uses the first.";
    BoundCueMissing => "bound_cue_missing", Warning,
        "The value has a bound, but no quote has a sign or a word for that bound.";
    CategoryNotInEvidence => "category_not_in_evidence", Warning,
        "No quote of the report contains the category of the value.";
    ConditionShape => "condition_shape", Error,
        "A condition does not have a role with an entity or a text, or a property with a value.";
    DimensionMismatch => "dimension_mismatch", Error,
        "The unit does not measure the dimension of the property.";
    DirectionCueMissing => "direction_cue_missing", Warning,
        "No quote has a word that states the direction of the effect.";
    DuplicateId => "duplicate_id", Error,
        "Two entities, or two reports, have the same identifier.";
    EmptyLabel => "empty_label", Error,
        "The label of an entity is empty.";
    EmptyMethod => "empty_method", Error,
        "A value from a figure does not tell how the reader read it.";
    EmptyName => "empty_name", Error,
        "A name that the submission must give is empty, for example the name of the reader.";
    EmptyQuote => "empty_quote", Error,
        "A quote is empty.";
    FigureNotFound => "figure_not_found", Error,
        "The text has no figure caption with the key that the value names.";
    FigureNotInEvidence => "figure_not_in_evidence", Error,
        "A value from a figure does not have the caption of the figure in its evidence.";
    InvalidId => "invalid_id", Error,
        "An identifier is empty or has a space.";
    InvalidInterval => "invalid_interval", Error,
        "The low end of an interval is greater than the high end.";
    InvalidJson => "invalid_json", Error,
        "The file is not JSON, or it does not have the fields of a submission.";
    InvalidKey => "invalid_key", Error,
        "A property key does not start with a letter, or has characters other than lower-case letters, digits, and underscores.";
    InvalidLevel => "invalid_level", Error,
        "The level of an interval is not a percent from 0 to 100.";
    InvalidP => "invalid_p", Error,
        "A p-value is not from 0 to 1.";
    InvalidResolution => "invalid_resolution", Error,
        "The resolution of a reading from a figure is not more than zero.";
    InvalidSampleSize => "invalid_sample_size", Error,
        "The number of samples is less than 1.";
    InvalidUnit => "invalid_unit", Error,
        "The unit is not a valid UCUM unit.";
    LabelDiffers => "label_differs", Warning,
        "The label of a property is not the stored label. arity keeps the stored label.";
    LongQuote => "long_quote", Warning,
        "A quote is long. A short quote is easier to check.";
    MethodNotInEvidence => "method_not_in_evidence", Warning,
        "No quote names the method of the report.";
    MissingDefinition => "missing_definition", Error,
        "The property is new, and the submission does not define it fully.";
    NegativeUncertainty => "negative_uncertainty", Error,
        "An uncertainty is less than zero.";
    NoAgents => "no_agents", Error,
        "An effect has no agent.";
    NoEvidence => "no_evidence", Error,
        "A report has no quote from the source.";
    NoMention => "no_mention", Error,
        "An entity has no quote of a place where the source names it.";
    NoReports => "no_reports", Error,
        "The submission has no reports.";
    NoSubject => "no_subject", Error,
        "A value has no subject.";
    NoVersion => "no_version", Warning,
        "The software that read the source has no version.";
    NoXref => "no_xref", Warning,
        "The entity has no registry identifier, so arity cannot link it to other works.";
    NumberNotInEvidence => "number_not_in_evidence", Error,
        "No quote of the report contains the number.";
    OccurrenceOutOfRange => "occurrence_out_of_range", Error,
        "The quote does not occur as many times in the block as the occurrence that the submission gives.";
    PropertyConflict => "property_conflict", Error,
        "The definition of a property does not agree with the stored definition.";
    QuoteNotFound => "quote_not_found", Error,
        "The quote does not occur in the block.";
    QuoteTooLong => "quote_too_long", Error,
        "A quote is longer than the maximum length of a quote.";
    SignificanceShape => "significance_shape", Error,
        "A significance does not have a statement, a p-value, or both.";
    TextMismatch => "text_mismatch", Error,
        "The submission names a text that is not the current text of the work.";
    TextNotInEvidence => "text_not_in_evidence", Error,
        "No quote contains a text that the report gives, for example the comparator.";
    TextUnreadable => "text_unreadable", Error,
        "arity cannot make the digest of the text.";
    UncertaintyShape => "uncertainty_shape", Error,
        "An uncertainty does not have a number, or a low and a high end.";
    UnexpectedField => "unexpected_field", Error,
        "The submission gives a field that does not apply, for example a unit for a property that is not a quantity.";
    UnitRequired => "unit_required", Error,
        "A quantity has no UCUM unit.";
    UnitTextMismatch => "unit_text_mismatch", Error,
        "The unit that the source writes is not the same unit as the UCUM unit.";
    UnitTextNotInEvidence => "unit_text_not_in_evidence", Error,
        "No quote contains the unit as the source writes it.";
    UnitTextRequired => "unit_text_required", Error,
        "A quantity does not give the unit as the source writes it.";
    UnitTextUnreadable => "unit_text_unreadable", Warning,
        "arity cannot read the unit that the source writes, so a person must check the unit.";
    UnknownBlock => "unknown_block", Error,
        "The text has no block with this key.";
    UnknownCategory => "unknown_category", Error,
        "The category is not one of the categories of the property.";
    UnknownEntity => "unknown_entity", Error,
        "No entity of the submission has this identifier.";
    UnknownFormat => "unknown_format", Error,
        "The format of the submission is not arity-submission/1.";
    UnknownProperty => "unknown_property", Error,
        "The property is not in the database, and the submission does not define it.";
    ValueShape => "value_shape", Error,
        "A value does not have exactly one of: a number, a low and a high end, a boolean, or a category.";
    ValueTypeMismatch => "value_type_mismatch", Error,
        "The value does not have the value type of the property.";
    XrefKindMismatch => "xref_kind_mismatch", Error,
        "The registry of the identifier does not identify entities of this kind.";
}

impl fmt::Display for IssueCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for IssueCode {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

/// A problem that the checks find in a submission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Issue {
    pub severity: Severity,
    /// A JSON Pointer to the part of the submission, for example
    /// `/reports/0/value/number`.
    pub path: String,
    pub code: IssueCode,
    pub message: String,
}

impl Issue {
    /// An issue of the kind that the code names, with the severity of that
    /// kind.
    pub fn new(code: IssueCode, path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: code.severity(),
            path: path.into(),
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = if self.path.is_empty() {
            "/"
        } else {
            &self.path
        };
        write!(
            f,
            "{} at {path}: {} [{}]",
            self.severity.as_str(),
            self.message,
            self.code
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn displays_the_path_and_the_code() {
        let issue = Issue::new(
            IssueCode::NumberNotInEvidence,
            "/reports/0/value/number",
            "no quote contains 3.2",
        );
        assert_eq!(
            issue.to_string(),
            "error at /reports/0/value/number: no quote contains 3.2 [number_not_in_evidence]"
        );
    }

    #[test]
    fn gives_each_code_a_unique_name_and_a_description() {
        let names: HashSet<&str> = IssueCode::ALL.iter().map(|code| code.as_str()).collect();
        assert_eq!(names.len(), IssueCode::ALL.len());
        for code in IssueCode::ALL {
            let name = code.as_str();
            assert!(
                name.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
                "{name}"
            );
            assert!(code.description().ends_with('.'), "{name}");
        }
    }

    #[test]
    fn serializes_the_code_as_its_name() {
        let issue = Issue::new(IssueCode::NoXref, "/entities/0/xrefs", "no identifier");
        let json = serde_json::to_value(&issue).unwrap();
        assert_eq!(json["code"], "no_xref");
        assert_eq!(json["severity"], "warning");
    }
}
