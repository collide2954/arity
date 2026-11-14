#![cfg(test)]

use std::collections::HashMap;
use std::ops::Range;

use arity_core::Digest;
use arity_protocol::{Count, Coverage, coverage};
use arity_text::{Block, BlockKind, Cell, Document};

fn cell(key: &str, row: u32, column: u32, header: bool, text: &str) -> Block {
    let mut block = Block::new(key, BlockKind::TableCell, text);
    block.cell = Some(Cell {
        table: "t1".to_owned(),
        row,
        column,
        header,
        row_span: 1,
        column_span: 1,
    });
    block
}

fn document(blocks: Vec<Block>) -> Document {
    Document::new(Digest::of(b"source"), "test 1", blocks).unwrap()
}

fn quoted(entries: &[(&str, Range<usize>)]) -> HashMap<String, Vec<Range<usize>>> {
    let mut map: HashMap<String, Vec<Range<usize>>> = HashMap::new();
    for (block, range) in entries {
        map.entry((*block).to_owned())
            .or_default()
            .push(range.clone());
    }
    map
}

fn numbers(coverage: &Coverage) -> Vec<&str> {
    coverage
        .unquoted
        .iter()
        .map(|unquoted| unquoted.number.as_str())
        .collect()
}

#[test]
fn counts_the_numbers_of_data_cells_and_not_of_header_cells_or_names() {
    let text = document(vec![
        cell("t1/r1c2", 1, 2, true, "IC50 (μM)"),
        cell("t1/r2c1", 2, 1, false, "5o"),
        cell("t1/r2c2", 2, 2, false, "7.30 ± 0.84"),
        cell("t1/r3c1", 3, 1, false, "MCF-7"),
        cell("t1/r3c2", 3, 2, false, "19.53 ± 1.05"),
    ]);
    let result = coverage(&text, &quoted(&[("t1/r2c2", 0..11)]), &HashMap::new());
    assert_eq!(
        result.cells,
        Count {
            numbers: 4,
            quoted: 2,
            skipped: 0
        }
    );
    assert_eq!(result.text, Count::default());
    assert_eq!(numbers(&result), ["19.53", "1.05"]);
    assert_eq!(result.unquoted[0].block, "t1/r3c2");
    assert_eq!((result.unquoted[0].start, result.unquoted[0].end), (0, 5));
}

#[test]
fn counts_the_numbers_of_the_text_that_have_a_unit_or_a_sign() {
    let paragraph = "The IC50 was 0.24 \u{00B5}M (Fig. 3), 46.9% of the dye broke down \
                     (p = 0.02), and in 3 rows the cells died at 37 \u{00B0}C in 2 a [12].";
    let text = document(vec![Block::new("s1/p1", BlockKind::Paragraph, paragraph)]);
    let start = paragraph[..paragraph.find("0.24").unwrap()].chars().count();
    let result = coverage(
        &text,
        &quoted(&[("s1/p1", start..start + 6)]),
        &HashMap::new(),
    );
    assert_eq!(numbers(&result), ["46.9", "0.02", "37"]);
    assert_eq!(
        result.text,
        Count {
            numbers: 4,
            quoted: 1,
            skipped: 0
        }
    );
}

#[test]
fn finds_a_quoted_number_through_the_normal_form() {
    let paragraph = "The IC50 of dox\u{00AD}orubicin was  7.3 \u{00B5}M.";
    let text = document(vec![Block::new("s1/p1", BlockKind::Paragraph, paragraph)]);
    let start = paragraph.chars().take_while(|c| *c != '7').count();
    let unquoted = coverage(&text, &HashMap::new(), &HashMap::new());
    assert_eq!(
        (unquoted.unquoted[0].start, unquoted.unquoted[0].end),
        (start, start + 3)
    );
    let result = coverage(
        &text,
        &quoted(&[("s1/p1", start..start + 6)]),
        &HashMap::new(),
    );
    assert_eq!(
        result.text,
        Count {
            numbers: 1,
            quoted: 1,
            skipped: 0
        }
    );
}

#[test]
fn counts_one_number_for_a_form_with_more_than_one_reading() {
    let text = document(vec![cell("t1/r2c2", 2, 2, false, "1,234")]);
    let result = coverage(&text, &HashMap::new(), &HashMap::new());
    assert_eq!(result.cells.numbers, 1);
}

#[test]
fn counts_skipped_numbers_apart_from_quoted_numbers() {
    let paragraph = "Earlier work found Ki = 4.5 nM [15], and here the IC50 was 0.24 μM.";
    let text = document(vec![Block::new("s1/p1", BlockKind::Paragraph, paragraph)]);
    let skip = paragraph[..paragraph.find("Ki").unwrap()].chars().count();
    let result = coverage(
        &text,
        &HashMap::new(),
        &quoted(&[("s1/p1", skip..skip + 11)]),
    );
    assert_eq!(
        result.text,
        Count {
            numbers: 2,
            quoted: 0,
            skipped: 1
        }
    );
    assert_eq!(numbers(&result), ["0.24"]);
}

#[test]
fn does_not_count_the_numbers_of_an_address() {
    let paragraph =
        "See http://www.mdpi.com/1420-3049/21/6/762/s1 and www.example.org/2/3 for 5 mg.";
    let text = document(vec![Block::new("s1/p1", BlockKind::Paragraph, paragraph)]);
    let result = coverage(&text, &HashMap::new(), &HashMap::new());
    assert_eq!(numbers(&result), ["5"]);
}
