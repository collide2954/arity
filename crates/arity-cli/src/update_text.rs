//! Makes the text of a work again with the latest version of the parser, and
//! moves the current runs of the work to the new text.
//!
//! A run moves when each of its quotes occurs in the new text as often as in
//! the old text. Each quote then becomes the exact words of the new text at
//! the same place, and the new run passes the same checks as each other
//! submission. When one run of a work cannot move, no run of the work moves,
//! so the current runs of a work always read one text.

use std::collections::HashMap;
use std::ops::Range;

use arity_core::Digest;
use arity_jats::Version;
use arity_protocol::{Severity, check};
use arity_store::{RetrievalRole, RunId, WorkId};
use arity_text::{Document, find_numbers, normalize, normalize_numbers};
use jiff::Timestamp;
use serde_json::{Map, Value, json};

use crate::context::Context;
use crate::error::{CliError, Result};
use crate::fetch::attach_sheet;

/// What arity does, or would do in a dry run, for one work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Update {
    /// The newest text of the work is from the latest version of the parser.
    Current,
    /// Each current run moves to the new text.
    Moved {
        text: Digest,
        /// The number of blocks whose text changes.
        blocks: usize,
        runs: Vec<MovedRun>,
    },
    /// A run cannot move, so nothing changes.
    Blocked { problems: Vec<Problem> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovedRun {
    pub from: RunId,
    /// The new run. A dry run stores no run.
    pub to: Option<RunId>,
    /// The number of quotes whose words change.
    pub quotes: usize,
    /// The number of skipped quotes that the new run leaves out, because
    /// the new text has no number in them, for example in the subscript of a
    /// chemical formula.
    pub skips: usize,
}

/// A submission that arity wrote again for the new text.
struct Rewritten {
    bytes: Vec<u8>,
    quotes: usize,
    skips: usize,
}

/// A part of a run that cannot move to the new text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub run: RunId,
    pub path: String,
    pub message: String,
}

pub fn update_text(
    context: &Context,
    work: WorkId,
    dry_run: bool,
    now: Timestamp,
) -> Result<Update> {
    let Some((_, old)) = context.store.latest_text(work)? else {
        return Err(CliError::Usage(
            "the work has no text; run `arity fetch` first".to_owned(),
        ));
    };
    if old.parser == Version::LATEST.name() {
        return Ok(Update::Current);
    }
    let (retrieval, new) = make_text(context, work, &old)?;
    let new_digest = new
        .digest()
        .map_err(|error| CliError::Usage(error.to_string()))?;
    let old_digest = old
        .digest()
        .map_err(|error| CliError::Usage(error.to_string()))?;
    let submissions: HashMap<RunId, String> = context
        .store
        .runs()?
        .into_iter()
        .map(|run| (run.id, run.submission))
        .collect();
    let properties = context.store.properties()?;
    let mut problems = Vec::new();
    let mut moved = Vec::new();
    let mut alignments = Alignments::new();
    for run in context.store.run_views()? {
        if run.work != work.0 || run.replaced_by.is_some() {
            continue;
        }
        let id = RunId(run.id);
        if run.text_sha256 != old_digest.as_str() {
            problems.push(Problem {
                run: id,
                path: "/text".to_owned(),
                message: format!(
                    "the run reads the text {}, not the newest text {old_digest}",
                    run.text_sha256
                ),
            });
            continue;
        }
        let raw = submissions.get(&id).map_or("", String::as_str);
        match move_run(id, raw, (&old, &new, &new_digest), &mut alignments) {
            Err(found) => problems.extend(found),
            Ok(rewritten) => match check(&rewritten.bytes, &new, &properties) {
                Ok(checked) => moved.push((id, rewritten, checked)),
                Err(rejected) => {
                    problems.extend(
                        rejected
                            .issues
                            .into_iter()
                            .filter(|issue| issue.severity == Severity::Error)
                            .map(|issue| Problem {
                                run: id,
                                path: issue.path,
                                message: issue.message,
                            }),
                    );
                }
            },
        }
    }
    if !problems.is_empty() {
        return Ok(Update::Blocked { problems });
    }
    let blocks = changed_blocks(&old, &new);
    let text = new_digest;
    if dry_run {
        let runs = moved
            .into_iter()
            .map(|(from, rewritten, _)| MovedRun {
                from,
                to: None,
                quotes: rewritten.quotes,
                skips: rewritten.skips,
            })
            .collect();
        return Ok(Update::Moved { text, blocks, runs });
    }
    let new_text = context.store.add_text(work, retrieval, &new, now)?;
    let mut runs = Vec::with_capacity(moved.len());
    for (from, rewritten, checked) in moved {
        let to = context
            .store
            .store_submission(work, new_text, &rewritten.bytes, &checked, now)?;
        runs.push(MovedRun {
            from,
            to: Some(to),
            quotes: rewritten.quotes,
            skips: rewritten.skips,
        });
    }
    Ok(Update::Moved { text, blocks, runs })
}

