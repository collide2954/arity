//! Reads JATS XML articles.
//!
//! The parser gives the bibliographic metadata of the article and its
//! canonical text as blocks: the title, the abstract, each paragraph, each
//! table cell, each caption, and each supplementary file.

mod blocks;
mod flatten;
mod metadata;
mod table;

use arity_core::{Digest, Doi, Pmid, WorkMetadata};
use arity_text::{Document, DocumentError};

/// The name and version that each parsed document records.
pub const PARSER: &str = concat!("arity-jats ", env!("CARGO_PKG_VERSION"));

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

pub fn parse(bytes: &[u8]) -> Result<Article, JatsError> {
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
    let built = blocks::build(root);
    Ok(Article {
        metadata: metadata::read(root),
        document: Document::new(Digest::of(bytes), PARSER, built.blocks)?,
        supplements: built.supplements,
        related: metadata::related(root),
    })
}
