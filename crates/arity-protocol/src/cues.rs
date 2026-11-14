//! Words that state a direction or a limit. A missing cue gives a warning,
//! not an error: the checks cannot read meaning, so a person must look.

use arity_core::vocabulary::{Bound, Direction};
use arity_text::normalize;

const INCREASE: &[&str] = &[
    "increas",
    "higher",
    "elevat",
    "enhanc",
    "upregulat",
    "up-regulat",
    "rais",
    "rise",
    "rose",
    "greater",
    "more ",
    "induc",
    "stimulat",
    "promot",
    "boost",
    "augment",
    "accumulat",
    "exceed",
    "gain",
    "potentiat",
    "amplif",
    "activat",
    "↑",
];

const DECREASE: &[&str] = &[
    "decreas",
    "reduc",
    "lower",
    "inhibit",
    "suppress",
    "downregulat",
    "down-regulat",
    "declin",
    "attenuat",
    "diminish",
    "less ",
    "loss",
    "drop",
    "fell",
    "fall",
    "impair",
    "abolish",
    "block",
    "prevent",
    "lessen",
    "deplet",
    "shorten",
    "sensitiz",
    "sensitis",
    "↓",
];

const NO_CHANGE: &[&str] = &[
    "no ",
    "not ",
    "n't",
    "unchanged",
    "unaffected",
    "unaltered",
    "similar",
    "comparable",
    "without",
    "failed to",
    "neither",
    "insignificant",
    "non-significant",
    "nonsignificant",
    "n.s",
    "same",
    "stable",
];

const LESS_THAN: &[&str] = &["<", "less than", "below", "lower than", "under"];
const AT_MOST: &[&str] = &["≤", "<=", "at most", "or less", "up to", "no more than"];
const GREATER_THAN: &[&str] = &[">", "greater than", "more than", "above", "exceed", "over"];
const AT_LEAST: &[&str] = &["≥", ">=", "at least", "or more", "no less than"];
const APPROXIMATELY: &[&str] = &[
    "~", "∼", "≈", "≃", "approx", "about", "around", "circa", "ca.", "roughly", "nearly",
];

fn joined(quotes: &[String]) -> String {
    let mut text = String::new();
    for quote in quotes {
        text.push(' ');
        text.push_str(&normalize(quote).as_str().to_lowercase());
    }
    text.push(' ');
    text
}

pub(crate) fn states_direction(direction: Direction, quotes: &[String]) -> bool {
    let cues = match direction {
        Direction::Increase => INCREASE,
        Direction::Decrease => DECREASE,
        Direction::NoChange => NO_CHANGE,
        Direction::Mixed => return true,
    };
    let text = joined(quotes);
    cues.iter().any(|cue| text.contains(cue))
}

pub(crate) fn states_bound(bound: Bound, quotes: &[String]) -> bool {
    let cues = match bound {
        Bound::LessThan => LESS_THAN,
        Bound::AtMost => AT_MOST,
        Bound::GreaterThan => GREATER_THAN,
        Bound::AtLeast => AT_LEAST,
        Bound::Approximately => APPROXIMATELY,
    };
    let text = joined(quotes);
    cues.iter().any(|cue| text.contains(cue))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quotes(texts: &[&str]) -> Vec<String> {
        texts.iter().map(|text| (*text).to_owned()).collect()
    }

    #[test]
    fn finds_direction_words() {
        assert!(states_direction(
            Direction::Decrease,
            &quotes(&["a reduction in the IC50 values"])
        ));
        assert!(states_direction(
            Direction::Increase,
            &quotes(&["Caspase-3 was Up-regulated"])
        ));
        assert!(states_direction(
            Direction::NoChange,
            &quotes(&["No effect of A on B was found"])
        ));
        assert!(states_direction(
            Direction::NoChange,
            &quotes(&["B did not change"])
        ));
        assert!(!states_direction(
            Direction::NoChange,
            &quotes(&["A strongly reduced B"])
        ));
        assert!(states_direction(Direction::Mixed, &[]));
    }

    #[test]
    fn finds_limit_words() {
        assert!(states_bound(Bound::LessThan, &quotes(&["IC50 < 0.1 μM"])));
        assert!(states_bound(Bound::AtLeast, &quotes(&["≥ 50%"])));
        assert!(states_bound(Bound::Approximately, &quotes(&["about 3 h"])));
        assert!(!states_bound(Bound::GreaterThan, &quotes(&["IC50 = 3 μM"])));
    }
}
