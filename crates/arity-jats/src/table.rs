use std::collections::HashSet;

use arity_text::{Block, BlockKind, Cell};
use roxmltree::Node;

use crate::Version;
use crate::flatten::{is, name, text};

const MAX_SPAN: u32 = 100;

/// Reads the cells of a `<table>` into blocks, with grid positions that take
/// row and column spans into account.
pub(crate) fn cells(
    table: Node<'_, '_>,
    table_key: &str,
    first_row: u32,
    version: Version,
) -> (Vec<Block>, u32) {
    let mut blocks = Vec::new();
    let mut occupied: HashSet<(u32, u32)> = HashSet::new();
    let mut row = first_row;
    for tr in table
        .descendants()
        .filter(|node| is(*node, "tr") && nearest_table(*node) == Some(table))
    {
        row += 1;
        let in_head = tr.ancestors().any(|ancestor| is(ancestor, "thead"));
        let mut column = 1;
        for cell in tr
            .children()
            .filter(|node| is(*node, "td") || is(*node, "th"))
        {
            while occupied.contains(&(row, column)) {
                column += 1;
            }
            let row_span = span(cell, "rowspan");
            let column_span = span(cell, "colspan");
            for r in row..row + row_span {
                for c in column..column + column_span {
                    occupied.insert((r, c));
                }
            }
            let mut block = Block::new(
                format!("{table_key}/r{row}c{column}"),
                BlockKind::TableCell,
                text(cell, version),
            );
            block.cell = Some(Cell {
                table: table_key.to_owned(),
                row,
                column,
                header: in_head || name(cell) == "th",
                row_span,
                column_span,
            });
            blocks.push(block);
            column += column_span;
        }
    }
    (blocks, row)
}

fn nearest_table<'a, 'input>(node: Node<'a, 'input>) -> Option<Node<'a, 'input>> {
    node.ancestors().find(|ancestor| is(*ancestor, "table"))
}

fn span(cell: Node<'_, '_>, attribute: &str) -> u32 {
    cell.attribute(attribute)
        .and_then(|value| value.trim().parse::<u32>().ok())
        .unwrap_or(1)
        .clamp(1, MAX_SPAN)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(xml: &str) -> Vec<(String, String, bool, u32, u32)> {
        let document = roxmltree::Document::parse(xml).unwrap();
        let (blocks, _) = cells(document.root_element(), "t1", 0, Version::V1);
        blocks
            .into_iter()
            .map(|block| {
                let cell = block.cell.unwrap();
                (
                    block.key,
                    block.text,
                    cell.header,
                    cell.row_span,
                    cell.column_span,
                )
            })
            .collect()
    }

    #[test]
    fn places_cells_on_a_grid_with_spans() {
        let xml = r#"<table>
            <thead>
              <tr><th rowspan="2">Compd.</th><th colspan="2">IC<sub>50</sub> (μM)</th></tr>
              <tr><th>MCF-7</th><th>PC-3</th></tr>
            </thead>
            <tbody>
              <tr><td>5a</td><td>19.53 ± 1.05</td><td>NA</td></tr>
            </tbody>
          </table>"#;
        assert_eq!(
            read(xml),
            [
                ("t1/r1c1".to_owned(), "Compd.".to_owned(), true, 2, 1),
                ("t1/r1c2".to_owned(), "IC50 (μM)".to_owned(), true, 1, 2),
                ("t1/r2c2".to_owned(), "MCF-7".to_owned(), true, 1, 1),
                ("t1/r2c3".to_owned(), "PC-3".to_owned(), true, 1, 1),
                ("t1/r3c1".to_owned(), "5a".to_owned(), false, 1, 1),
                ("t1/r3c2".to_owned(), "19.53 ± 1.05".to_owned(), false, 1, 1),
                ("t1/r3c3".to_owned(), "NA".to_owned(), false, 1, 1),
            ]
        );
    }

    #[test]
    fn limits_absurd_spans_and_ignores_bad_values() {
        let xml = r#"<table><tr><td rowspan="0">a</td><td colspan="x">b</td><td colspan="9999">c</td></tr></table>"#;
        let rows = read(xml);
        assert_eq!(rows[0].3, 1);
        assert_eq!(rows[1].4, 1);
        assert_eq!(rows[2].4, MAX_SPAN);
    }

    #[test]
    fn skips_rows_of_nested_tables() {
        let xml = "<table><tr><td>outer<table><tr><td>inner</td></tr></table></td></tr></table>";
        let rows = read(xml);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, "t1/r1c1");
    }
}
