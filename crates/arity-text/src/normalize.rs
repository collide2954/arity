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
                self.original(first..first + needle_chars)
            })
            .collect()
    }

    /// The range in the original text that a range of the normal form comes
    /// from. Both ranges count Unicode scalar values. An empty range, or a
    /// range past the end, has no original.
    pub fn original(&self, range: Range<usize>) -> Option<Range<usize>> {
        let last = range
            .end
            .checked_sub(1)
            .filter(|last| *last >= range.start)?;
        let start = *self.origins.get(range.start)?;
        let end = *self.origins.get(last)? + 1;
        Some(start..end)
    }
}

pub fn normalize(text: &str) -> Normalized {
    normalize_with(text, false)
}

/// Text in the normal form that number search uses. It is the normal form of
/// quote search, but a run of superscript characters starts with `^`, and a
/// run of subscript characters starts with `_`. Thus `10⁻³` becomes `10^-3`,
/// and not `10-3`. A script with a mark, for example the citations of
/// `solvents^{59,60}`, has no numbers, so this form leaves it out.
pub fn normalize_numbers(text: &str) -> Normalized {
    normalize_with(text, true)
}

/// The places of the characters of each script with a mark: `^{...}` or
/// `_{...}`, with its braces.
fn marked_scripts(chars: &[char]) -> Vec<bool> {
    let mut inside = vec![false; chars.len()];
    let mut index = 0;
    while index + 1 < chars.len() {
        if matches!(chars[index], '^' | '_') && chars[index + 1] == '{' {
            let mut depth = 0_usize;
            let close = (index + 1..chars.len()).find(|&at| {
                match chars[at] {
                    '{' => depth += 1,
                    '}' => depth -= 1,
                    _ => {}
                }
                depth == 0
            });
            if let Some(close) = close {
                inside[index..=close].fill(true);
                index = close + 1;
                continue;
            }
        }
        index += 1;
    }
    inside
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Script {
    Base,
    Superscript,
    Subscript,
}

fn script(c: char) -> Script {
    match c {
        '\u{00B2}' | '\u{00B3}' | '\u{00B9}' | '\u{2070}' | '\u{2074}'..='\u{207E}' => {
            Script::Superscript
        }
        '\u{2080}'..='\u{208E}' => Script::Subscript,
        _ => Script::Base,
    }
}

fn normalize_with(text: &str, mark_scripts: bool) -> Normalized {
    let mut out = String::with_capacity(text.len());
    let mut origins = Vec::with_capacity(text.len());
    let mut pending_space: Option<usize> = None;
    let mut previous = Script::Base;
    let skipped = if mark_scripts {
        marked_scripts(&text.chars().collect::<Vec<_>>())
    } else {
        Vec::new()
    };
    for (index, c) in text.chars().enumerate() {
        if is_removed(c) || skipped.get(index).copied().unwrap_or(false) {
            continue;
        }
        let current = if mark_scripts {
            script(c)
        } else {
            Script::Base
        };
        if current != previous && current != Script::Base {
            if let Some(space) = pending_space.take() {
                out.push(' ');
                origins.push(space);
            }
            out.push(if current == Script::Superscript {
                '^'
            } else {
                '_'
            });
            origins.push(index);
        }
        previous = current;
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

    #[test]
    fn marks_superscripts_and_subscripts_in_the_form_for_numbers() {
        let text = "10\u{207b}\u{00b3} M, IC\u{2085}\u{2080}";
        assert_eq!(normalize(text).as_str(), "10-3 M, IC50");
        let marked = normalize_numbers(text);
        assert_eq!(marked.as_str(), "10^-3 M, IC_50");
        assert_eq!(marked.original(0..5), Some(0..4));
        assert_eq!(marked.original(11..14), Some(10..12));
    }

    #[test]
    fn maps_a_normalized_range_back_to_the_original_text() {
        let normalized = normalize("a\u{00AD}b  7.3 \u{00B5}M");
        assert_eq!(normalized.as_str(), "ab 7.3 \u{03BC}M");
        assert_eq!(normalized.original(3..6), Some(5..8));
        assert_eq!(normalized.original(0..0), None);
        assert_eq!(normalized.original(8..12), None);
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
