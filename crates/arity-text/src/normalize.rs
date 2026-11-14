use std::ops::Range;

use unicode_normalization::UnicodeNormalization;

/// Text in the normal form that quote search uses, with a map back to the
/// original text.
///
/// The normal form applies NFKC to each character, folds dash and quote
/// variants, removes soft hyphens and zero-width characters, and collapses
/// each run of white space to one space.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Normalized {
    text: String,
    origins: Vec<usize>,
}

impl Normalized {
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// Finds each place where the quote occurs. Each range counts Unicode
    /// scalar values in the original text.
    pub fn find(&self, quote: &str) -> Vec<Range<usize>> {
        let needle = normalize(quote).text;
        if needle.is_empty() {
            return Vec::new();
        }
        let needle_chars = needle.chars().count();
        self.text
            .match_indices(&needle)
            .filter_map(|(byte, _)| {
                let first = self.text[..byte].chars().count();
                let start = *self.origins.get(first)?;
                let end = *self.origins.get(first + needle_chars - 1)? + 1;
                Some(start..end)
            })
            .collect()
    }
}

pub fn normalize(text: &str) -> Normalized {
    let mut out = String::with_capacity(text.len());
    let mut origins = Vec::with_capacity(text.len());
    let mut pending_space: Option<usize> = None;
    for (index, c) in text.chars().enumerate() {
        if is_removed(c) {
            continue;
        }
        for folded in std::iter::once(c).nfkc().map(fold) {
            if folded == ' ' {
                if !out.is_empty() && pending_space.is_none() {
                    pending_space = Some(index);
                }
                continue;
            }
            if let Some(space) = pending_space.take() {
                out.push(' ');
                origins.push(space);
            }
            out.push(folded);
            origins.push(index);
        }
    }
    Normalized { text: out, origins }
}

fn is_removed(c: char) -> bool {
    matches!(
        c,
        '\u{00AD}' | '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{2060}' | '\u{FEFF}'
    )
}

fn fold(c: char) -> char {
    match c {
        c if c.is_whitespace() => ' ',
        '\u{2010}'..='\u{2015}' | '\u{2212}' | '\u{FE58}' | '\u{FE63}' | '\u{FF0D}' => '-',
        '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}' | '\u{2032}' | '\u{2035}' => '\'',
        '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{201F}' => '"',
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[derive(serde::Deserialize)]
    struct Vectors {
        normalize: Vec<NormalizeCase>,
        find: Vec<FindCase>,
    }

    #[derive(serde::Deserialize)]
    struct NormalizeCase {
        input: String,
        output: String,
    }

    #[derive(serde::Deserialize)]
    struct FindCase {
        text: String,
        quote: String,
        ranges: Vec<[usize; 2]>,
    }

    fn vectors() -> Vectors {
        serde_json::from_str(include_str!("../../../testdata/text/normalization.json")).unwrap()
    }

    #[test]
    fn follows_the_shared_normalization_vectors() {
        for case in vectors().normalize {
            assert_eq!(
                normalize(&case.input).as_str(),
                case.output,
                "{:?}",
                case.input
            );
        }
    }

    #[test]
    fn follows_the_shared_search_vectors() {
        for case in vectors().find {
            let expected: Vec<Range<usize>> = case.ranges.iter().map(|[a, b]| *a..*b).collect();
            assert_eq!(
                normalize(&case.text).find(&case.quote),
                expected,
                "{:?} in {:?}",
                case.quote,
                case.text
            );
        }
    }

    proptest! {
        #[test]
        fn normalization_is_idempotent(text in "\\PC{0,40}") {
            let once = normalize(&text);
            let twice = normalize(once.as_str());
            prop_assert_eq!(twice.as_str(), once.as_str());
        }

        #[test]
        fn each_origin_points_into_the_original(text in "\\PC{0,40}") {
            let normalized = normalize(&text);
            let count = text.chars().count();
            prop_assert!(normalized.origins.iter().all(|origin| *origin < count));
            prop_assert_eq!(normalized.origins.len(), normalized.text.chars().count());
        }
    }
}
