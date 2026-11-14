//! The submission format of arity and the checks that each submission must
//! pass.
//!
//! A reader reads the canonical text of a source and writes a submission. The
//! checks make sure that each report agrees with the text: each quote occurs
//! in its block, each number occurs in a quote, and each unit agrees with its
//! property. A submission with an error does not go into the database.

mod check;
mod coverage;
mod cues;
mod issue;
mod properties;
mod submission;

pub use check::{Checked, Rejected, check};
pub use coverage::{Count, Coverage, Unquoted, coverage};
pub use issue::{Issue, IssueCode, Severity};
pub use properties::{Properties, PropertyDefinition};
pub use submission::{
    AgentInput, ConditionInput, EffectReportInput, EntityInput, EvidenceInput, FigureReadingInput,
    MagnitudeInput, MethodInput, Person, PropertyInput, Reader, ReportInput, SUBMISSION_FORMAT,
    SignificanceInput, Submission, TargetInput, UncertaintyInput, ValueInput, ValueReportInput,
    schema,
};
