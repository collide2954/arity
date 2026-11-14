//! The closed lists of terms that reports use.
//!
//! Each term has one spelling in JSON, in the database, and on the website.

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{text:?} is not a term of {vocabulary}; use one of: {allowed}")]
pub struct UnknownTerm {
    pub vocabulary: &'static str,
    pub text: String,
    pub allowed: String,
}

macro_rules! vocabulary {
    (
        $(#[$meta:meta])*
        pub enum $name:ident {
            $($(#[$variant_meta:meta])* $variant:ident => $text:literal),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord,
            serde::Serialize, serde::Deserialize, schemars::JsonSchema,
        )]
        pub enum $name {
            $($(#[$variant_meta])* #[serde(rename = $text)] $variant),+
        }

        impl $name {
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            pub fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $text),+
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl std::str::FromStr for $name {
            type Err = UnknownTerm;

            fn from_str(text: &str) -> Result<Self, Self::Err> {
                Self::ALL
                    .iter()
                    .copied()
                    .find(|term| term.as_str() == text)
                    .ok_or_else(|| UnknownTerm {
                        vocabulary: stringify!($name),
                        text: text.to_owned(),
                        allowed: Self::ALL
                            .iter()
                            .map(|term| term.as_str())
                            .collect::<Vec<_>>()
                            .join(", "),
                    })
            }
        }
    };
}

vocabulary! {
    /// The direction of a change, as the source states it.
    pub enum Direction {
        Increase => "increase",
        Decrease => "decrease",
        /// The source states that there is no change or no effect.
        NoChange => "no_change",
        /// The source states that the change is not the same in all cases.
        Mixed => "mixed",
    }
}

impl Direction {
    /// The value that a table or a matrix shows: 1, -1, or 0.
    pub fn score(self) -> Option<i8> {
        match self {
            Self::Increase => Some(1),
            Self::Decrease => Some(-1),
            Self::NoChange => Some(0),
            Self::Mixed => None,
        }
    }
}

vocabulary! {
    /// What a source says about two things: that one changes the other, or
    /// that the two vary together.
    pub enum Relation {
        Effect => "effect",
        Association => "association",
    }
}

vocabulary! {
    /// The statistic that a value is.
    pub enum Statistic {
        Mean => "mean",
        Median => "median",
        GeometricMean => "geometric_mean",
        Mode => "mode",
        Single => "single",
        Minimum => "minimum",
        Maximum => "maximum",
        Other => "other",
    }
}

vocabulary! {
    /// The kind of uncertainty that a source gives with a value.
    pub enum UncertaintyKind {
        StandardDeviation => "sd",
        StandardError => "sem",
        ConfidenceInterval => "ci",
        Range => "range",
        InterquartileRange => "iqr",
        Other => "other",
    }
}

vocabulary! {
    /// A limit that a source states instead of an exact value, for example
    /// "less than 0.1".
    pub enum Bound {
        LessThan => "lt",
        AtMost => "le",
        GreaterThan => "gt",
        AtLeast => "ge",
        Approximately => "approx",
    }
}

vocabulary! {
    /// The part that an entity has in the conditions of a report.
    pub enum Role {
        Organism => "organism",
        CellLine => "cell_line",
        CellType => "cell_type",
        AnatomicalSite => "anatomical_site",
        Sample => "sample",
        Target => "target",
        Solvent => "solvent",
        Medium => "medium",
        CoTreatment => "co_treatment",
        Comparator => "comparator",
        Population => "population",
        Device => "device",
        Environment => "environment",
        Other => "other",
    }
}

vocabulary! {
    /// The measure of the size of an effect.
    pub enum EffectMeasure {
        FoldChange => "fold_change",
        Log2FoldChange => "log2_fold_change",
        PercentChange => "percent_change",
        Difference => "difference",
        Ratio => "ratio",
        OddsRatio => "odds_ratio",
        RiskRatio => "risk_ratio",
        HazardRatio => "hazard_ratio",
        StandardizedMeanDifference => "standardized_mean_difference",
        Correlation => "correlation",
        RankCorrelation => "rank_correlation",
        CoefficientOfDetermination => "coefficient_of_determination",
        RegressionCoefficient => "regression_coefficient",
        Other => "other",
    }
}

vocabulary! {
    /// A statement of statistical significance, as the source makes it.
    pub enum SignificanceStatement {
        Significant => "significant",
        NotSignificant => "not_significant",
    }
}

vocabulary! {
    /// The comparison in a stated p-value, for example `p < 0.05`.
    pub enum Comparison {
        Equal => "eq",
        LessThan => "lt",
        AtMost => "le",
        GreaterThan => "gt",
        AtLeast => "ge",
    }
}

vocabulary! {
    /// The result of a check of a report against its source by a person.
    pub enum CheckState {
        Unchecked => "unchecked",
        Confirmed => "confirmed",
        Rejected => "rejected",
    }
}

vocabulary! {
    /// The kind of value that a property takes.
    pub enum ValueType {
        Quantity => "quantity",
        Boolean => "boolean",
        Category => "category",
    }
}

vocabulary! {
    /// The kind of the agent that reads a source.
    pub enum AgentKind {
        Person => "person",
        Software => "software",
    }
}

vocabulary! {
    /// Why a reader does not report a number of the text.
    pub enum SkipReason {
        /// The source states a value that another work reports. arity records
        /// that value from its own source, so that the value does not count
        /// two times.
        Cited => "cited",
        /// The source states again a value that it states in another place,
        /// for example in the abstract, and a report quotes that place.
        Restated => "restated",
        /// Another reason. The note of the skipped quote tells it.
        Other => "other",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip<T>(terms: &[T])
    where
        T: Copy
            + std::fmt::Debug
            + PartialEq
            + std::str::FromStr
            + serde::Serialize
            + std::fmt::Display,
        <T as std::str::FromStr>::Err: std::fmt::Debug,
    {
        for term in terms {
            let text = term.to_string();
            assert_eq!(text.parse::<T>().unwrap(), *term);
            assert_eq!(serde_json::to_string(term).unwrap(), format!("\"{text}\""));
        }
    }

    #[test]
    fn every_term_round_trips() {
        round_trip(Direction::ALL);
        round_trip(Relation::ALL);
        round_trip(Statistic::ALL);
        round_trip(UncertaintyKind::ALL);
        round_trip(Bound::ALL);
        round_trip(Role::ALL);
        round_trip(EffectMeasure::ALL);
        round_trip(SignificanceStatement::ALL);
        round_trip(Comparison::ALL);
        round_trip(CheckState::ALL);
        round_trip(ValueType::ALL);
        round_trip(AgentKind::ALL);
    }

    #[test]
    fn unknown_terms_name_the_allowed_terms() {
        let error = "none".parse::<Direction>().unwrap_err();
        assert_eq!(
            error.to_string(),
            "\"none\" is not a term of Direction; use one of: increase, decrease, no_change, mixed"
        );
    }

    #[test]
    fn no_change_scores_zero() {
        assert_eq!(Direction::NoChange.score(), Some(0));
        assert_eq!(Direction::Increase.score(), Some(1));
        assert_eq!(Direction::Decrease.score(), Some(-1));
        assert_eq!(Direction::Mixed.score(), None);
    }

    #[test]
    fn the_json_schema_lists_the_terms() {
        let schema = serde_json::to_string(&schemars::schema_for!(Direction)).unwrap();
        for term in Direction::ALL {
            assert!(
                schema.contains(&format!("\"{term}\"")),
                "{term} in {schema}"
            );
        }
    }
}
