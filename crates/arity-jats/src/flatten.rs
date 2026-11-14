use roxmltree::Node;

pub(crate) const XLINK: &str = "http://www.w3.org/1999/xlink";

/// Elements that are not part of the text that surrounds them. arity reads
/// them as their own blocks, or not at all.
const NOT_TEXT: &[&str] = &[
    "table-wrap",
    "table-wrap-group",
    "fig",
    "fig-group",
    "supplementary-material",
    "graphic",
    "inline-graphic",
    "media",
    "object-id",
    "ref-list",
];

/// Elements that break words apart.
const BLOCK_LEVEL: &[&str] = &[
    "p",
    "list",
    "list-item",
    "disp-formula",
    "disp-quote",
    "title",
    "label",
    "caption",
    "def-list",
    "def-item",
    "term",
    "def",
    "boxed-text",
    "statement",
    "td",
    "th",
    "tr",
    "fn",
];

pub(crate) fn name<'a>(node: Node<'a, '_>) -> &'a str {
    node.tag_name().name()
}

pub(crate) fn is(node: Node<'_, '_>, tag: &str) -> bool {
    node.is_element() && name(node) == tag
}

pub(crate) fn child<'a, 'input>(node: Node<'a, 'input>, tag: &str) -> Option<Node<'a, 'input>> {
    node.children().find(|candidate| is(*candidate, tag))
}

pub(crate) fn children<'a, 'input>(
    node: Node<'a, 'input>,
    tag: &'static str,
) -> impl Iterator<Item = Node<'a, 'input>> {
    node.children().filter(move |candidate| is(*candidate, tag))
}

pub(crate) fn href<'a>(node: Node<'a, '_>) -> Option<&'a str> {
    node.attribute((XLINK, "href"))
}

pub(crate) fn non_empty(text: String) -> Option<String> {
    (!text.is_empty()).then_some(text)
}

/// The text of an element, with each run of white space as one space.
pub(crate) fn text(node: Node<'_, '_>) -> String {
    let mut raw = String::new();
    push_children(node, &mut raw);
    raw.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn push_children(node: Node<'_, '_>, out: &mut String) {
    for child in node.children() {
        if child.is_text() {
            out.push_str(child.text().unwrap_or_default());
        } else if child.is_element() {
            let tag = name(child);
            if NOT_TEXT.contains(&tag) {
                continue;
            }
            match tag {
                "break" => out.push(' '),
                "alternatives" => {
                    if let Some(choice) = preferred_alternative(child) {
                        push_element(choice, out);
                    }
                }
                _ => push_element(child, out),
            }
        }
    }
}

fn push_element(node: Node<'_, '_>, out: &mut String) {
    let block = BLOCK_LEVEL.contains(&name(node));
    if block {
        out.push(' ');
    }
    push_children(node, out);
    if block {
        out.push(' ');
    }
}

fn preferred_alternative<'a, 'input>(node: Node<'a, 'input>) -> Option<Node<'a, 'input>> {
    let elements: Vec<Node<'a, 'input>> = node.children().filter(Node::is_element).collect();
    ["math", "table", "tex-math"]
        .iter()
        .find_map(|wanted| elements.iter().copied().find(|e| name(*e) == *wanted))
        .or_else(|| {
            elements
                .iter()
                .copied()
                .find(|e| !NOT_TEXT.contains(&name(*e)))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flatten(xml: &str) -> String {
        let document = roxmltree::Document::parse(xml).unwrap();
        text(document.root_element())
    }

    #[test]
    fn keeps_inline_markup_as_plain_text() {
        assert_eq!(
            flatten("<p>The IC<sub>50</sub> was <bold>3.2</bold>\n   μM (<xref>Fig 2</xref>).</p>"),
            "The IC50 was 3.2 μM (Fig 2)."
        );
        assert_eq!(flatten("<p>9 × 10<sup>3</sup> cells</p>"), "9 × 103 cells");
    }

    #[test]
    fn leaves_out_floats_and_graphics() {
        assert_eq!(
            flatten(
                "<p>See below.<table-wrap><caption><p>Table</p></caption></table-wrap><graphic/></p>"
            ),
            "See below."
        );
    }

    #[test]
    fn separates_block_level_elements() {
        assert_eq!(
            flatten(
                "<p>Items<list><list-item><p>one</p></list-item><list-item><p>two</p></list-item></list></p>"
            ),
            "Items one two"
        );
        assert_eq!(flatten("<p>line<break/>break</p>"), "line break");
    }

    #[test]
    fn prefers_mathml_in_alternatives() {
        let xml = r#"<p>Here <alternatives><tex-math>x^2</tex-math><math xmlns="http://www.w3.org/1998/Math/MathML"><mi>x</mi><mn>2</mn></math><inline-graphic/></alternatives> ends.</p>"#;
        assert_eq!(flatten(xml), "Here x2 ends.");
    }
}
