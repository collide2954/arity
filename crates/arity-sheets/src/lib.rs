//! Reads supplementary data files into text blocks.
//!
//! Each non-empty cell becomes a block with the key
//! `{supplement}/s{sheet}/r{row}c{column}`, so a reader can quote a cell
//! exactly as for a table in the article. Rows and columns start at 1.

use std::io::Cursor;

use arity_text::{Block, BlockKind, Cell};
use calamine::{Data, Reader, open_workbook_auto_from_rs};

/// The largest number of cells that arity reads from one file.
pub const MAX_CELLS: usize = 200_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Workbook,
    Delimited(u8),
}

#[derive(Debug, thiserror::Error)]
pub enum SheetError {
    #[error("cannot read the workbook: {0}")]
    Workbook(String),
    #[error("cannot read the delimited file: {0}")]
    Delimited(#[from] csv::Error),
    #[error("the file has more than {MAX_CELLS} cells")]
    TooLarge,
}

/// The format of a supplementary file, from its name. arity reads only
/// these formats.
pub fn format_of(file_name: &str) -> Option<Format> {
    let extension = file_name.rsplit('.').next()?.to_ascii_lowercase();
    match extension.as_str() {
        "xlsx" | "xlsm" | "xlsb" | "xls" | "ods" => Some(Format::Workbook),
        "csv" => Some(Format::Delimited(b',')),
        "tsv" | "tab" => Some(Format::Delimited(b'\t')),
        _ => None,
    }
}

fn cell_block(
    key: &str,
    sheet: usize,
    name: &str,
    row: usize,
    column: usize,
    text: String,
) -> Block {
    let table = format!("{key}/s{sheet}");
    let row = u32::try_from(row).unwrap_or(u32::MAX);
    let column = u32::try_from(column).unwrap_or(u32::MAX);
    let mut block = Block::new(
        format!("{table}/r{row}c{column}"),
        BlockKind::SheetCell,
        text,
    );
    block.label = Some(name.to_owned());
    block.cell = Some(Cell {
        table,
        row,
        column,
        header: false,
        row_span: 1,
        column_span: 1,
    });
    block
}

fn data_text(data: &Data) -> Option<String> {
    let text = match data {
        Data::Empty => return None,
        Data::String(text) => text.trim().to_owned(),
        Data::Float(value) => format!("{value}"),
        Data::Int(value) => value.to_string(),
        Data::Bool(value) => if *value { "TRUE" } else { "FALSE" }.to_owned(),
        Data::DateTime(value) => value.to_string(),
        Data::DateTimeIso(text) | Data::DurationIso(text) => text.clone(),
        Data::Error(error) => format!("#{error:?}"),
    };
    (!text.is_empty()).then_some(text)
}

/// Reads all non-empty cells of a file into blocks.
pub fn read(bytes: &[u8], format: Format, key: &str) -> Result<Vec<Block>, SheetError> {
    match format {
        Format::Workbook => read_workbook(bytes, key),
        Format::Delimited(delimiter) => read_delimited(bytes, delimiter, key),
    }
}

fn read_workbook(bytes: &[u8], key: &str) -> Result<Vec<Block>, SheetError> {
    let mut workbook = open_workbook_auto_from_rs(Cursor::new(bytes.to_vec()))
        .map_err(|error| SheetError::Workbook(error.to_string()))?;
    let names = workbook.sheet_names();
    let mut blocks = Vec::new();
    for (index, name) in names.iter().enumerate() {
        let range = workbook
            .worksheet_range(name)
            .map_err(|error| SheetError::Workbook(error.to_string()))?;
        let (first_row, first_column) = range.start().unwrap_or((0, 0));
        for (row_offset, row) in range.rows().enumerate() {
            for (column_offset, data) in row.iter().enumerate() {
                let Some(text) = data_text(data) else {
                    continue;
                };
                if blocks.len() >= MAX_CELLS {
                    return Err(SheetError::TooLarge);
                }
                let row_number = usize::try_from(first_row).unwrap_or(0) + row_offset + 1;
                let column_number = usize::try_from(first_column).unwrap_or(0) + column_offset + 1;
                blocks.push(cell_block(
                    key,
                    index + 1,
                    name,
                    row_number,
                    column_number,
                    text,
                ));
            }
        }
    }
    Ok(blocks)
}

fn read_delimited(bytes: &[u8], delimiter: u8, key: &str) -> Result<Vec<Block>, SheetError> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(false)
        .flexible(true)
        .from_reader(bytes);
    let mut blocks = Vec::new();
    for (row_index, record) in reader.records().enumerate() {
        let record = record?;
        for (column_index, field) in record.iter().enumerate() {
            let text = field.trim();
            if text.is_empty() {
                continue;
            }
            if blocks.len() >= MAX_CELLS {
                return Err(SheetError::TooLarge);
            }
            blocks.push(cell_block(
                key,
                1,
                "data",
                row_index + 1,
                column_index + 1,
                text.to_owned(),
            ));
        }
    }
    Ok(blocks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knows_the_formats_by_extension() {
        assert_eq!(format_of("data.XLSX"), Some(Format::Workbook));
        assert_eq!(format_of("pone.0291981.s001.ods"), Some(Format::Workbook));
        assert_eq!(format_of("values.csv"), Some(Format::Delimited(b',')));
        assert_eq!(format_of("values.tsv"), Some(Format::Delimited(b'\t')));
        assert_eq!(format_of("methods.pdf"), None);
        assert_eq!(format_of("noextension"), None);
    }

    #[test]
    fn reads_the_cells_of_a_csv_file() {
        let csv = "compound,IC50 (uM)\nDOX,\"0.24\"\ncisplatin,,\n";
        let blocks = read(csv.as_bytes(), Format::Delimited(b','), "supp1").unwrap();
        let cells: Vec<(&str, &str)> = blocks
            .iter()
            .map(|block| (block.key.as_str(), block.text.as_str()))
            .collect();
        assert_eq!(
            cells,
            [
                ("supp1/s1/r1c1", "compound"),
                ("supp1/s1/r1c2", "IC50 (uM)"),
                ("supp1/s1/r2c1", "DOX"),
                ("supp1/s1/r2c2", "0.24"),
                ("supp1/s1/r3c1", "cisplatin"),
            ]
        );
        let cell = blocks[3].cell.as_ref().unwrap();
        assert_eq!(
            (cell.table.as_str(), cell.row, cell.column),
            ("supp1/s1", 2, 2)
        );
        assert_eq!(blocks[3].kind, BlockKind::SheetCell);
    }

    #[test]
    fn reads_tab_separated_files() {
        let tsv = "a\tb\n1\t2\n";
        let blocks = read(tsv.as_bytes(), Format::Delimited(b'\t'), "supp2").unwrap();
        assert_eq!(blocks.len(), 4);
        assert_eq!(blocks[3].key, "supp2/s1/r2c2");
    }

    #[test]
    fn writes_spreadsheet_values_as_text() {
        assert_eq!(data_text(&Data::Float(0.24)).as_deref(), Some("0.24"));
        assert_eq!(data_text(&Data::Float(3.0)).as_deref(), Some("3"));
        assert_eq!(data_text(&Data::Int(7)).as_deref(), Some("7"));
        assert_eq!(data_text(&Data::Bool(true)).as_deref(), Some("TRUE"));
        assert_eq!(
            data_text(&Data::String("  x ".to_owned())).as_deref(),
            Some("x")
        );
        assert_eq!(data_text(&Data::Empty), None);
        assert_eq!(data_text(&Data::String("   ".to_owned())), None);
    }

    #[test]
    fn reads_each_sheet_of_a_workbook() {
        let mut workbook = rust_xlsxwriter::Workbook::new();
        let first = workbook.add_worksheet().set_name("IC50").unwrap();
        first.write_string(0, 0, "compound").unwrap();
        first.write_string(0, 1, "IC50 (μM)").unwrap();
        first.write_string(1, 0, "DOX").unwrap();
        first.write_number(1, 1, 0.24).unwrap();
        let second = workbook.add_worksheet().set_name("Raw").unwrap();
        second.write_number(2, 3, 17.0).unwrap();
        let bytes = workbook.save_to_buffer().unwrap();

        let blocks = read(&bytes, Format::Workbook, "supp1").unwrap();
        let cells: Vec<(&str, &str, &str)> = blocks
            .iter()
            .map(|block| {
                (
                    block.key.as_str(),
                    block.label.as_deref().unwrap(),
                    block.text.as_str(),
                )
            })
            .collect();
        assert_eq!(
            cells,
            [
                ("supp1/s1/r1c1", "IC50", "compound"),
                ("supp1/s1/r1c2", "IC50", "IC50 (μM)"),
                ("supp1/s1/r2c1", "IC50", "DOX"),
                ("supp1/s1/r2c2", "IC50", "0.24"),
                ("supp1/s2/r3c4", "Raw", "17"),
            ]
        );
    }

    #[test]
    fn rejects_a_file_that_is_not_a_workbook() {
        assert!(matches!(
            read(b"not a workbook", Format::Workbook, "supp1"),
            Err(SheetError::Workbook(_))
        ));
    }
}
