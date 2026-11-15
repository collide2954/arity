//! Reads JATS XML articles.
//!
//! The parser gives the bibliographic metadata of the article and its
//! canonical text as blocks: the title, the abstract, each paragraph, each
//! table cell, each caption, and each supplementary file.

mod blocks;
mod flatten;
mod metadata;
mod script;
mod table;
mod tex;

use arity_core::{Digest, Doi, Pmid, WorkMetadata};
use arity_text::{Document, DocumentError};

/// A version of the canonical text that the parser makes. Each document
/// records the name of its version, and the name is part of the digest of
/// the document. Thus a version never changes its output, and a rebuild
/// makes each archived text with the version that made it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Version {
    /// Writes superscripts and subscripts as plain characters, and a formula
    /// that has only TeX as its TeX source.
    V1,
    /// Writes the digits and the signs of superscripts and subscripts as
    /// script characters, for example `10⁴` and `IC₅₀`, and other scripts with
    /// a mark, for example `^{a}`. Writes the fractions, the roots, and the
    /// brackets of MathML formulas, and a formula that has only TeX as plain
    /// text.
    V2,
    /// Writes as version 2, and also writes the variant Greek letters of TeX,
    /// for example `\varDelta`, as their characters.
    V3,
}

impl Version {
    /// The version that makes new texts.
    pub const LATEST: Self = Self::V3;

    const ALL: [Self; 3] = [Self::V1, Self::V2, Self::V3];

    pub fn name(self) -> &'static str {
        match self {
            Self::V1 => "arity-jats 0.1.0",
            Self::V2 => "arity-jats 0.2.0",
            Self::V3 => "arity-jats 0.3.0",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|version| version.name() == name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Article {
    pub metadata: WorkMetadata,
    pub document: Document,
    pub supplements: Vec<Supplement>,
    pub related: Vec<RelatedArticle>,
}

/// A supplementary file that an article names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Supplement {
    pub key: String,
    pub href: String,
    pub media_type: Option<String>,
    pub label: Option<String>,
    pub caption: String,
}

/// A link from the article to another article, for example from a
/// retraction notice to the retracted article.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelatedArticle {
    /// The relation that the article states, for example `retracted-article`.
    pub relation: String,
    pub doi: Option<Doi>,
    pub pmid: Option<Pmid>,
    pub href: Option<String>,
    pub citation: String,
}

#[derive(Debug, thiserror::Error)]
pub enum JatsError {
    #[error("the file is not UTF-8 text")]
    Encoding,
    #[error("the file is not well-formed XML: {0}")]
    Xml(#[from] roxmltree::Error),
    #[error("the root element is <{0}>, not <article>")]
    NotAnArticle(String),
    #[error(transparent)]
    Document(#[from] DocumentError),
}

/// Reads an article with the latest version of the parser.
pub fn parse(bytes: &[u8]) -> Result<Article, JatsError> {
    parse_as(bytes, Version::LATEST)
}

/// Reads an article with one version of the parser.
pub fn parse_as(bytes: &[u8], version: Version) -> Result<Article, JatsError> {
    let text = std::str::from_utf8(bytes).map_err(|_| JatsError::Encoding)?;
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let options = roxmltree::ParsingOptions {
        allow_dtd: true,
        ..roxmltree::ParsingOptions::default()
    };
    let xml = roxmltree::Document::parse_with_options(text, options)?;
    let root = xml.root_element();
    if root.tag_name().name() != "article" {
        return Err(JatsError::NotAnArticle(root.tag_name().name().to_owned()));
    }
    let built = blocks::build(root, version);
    Ok(Article {
        metadata: metadata::read(root),
        document: Document::new(Digest::of(bytes), version.name(), built.blocks)?,
        supplements: built.supplements,
        related: metadata::related(root),
    })
}
