use std::collections::HashSet;

use arity_core::Digest;
use serde::{Deserialize, Serialize};

pub const DOCUMENT_FORMAT: &str = "arity-text/1";

/// The canonical text of one source file, as a list of blocks.
///
/// A reader quotes from blocks, and arity checks each quote against the
/// block that the reader names. The digest of the serialized document
/// identifies the exact text that a reader saw.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub format: String,
    /// The digest of the source file that the parser read.
    pub source: Digest,
    /// The name and version of the parser.
    pub parser: String,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Block {
    pub key: String,
    pub kind: BlockKind,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub section: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cell: Option<Cell>,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockKind {
    Title,
    Abstract,
    Heading,
    Paragraph,
    ListItem,
    TableCaption,
    TableCell,
    TableFootnote,
    FigureCaption,
    SupplementCaption,
    SheetCell,
    Formula,
    Footnote,
}

impl BlockKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::Abstract => "abstract",
            Self::Heading => "heading",
            Self::Paragraph => "paragraph",
            Self::ListItem => "list_item",
            Self::TableCaption => "table_caption",
            Self::TableCell => "table_cell",
            Self::TableFootnote => "table_footnote",
            Self::FigureCaption => "figure_caption",
            Self::SupplementCaption => "supplement_caption",
            Self::SheetCell => "sheet_cell",
            Self::Formula => "formula",
            Self::Footnote => "footnote",
        }
    }
}

/// The position of a block in a table or a sheet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cell {
    /// The key of the table or the sheet.
    pub table: String,
    pub row: u32,
    pub column: u32,
    #[serde(default, skip_serializing_if = "is_false")]
    pub header: bool,
    #[serde(default = "one", skip_serializing_if = "is_one")]
    pub row_span: u32,
    #[serde(default = "one", skip_serializing_if = "is_one")]
    pub column_span: u32,
}

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde passes the field by reference"
)]
fn is_false(value: &bool) -> bool {
    !value
}

fn one() -> u32 {
    1
}

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde passes the field by reference"
)]
fn is_one(value: &u32) -> bool {
    *value == 1
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DocumentError {
    #[error("a block has an empty key")]
    EmptyKey,
    #[error("more than one block has the key {0:?}")]
    DuplicateKey(String),
    #[error("the document format {0:?} is not {DOCUMENT_FORMAT}")]
    UnknownFormat(String),
}

impl Document {
    pub fn new(source: Digest, parser: &str, blocks: Vec<Block>) -> Result<Self, DocumentError> {
        let document = Self {
            format: DOCUMENT_FORMAT.to_owned(),
            source,
            parser: parser.to_owned(),
            blocks,
        };
        document.validate()?;
        Ok(document)
    }

    pub fn validate(&self) -> Result<(), DocumentError> {
        if self.format != DOCUMENT_FORMAT {
            return Err(DocumentError::UnknownFormat(self.format.clone()));
        }
        let mut keys = HashSet::with_capacity(self.blocks.len());
        for block in &self.blocks {
            if block.key.is_empty() {
                return Err(DocumentError::EmptyKey);
            }
            if !keys.insert(block.key.as_str()) {
                return Err(DocumentError::DuplicateKey(block.key.clone()));
            }
        }
        Ok(())
    }

    pub fn block(&self, key: &str) -> Option<&Block> {
        self.blocks.iter().find(|block| block.key == key)
    }

    /// The canonical JSON form. Equal documents give equal bytes.
    pub fn to_canonical_json(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    pub fn digest(&self) -> Result<Digest, serde_json::Error> {
        Ok(Digest::of(&self.to_canonical_json()?))
    }
}

impl Block {
    pub fn new(key: impl Into<String>, kind: BlockKind, text: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            kind,
            section: Vec::new(),
            label: None,
            cell: None,
            text: text.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Document {
        let mut cell = Block::new("t1/r1c2", BlockKind::TableCell, "3.2 ± 0.4");
        cell.cell = Some(Cell {
            table: "t1".to_owned(),
            row: 1,
            column: 2,
            header: false,
            row_span: 1,
            column_span: 1,
        });
        let mut paragraph = Block::new("s2/p1", BlockKind::Paragraph, "No effect was seen.");
        paragraph.section = vec!["Results".to_owned()];
        Document::new(Digest::of(b"source"), "test 1", vec![paragraph, cell]).unwrap()
    }

    #[test]
    fn rejects_duplicate_and_empty_keys() {
        let block = Block::new("p1", BlockKind::Paragraph, "a");
        assert_eq!(
            Document::new(Digest::of(b""), "test", vec![block.clone(), block]),
            Err(DocumentError::DuplicateKey("p1".to_owned()))
        );
        assert_eq!(
            Document::new(
                Digest::of(b""),
                "test",
                vec![Block::new("", BlockKind::Title, "x")]
            ),
            Err(DocumentError::EmptyKey)
        );
    }

    #[test]
    fn finds_blocks_by_key() {
        let document = sample();
        assert_eq!(document.block("t1/r1c2").unwrap().text, "3.2 ± 0.4");
        assert!(document.block("missing").is_none());
    }

    #[test]
    fn serializes_without_default_fields() {
        let json = serde_json::to_value(sample()).unwrap();
        assert_eq!(
            json["blocks"][1]["cell"],
            serde_json::json!({ "table": "t1", "row": 1, "column": 2 })
        );
        assert!(json["blocks"][1].get("section").is_none());
        let back: Document = serde_json::from_value(json).unwrap();
        assert_eq!(back, sample());
    }

    #[test]
    fn equal_documents_have_equal_digests() {
        let digest = sample().digest().unwrap();
        assert_eq!(sample().digest().unwrap(), digest);
        let mut changed = sample();
        changed.blocks[0].text.push('!');
        assert_ne!(changed.digest().unwrap(), digest);
    }

    #[test]
    fn block_kind_names_match_the_json_names() {
        for kind in [
            BlockKind::Title,
            BlockKind::ListItem,
            BlockKind::TableCell,
            BlockKind::SupplementCaption,
            BlockKind::SheetCell,
        ] {
            assert_eq!(
                serde_json::to_value(kind).unwrap(),
                serde_json::json!(kind.as_str())
            );
        }
    }

    #[test]
    fn rejects_an_unknown_format_when_read() {
        let mut json = serde_json::to_value(sample()).unwrap();
        json["format"] = serde_json::json!("other/9");
        let document: Document = serde_json::from_value(json).unwrap();
        assert_eq!(
            document.validate(),
            Err(DocumentError::UnknownFormat("other/9".to_owned()))
        );
    }
}
