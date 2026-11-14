use std::fmt;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// The submission cannot go into the database.
    Error,
    /// The submission can go into the database, and a person must look at
    /// the part that the warning names.
    Warning,
}

/// A problem that the checks find in a submission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Issue {
    pub severity: Severity,
    /// A JSON Pointer to the part of the submission, for example
    /// `/reports/0/value/number`.
    pub path: String,
    /// A stable name for the kind of problem, for example `quote_not_found`.
    pub code: &'static str,
    pub message: String,
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let severity = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        let path = if self.path.is_empty() {
            "/"
        } else {
            &self.path
        };
        write!(f, "{severity} at {path}: {} [{}]", self.message, self.code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_the_path_and_the_code() {
        let issue = Issue {
            severity: Severity::Error,
            path: "/reports/0/value/number".to_owned(),
            code: "number_not_in_evidence",
            message: "no quote contains 3.2".to_owned(),
        };
        assert_eq!(
            issue.to_string(),
            "error at /reports/0/value/number: no quote contains 3.2 [number_not_in_evidence]"
        );
    }
}
