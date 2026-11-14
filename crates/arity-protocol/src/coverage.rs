//! The coverage of a text: the numbers that it states, and which of them the
//! quotes of the reports contain. Coverage tells a reader what is left to
//! read. It makes no statement about the source.

use std::collections::HashMap;
use std::hash::BuildHasher;
use std::ops::Range;

use arity_text::{BlockKind, Document, find_numbers, normalize};
use arity_units::Unit;
use serde::Serialize;

/// Words that the unit reader can read as a unit, but that are words in the
/// text of an article.
const WORDS: &[&str] = &[
    "a", "an", "and", "as", "at", "by", "in", "is", "it", "of", "on", "or", "per", "the", "to",
];

const SIGNS: &[char] = &[
    '=', '<', '>', '\u{2264}', '\u{2265}', '\u{2248}', '~', '\u{223C}', '\u{B1}',
];

/// The numbers of one kind of block, and how many of them a quote contains.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct Count {
    pub numbers: usize,
    pub quoted: usize,
}

/// A number that no quote contains.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Unquoted {
    pub block: String,
    /// The number as the text writes it.
    pub number: String,
    /// The range of the number in the text of the block, in Unicode scalar
    /// values.
    pub start: usize,
    pub end: usize,
}

/// How many numbers of a text the quotes of its reports contain.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Coverage {
    /// The numbers in the cells of tables and data files, other than header
    /// cells.
    pub cells: Count,
    /// The numbers in the other blocks that have a unit, a percent sign, or an
    /// uncertainty after them, or a sign such as `=` or `<` before them.
    pub text: Count,
    /// Each number that no quote contains, in the order of the text.
    pub unquoted: Vec<Unquoted>,
}

fn is_unit(token: &str) -> bool {
    !token.is_empty()
        && !WORDS.contains(&token.to_lowercase().as_str())
        && Unit::parse_reported(token).is_ok()
}

/// True when the number is a part of a name or a code, for example `IC50`,
/// `5o`, `CO2`, or `MCF-7`.
fn in_name(chars: &[char], range: &Range<usize>) -> bool {
    let before = range
        .start
        .checked_sub(1)
        .and_then(|index| chars.get(index));
    let after = chars.get(range.end);
    let hyphen_after_letter = before == Some(&'-')
        && range
            .start
            .checked_sub(2)
            .and_then(|index| chars.get(index))
            .is_some_and(|c| c.is_alphabetic());
    before.is_some_and(|c| c.is_alphabetic())
        || after.is_some_and(|c| c.is_alphabetic())
        || hyphen_after_letter
}

/// True when a unit, a percent sign, or an uncertainty follows the number, or
/// a sign such as `=` precedes it.
fn has_measure(chars: &[char], range: &Range<usize>) -> bool {
    let mut next = range.end;
    while chars.get(next) == Some(&' ') {
        next += 1;
    }
    if matches!(chars.get(next), Some('%' | '\u{B1}')) {
        return true;
    }
    let token: String = chars[next..].iter().take_while(|c| **c != ' ').collect();
    let token = token.trim_end_matches([',', '.', ';', ':', ')', ']', '}']);
    if is_unit(token) {
        return true;
    }
    chars[..range.start]
        .iter()
        .rev()
        .find(|c| **c != ' ')
        .is_some_and(|c| SIGNS.contains(c))
}

/// Counts the numbers of a text, and finds each number that no quote
/// contains. `quoted` holds the quoted ranges of each block, in Unicode scalar
/// values of the text of the block.
pub fn coverage<S: BuildHasher>(
    document: &Document,
    quoted: &HashMap<String, Vec<Range<usize>>, S>,
) -> Coverage {
    let mut coverage = Coverage::default();
    for block in &document.blocks {
        let in_cell = matches!(block.kind, BlockKind::TableCell | BlockKind::SheetCell);
        if block.cell.as_ref().is_some_and(|cell| cell.header)
            || matches!(block.kind, BlockKind::Title | BlockKind::Heading)
        {
            continue;
        }
        let normalized = normalize(&block.text);
        let chars: Vec<char> = normalized.as_str().chars().collect();
        let mut found = find_numbers(normalized.as_str());
        found.sort_by_key(|number| (number.range.start, usize::MAX - number.range.end));
        let mut counted: Vec<Range<usize>> = Vec::new();
        for number in found {
            let range = number.range;
            if counted
                .iter()
                .any(|other| other.start < range.end && range.start < other.end)
                || in_name(&chars, &range)
                || !(in_cell || has_measure(&chars, &range))
            {
                continue;
            }
            let Some(original) = normalized.original(range.clone()) else {
                continue;
            };
            counted.push(range.clone());
            let count = if in_cell {
                &mut coverage.cells
            } else {
                &mut coverage.text
            };
            count.numbers += 1;
            let is_quoted = quoted.get(&block.key).is_some_and(|ranges| {
                ranges
                    .iter()
                    .any(|quote| quote.start <= original.start && original.end <= quote.end)
            });
            if is_quoted {
                count.quoted += 1;
            } else {
                coverage.unquoted.push(Unquoted {
                    block: block.key.clone(),
                    number: chars[range].iter().collect(),
                    start: original.start,
                    end: original.end,
                });
            }
        }
    }
    coverage
}
