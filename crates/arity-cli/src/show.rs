use std::fmt::Write;

use arity_core::{Digest, WorkMetadata};
use arity_text::{Block, BlockKind, Document};

/// Writes the canonical text of a work for a reader: each block with its
/// key, each table row with the key of each cell.
pub fn render(metadata: &WorkMetadata, digest: &Digest, document: &Document) -> String {
    let mut out = String::new();
    let identifiers: Vec<String> = [
        metadata.pmcid.as_ref().map(ToString::to_string),
        metadata.doi.as_ref().map(|doi| format!("doi:{doi}")),
        metadata.pmid.as_ref().map(|pmid| format!("pmid:{pmid}")),
    ]
    .into_iter()
    .flatten()
    .collect();
    let _ = writeln!(out, "{}", identifiers.join(" · "));
    let _ = writeln!(out, "{}", metadata.title);
    let _ = writeln!(out, "{}", metadata.short_citation());
    let license = metadata
        .license
        .as_ref()
        .map_or_else(|| "not stated".to_owned(), arity_core::License::label);
    let _ = writeln!(out, "License: {license}");
    let _ = writeln!(out, "Text: {digest}");
    let _ = writeln!(
        out,
        "Put this digest in the \"text\" field of the submission. Quote the text exactly as it is below.",
    );

    let mut section: Option<String> = None;
    let mut index = 0;
    let blocks = &document.blocks;
    while index < blocks.len() {
        let block = &blocks[index];
        if let Some(cell) = &block.cell {
            let table = cell.table.clone();
            let end = blocks[index..]
                .iter()
                .position(|other| other.cell.as_ref().is_none_or(|c| c.table != table))
                .map_or(blocks.len(), |offset| index + offset);
            render_cells(&mut out, &blocks[index..end]);
            index = end;
            continue;
        }
        let heading = heading_of(block);
        if heading.is_some() && heading != section {
            let _ = writeln!(out, "\n== {}", heading.as_deref().unwrap_or_default());
            section = heading;
        }
        let label = block
            .label
            .as_ref()
            .map_or_else(String::new, |label| format!(" {label}:"));
        let _ = writeln!(out, "[{}]{label} {}", block.key, block.text);
        index += 1;
    }
    out
}

fn heading_of(block: &Block) -> Option<String> {
    match block.kind {
        BlockKind::Title => Some("Title".to_owned()),
        BlockKind::TableCaption | BlockKind::TableFootnote => {
            block.label.clone().or_else(|| Some("Table".to_owned()))
        }
        BlockKind::FigureCaption => Some("Figures".to_owned()),
        BlockKind::SupplementCaption => Some("Supplementary files".to_owned()),
        _ if block.section.is_empty() => None,
        _ => Some(block.section.join(" › ")),
    }
}

fn render_cells(out: &mut String, cells: &[Block]) {
    let mut row: Option<u32> = None;
    for block in cells {
        let Some(cell) = &block.cell else {
            continue;
        };
        if row == Some(cell.row) {
            out.push_str(" | ");
        } else {
            out.push('\n');
            row = Some(cell.row);
        }
        let header = if cell.header { "*" } else { "" };
        let _ = write!(out, "[{}]{header} {}", block.key, block.text);
    }
    out.push('\n');
}

/// The entities that earlier runs of a work name. A submission that gives an
/// entity the same id names the same entity.
pub fn entities(entities: &[arity_store::WorkEntity]) -> String {
    let mut out = String::from(
        "\nEntities that earlier runs name. Give an entity the same id to name the same entity:\n",
    );
    for entity in entities {
        let _ = writeln!(
            out,
            "  {}  {}  {}",
            entity.local_id, entity.kind, entity.label
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use arity_text::Cell;

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

    #[test]
    fn writes_each_block_with_its_key() {
        let mut paragraph = Block::new("s1/p1", BlockKind::Paragraph, "Cells were grown.");
        paragraph.section = vec!["Methods".to_owned()];
        let mut caption = Block::new("t1/caption", BlockKind::TableCaption, "IC50 values.");
        caption.label = Some("Table 1".to_owned());
        let document = Document::new(
            Digest::of(b"x"),
            "test",
            vec![
                Block::new("title", BlockKind::Title, "A title"),
                paragraph,
                caption,
                cell("t1/r1c1", 1, 1, true, "Compound"),
                cell("t1/r1c2", 1, 2, true, "IC50 (μM)"),
                cell("t1/r2c1", 2, 1, false, "DOX"),
                cell("t1/r2c2", 2, 2, false, "0.24"),
            ],
        )
        .unwrap();
        let metadata = WorkMetadata {
            title: "A title".to_owned(),
            pmcid: Some("PMC1".parse().unwrap()),
            ..WorkMetadata::default()
        };
        let digest = document.digest().unwrap();
        let text = render(&metadata, &digest, &document);
        assert!(text.contains(&format!("Text: {digest}")));
        assert!(text.contains("\n== Methods\n[s1/p1] Cells were grown.\n"));
        assert!(text.contains("\n== Table 1\n[t1/caption] Table 1: IC50 values.\n"));
        assert!(text.contains("\n[t1/r1c1]* Compound | [t1/r1c2]* IC50 (μM)\n"));
        assert!(text.contains("\n[t1/r2c1] DOX | [t1/r2c2] 0.24\n"));
    }
}