/// Makes the text again from the files of the old text, with the latest
/// version of the parser.
fn make_text(
    context: &Context,
    work: WorkId,
    old: &Document,
) -> Result<(arity_store::RetrievalId, Document)> {
    let retrievals = context.store.retrievals(work)?;
    let (id, full_text) = retrievals
        .iter()
        .find(|(_, retrieval)| {
            retrieval.role == RetrievalRole::FullText && retrieval.digest == old.source
        })
        .ok_or_else(|| {
            CliError::Usage(format!(
                "the work has no full text with the digest {}",
                old.source
            ))
        })?;
    let article = arity_jats::parse_as(&context.blobs.get(&full_text.digest)?, Version::LATEST)?;
    let mut document = article.document;
    for supplement in &article.supplements {
        let data = retrievals.iter().find(|(_, retrieval)| {
            retrieval.role == RetrievalRole::Supplement
                && retrieval.supplement_key.as_deref() == Some(supplement.key.as_str())
                && retrieval.retrieved_at == full_text.retrieved_at
        });
        if let Some((_, data)) = data {
            attach_sheet(&mut document, supplement, &context.blobs.get(&data.digest)?)?;
        }
    }
    Ok((*id, document))
}

fn changed_blocks(old: &Document, new: &Document) -> usize {
    let before: HashMap<&str, &str> = old
        .blocks
        .iter()
        .map(|block| (block.key.as_str(), block.text.as_str()))
        .collect();
    new.blocks
        .iter()
        .filter(|block| before.get(block.key.as_str()) != Some(&block.text.as_str()))
        .count()
}

/// Writes the submission of a run again for the new text, or gives each
/// quote that cannot move.
fn move_run(
    run: RunId,
    raw: &str,
    (old, new, new_digest): (&Document, &Document, &Digest),
    alignments: &mut Alignments,
) -> std::result::Result<Rewritten, Vec<Problem>> {
    let problem = |path: String, message: String| Problem { run, path, message };
    let mut submission: Value = serde_json::from_str(raw).map_err(|error| {
        vec![problem(
            String::new(),
            format!("the submission is not JSON: {error}"),
        )]
    })?;
    let mut problems = Vec::new();
    let mut changed = 0;
    for (path, item) in quotes(&mut submission) {
        match move_quote(item, old, new, alignments) {
            Ok(moved) => changed += usize::from(moved),
            Err(message) => problems.push(problem(path, message)),
        }
    }
    if !problems.is_empty() {
        return Err(problems);
    }
    let Some(fields) = submission.as_object_mut() else {
        return Err(vec![problem(
            String::new(),
            "the submission is not a JSON object".to_owned(),
        )]);
    };
    let skips = leave_out_skips_without_numbers(fields);
    let mut parts = vec![moved_note(run)];
    if skips > 0 {
        parts.push(format!(
            "It leaves out {skips} skipped quotes that have no number in the new text, for example the subscripts of chemical formulas."
        ));
    }
    if let Some(earlier) = fields.get("note").and_then(Value::as_str) {
        parts.push(format!("The note of run {}: {earlier}", run.0));
    }
    let note = parts.join(" ");
    let mut written = Map::new();
    for (key, value) in std::mem::take(fields) {
        match key.as_str() {
            "note" | "replaces" => {}
            "text" => {
                written.insert(key, json!(new_digest.as_str()));
            }
            "responsible" => {
                written.insert(key, value);
                written.insert("note".to_owned(), json!(note));
                written.insert("replaces".to_owned(), json!(run.0));
            }
            _ => {
                written.insert(key, value);
            }
        }
    }
    *fields = written;
    let mut bytes = serde_json::to_vec_pretty(&submission)
        .map_err(|error| vec![problem(String::new(), error.to_string())])?;
    bytes.push(b'\n');
    Ok(Rewritten {
        bytes,
        quotes: changed,
        skips,
    })
}

/// Leaves out each skipped quote that has no number in the new text. The old
/// text wrote subscripts and superscripts as plain digits, so a reader had to
/// skip, for example, the 2 and the 6 of `CoV2O6`. Gives the number of
/// skipped quotes that it leaves out.
fn leave_out_skips_without_numbers(fields: &mut Map<String, Value>) -> usize {
    let Some(skipped) = fields.get_mut("skipped").and_then(Value::as_array_mut) else {
        return 0;
    };
    let before = skipped.len();
    skipped.retain(|item| {
        item["quote"]
            .as_str()
            .is_some_and(|quote| !find_numbers(normalize_numbers(quote).as_str()).is_empty())
    });
    let left_out = before - skipped.len();
    if skipped.is_empty() {
        fields.shift_remove("skipped");
    }
    left_out
}

