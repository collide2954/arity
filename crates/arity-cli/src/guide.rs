//! The guide for readers: the page "Read a work" of the book, so that the
//! guide that `arity guide` prints and the book never differ.

/// The guide for readers, as Markdown.
pub const GUIDE: &str = include_str!("../../../docs/src/reading.md");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_the_page_of_the_book_for_readers() {
        assert!(GUIDE.starts_with("# Read a work\n"));
        assert!(GUIDE.contains("arity submit --dry-run"));
        assert!(GUIDE.contains("## Correct a run"));
    }
}
