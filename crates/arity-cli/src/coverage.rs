use std::fmt::Write;

use arity_protocol::{Count, Coverage};
use arity_store::WorkId;
use arity_text::Document;

use crate::context::Context;
use crate::error::{CliError, Result};

/// The length of a block that `render` prints in full.
const SHORT: usize = 80;
/// The number of characters that `render` prints before and after a number.
const CONTEXT: usize = 40;

/// The coverage of the newest text of a work.
pub fn of_work(context: &Context, work: WorkId) -> Result<(Document, Coverage)> {
    let (text, document) = context.store.latest_text(work)?.ok_or_else(|| {
        CliError::Usage("the work has no text yet; run `arity fetch` first".to_owned())
    })?;
    let quoted = context.store.quoted_ranges(text)?;
    let skipped = context.store.skipped_ranges(text)?;
    let coverage = arity_protocol::coverage(&document, &quoted, &skipped);
    Ok((document, coverage))
}

fn excerpt(text: &str, start: usize, end: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    let from = start.saturating_sub(CONTEXT);
    let to = (end + CONTEXT).min(chars.len());
    let mut excerpt: String = chars[from..to].iter().collect();
    if from > 0 {
        excerpt.insert(0, '…');
    }
    if to < chars.len() {
        excerpt.push('…');
    }
    excerpt
}

fn count_line(name: &str, count: &Count) -> String {
    format!(
        "{name}: {} numbers. Reports quote {}, readers skipped {}, and {} are left.\n",
        count.numbers,
        count.quoted,
        count.skipped,
        count
            .numbers
            .saturating_sub(count.quoted)
            .saturating_sub(count.skipped)
    )
}

/// The coverage as text: the counts, and then each block with the numbers
/// that no report quotes and no reader skipped.
pub fn render(document: &Document, coverage: &Coverage) -> String {
    let mut out = count_line("Table cells", &coverage.cells);
    out.push_str(&count_line("Text with a unit or a sign", &coverage.text));
    let mut index = 0;
    while let Some(first) = coverage.unquoted.get(index) {
        let numbers: Vec<_> = coverage.unquoted[index..]
            .iter()
            .take_while(|unquoted| unquoted.block == first.block)
            .collect();
        index += numbers.len();
        let text = document
            .block(&first.block)
            .map_or("", |block| block.text.as_str());
        if text.chars().count() <= SHORT {
            let list: Vec<&str> = numbers
                .iter()
                .map(|unquoted| unquoted.number.as_str())
                .collect();
            let _ = writeln!(out, "\n[{}] {text}  ({})", first.block, list.join(", "));
        } else {
            let _ = writeln!(out, "\n[{}]", first.block);
            for unquoted in numbers {
                let _ = writeln!(
                    out,
                    "  {}: {}",
                    unquoted.number,
                    excerpt(text, unquoted.start, unquoted.end)
                );
            }
        }
    }
    out
}