fn moved_note(run: RunId) -> String {
    format!(
        "This run moves run {} to the text of {}. Its reports are the reports of run {}, with the words of the new text in their quotes.",
        run.0,
        Version::LATEST.name(),
        run.0
    )
}

/// Each place in a submission that has a quote: the mentions of the entities,
/// the evidence of the reports, and the skipped quotes, with its path.
fn quotes(submission: &mut Value) -> Vec<(String, &mut Value)> {
    let mut found = Vec::new();
    let Some(fields) = submission.as_object_mut() else {
        return found;
    };
    for (list, items) in fields.iter_mut() {
        let inner = match list.as_str() {
            "entities" => Some("mentions"),
            "reports" => Some("evidence"),
            "skipped" => None,
            _ => continue,
        };
        let Some(items) = items.as_array_mut() else {
            continue;
        };
        for (index, item) in items.iter_mut().enumerate() {
            let Some(inner) = inner else {
                found.push((format!("/{list}/{index}"), item));
                continue;
            };
            let Some(quotes) = item.get_mut(inner).and_then(Value::as_array_mut) else {
                continue;
            };
            for (place, quote) in quotes.iter_mut().enumerate() {
                found.push((format!("/{list}/{index}/{inner}/{place}"), quote));
            }
        }
    }
    found
}

/// Gives a quote the words of the new text at the same place. arity aligns
/// the old block with the new block, and the new quote goes from the first to
/// the last character of the old quote that the alignment keeps. Gives true
/// when the words change.
fn move_quote(
    item: &mut Value,
    old: &Document,
    new: &Document,
    alignments: &mut Alignments,
) -> std::result::Result<bool, String> {
    let block = item["block"].as_str().unwrap_or_default().to_owned();
    let quote = item["quote"].as_str().unwrap_or_default().to_owned();
    let occurrence = item["occurrence"]
        .as_u64()
        .and_then(|occurrence| usize::try_from(occurrence).ok())
        .unwrap_or(1);
    let (Some(before), Some(after)) = (old.block(&block), new.block(&block)) else {
        return Err(format!("the new text has no block {block}"));
    };
    let Some(range) = normalize(&before.text)
        .find(&quote)
        .into_iter()
        .nth(occurrence.saturating_sub(1))
    else {
        return Err(format!(
            "the old text has no quote {quote:?} in the block {block}"
        ));
    };
    let new_chars: Vec<char> = after.text.chars().collect();
    let kept = alignments.entry(block.clone()).or_insert_with(|| {
        let old_keys: Vec<String> = before.text.chars().map(key).collect();
        let new_keys: Vec<String> = new_chars.iter().copied().map(key).collect();
        align(&old_keys, &new_keys)
    });
    let Some(moved) = map_range(kept, &new_chars, range) else {
        return Err(format!(
            "no character of the quote {quote:?} is in the block {block} of the new text"
        ));
    };
    let words: String = new_chars[moved.clone()].iter().collect();
    let Some(place) = normalize(&after.text)
        .find(&words)
        .iter()
        .position(|found| *found == moved)
    else {
        return Err(format!(
            "arity cannot find the new quote {words:?} in the block {block}"
        ));
    };
    let changed = words != quote || place + 1 != occurrence;
    item["quote"] = json!(words);
    if place + 1 != occurrence {
        item["occurrence"] = json!(place + 1);
    }
    Ok(changed)
}

/// The alignment of each block of the old text with the same block of the
/// new text, for each block that a quote names.
type Alignments = HashMap<String, Vec<Option<usize>>>;

/// The form of a character that the alignment compares: its normal form, so
/// that "¹" and "1", or "−" and "-", are the same.
fn key(c: char) -> String {
    if c.is_whitespace() {
        " ".to_owned()
    } else {
        normalize(&c.to_string()).as_str().to_owned()
    }
}

/// Maps a range of the old characters to the new characters with their
/// alignment. The new range goes from the first to the last old character in
/// the range that the alignment keeps, and it includes each mark of a script,
/// `^{`, `_{`, or `}`, that it cuts.
fn map_range(kept: &[Option<usize>], new: &[char], range: Range<usize>) -> Option<Range<usize>> {
    let mut inside = range.filter_map(|index| kept.get(index).copied().flatten());
    let first = inside.next()?;
    let last = inside.next_back().unwrap_or(first);
    let mut start = first;
    if start >= 2 && new[start - 1] == '{' && matches!(new[start - 2], '^' | '_') {
        start -= 2;
    }
    let mut end = last + 1;
    let mut open = 0_usize;
    let mut index = start;
    while index < end || (open > 0 && index < new.len()) {
        match new[index] {
            '{' if index > 0 && matches!(new[index - 1], '^' | '_') => open += 1,
            '}' if open > 0 => open -= 1,
            _ => {}
        }
        index += 1;
        end = end.max(index);
    }
    Some(start..end)
}

/// Aligns two sequences with the algorithm of Myers. Gives, for each item of
/// the first sequence, the place of the same item in the second sequence
/// when the alignment keeps it.
fn align(old: &[String], new: &[String]) -> Vec<Option<usize>> {
    let max = old.len() + new.len();
    // The furthest place in the old sequence on each diagonal. The diagonal
    // `line` holds the places where the old place minus the new place is
    // `line - max`.
    let mut frontier: Vec<usize> = vec![0; 2 * max + 2];
    // Each step keeps the part of the frontier that its diagonals can reach.
    let mut trace: Vec<Vec<usize>> = Vec::new();
    let mut kept = vec![None; old.len()];
    'search: for depth in 0..=max {
        trace.push(frontier[max - depth..=max + depth].to_vec());
        for diagonal in (0..=2 * depth).step_by(2) {
            let line = max + diagonal - depth;
            let from_below =
                diagonal == 0 || (diagonal != 2 * depth && frontier[line - 1] < frontier[line + 1]);
            let mut old_at = if from_below {
                frontier[line + 1]
            } else {
                frontier[line - 1] + 1
            };
            let mut new_at = old_at + max - line;
            while old_at < old.len() && new_at < new.len() && old[old_at] == new[new_at] {
                old_at += 1;
                new_at += 1;
            }
            frontier[line] = old_at;
            if old_at >= old.len() && new_at >= new.len() {
                break 'search;
            }
        }
    }
    let (mut old_at, mut new_at) = (old.len(), new.len());
    for depth in (0..trace.len()).rev() {
        let window = &trace[depth];
        let before = |line: usize| window[line + depth - max];
        let line = max + old_at - new_at;
        if depth == 0 {
            while old_at > 0 && new_at > 0 {
                old_at -= 1;
                new_at -= 1;
                kept[old_at] = Some(new_at);
            }
            break;
        }
        let lowest = line == max - depth;
        let highest = line == max + depth;
        let from_below = lowest || (!highest && before(line - 1) < before(line + 1));
        let previous = if from_below { line + 1 } else { line - 1 };
        let previous_old = before(previous);
        let previous_new = previous_old + max - previous;
        let (start_old, start_new) = if from_below {
            (previous_old, previous_new + 1)
        } else {
            (previous_old + 1, previous_new)
        };
        while old_at > start_old && new_at > start_new {
            old_at -= 1;
            new_at -= 1;
            kept[old_at] = Some(new_at);
        }
        old_at = previous_old;
        new_at = previous_new;
    }
    kept
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The new quote of the first place of a quote in the old text.
    fn moved(old: &str, new: &str, quote: &str) -> String {
        let start = old[..old.find(quote).unwrap()].chars().count();
        let range = start..start + quote.chars().count();
        let old_keys: Vec<String> = old.chars().map(key).collect();
        let new_chars: Vec<char> = new.chars().collect();
        let new_keys: Vec<String> = new_chars.iter().copied().map(key).collect();
        let kept = align(&old_keys, &new_keys);
        new_chars[map_range(&kept, &new_chars, range).unwrap()]
            .iter()
            .collect()
    }

    #[test]
    fn moves_a_quote_with_the_marks_of_its_scripts() {
        assert_eq!(moved("NA b", "NA ^{b}", "NA b"), "NA ^{b}");
        assert_eq!(
            moved("a Mean ± SD", "^{a} Mean ± SD", "a Mean"),
            "^{a} Mean"
        );
        assert_eq!(
            moved(
                "the solvents59,60 and",
                "the solvents^{59,60} and",
                "solvents59,60"
            ),
            "solvents^{59,60}"
        );
        assert_eq!(moved("in cm−1 units", "in cm⁻¹ units", "cm−1"), "cm⁻¹");
    }

    #[test]
    fn leaves_out_a_script_after_the_end_of_the_quote() {
        assert_eq!(
            moved("0.24 ± 0.02a", "0.24 ± 0.02^{a}", "0.24 ± 0.02"),
            "0.24 ± 0.02"
        );
    }

    #[test]
    fn moves_a_quote_over_a_formula_that_the_new_text_writes_as_text() {
        let old =
            "ratio \\documentclass{minimal}\\begin{document}$$\\times$$\\end{document}100 here";
        let quote = "ratio \\documentclass{minimal}\\begin{document}$$\\times$$\\end{document}100";
        assert_eq!(moved(old, "ratio ×100 here", quote), "ratio ×100");
    }

    #[test]
    fn keeps_a_quote_of_an_unchanged_block() {
        assert_eq!(moved("same text", "same text", "text"), "text");
        assert_eq!(align(&[], &[]), Vec::<Option<usize>>::new());
    }
}
