use roxmltree::Node;

use crate::Version;
use crate::script::{self, Script, group};
use crate::tex;

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
pub(crate) fn text(node: Node<'_, '_>, version: Version) -> String {
    let mut raw = String::new();
    push_children(node, version, &mut raw);
    collapse(&raw)
}

fn collapse(raw: &str) -> String {
    raw.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn push_children(node: Node<'_, '_>, version: Version, out: &mut String) {
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
                        push_element(choice, version, out);
                    }
                }
                _ => push_element(child, version, out),
            }
        }
    }
}

fn push_element(node: Node<'_, '_>, version: Version, out: &mut String) {
    if version >= Version::V2 && push_markup(node, version, out) {
        return;
    }
    let block = BLOCK_LEVEL.contains(&name(node));
    if block {
        out.push(' ');
    }
    push_children(node, version, out);
    if block {
        out.push(' ');
    }
}

/// The text of the children of an element, from version 2.
fn content(node: Node<'_, '_>, version: Version) -> String {
    text(node, version)
}

/// The text of each child element, from version 2: for example the base and
/// the script of a MathML superscript.
fn parts(node: Node<'_, '_>, version: Version) -> Vec<String> {
    node.children()
        .filter(Node::is_element)
        .map(|part| {
            let mut raw = String::new();
            push_element(part, version, &mut raw);
            collapse(&raw)
        })
        .collect()
}

/// Writes the markup that version 2 and later versions write as plain text:
/// the scripts, the parts of MathML formulas, and formulas in TeX. From
/// version 3, also small capitals as capitals, because a source can write the
/// M of a molar unit in small capitals, for example `n<sc>m</sc>` for nM.
/// Gives false for each other element.
fn push_markup(node: Node<'_, '_>, version: Version, out: &mut String) -> bool {
    match name(node) {
        "sup" => out.push_str(&script::write(&content(node, version), Script::Superscript)),
        "sub" => out.push_str(&script::write(&content(node, version), Script::Subscript)),
        "msqrt" => {
            out.push('√');
            out.push_str(&group(&content(node, version)));
        }
        "mfenced" => out.push_str(&fenced(node, version)),
        "sc" if version >= Version::V3 => {
            out.extend(
                content(node, version)
                    .chars()
                    .map(|c| c.to_ascii_uppercase()),
            );
        }
        "mtr" | "mtd" => {
            out.push(' ');
            push_children(node, version, out);
            out.push(' ');
        }
        "tex-math" => {
            let source: String = node
                .descendants()
                .filter(Node::is_text)
                .filter_map(|part| part.text())
                .collect();
            out.push_str(&tex::text(&source, version));
        }
        "annotation" | "annotation-xml" => {}
        tag @ ("msup" | "msub" | "msubsup" | "mfrac" | "mroot") => {
            let parts = parts(node, version);
            let written = match (tag, parts.as_slice()) {
                ("msup", [base, sup]) => {
                    format!("{base}{}", script::write(sup, Script::Superscript))
                }
                ("msub", [base, sub]) => format!("{base}{}", script::write(sub, Script::Subscript)),
                ("msubsup", [base, sub, sup]) => format!(
                    "{base}{}{}",
                    script::write(sub, Script::Subscript),
                    script::write(sup, Script::Superscript)
                ),
                ("mfrac", [numerator, denominator]) => {
                    format!("{}/{}", group(numerator), group(denominator))
                }
                ("mroot", [base, index]) => {
                    format!(
                        "{}√{}",
                        script::write(index, Script::Superscript),
                        group(base)
                    )
                }
                _ => return false,
            };
            out.push_str(&written);
        }
        _ => return false,
    }
    true
}

/// Writes a MathML `mfenced` with its brackets, and with its separators when
/// it names them. Sources often put each part of one expression in an
/// `mfenced` that names no separators, so arity does not add the default
/// comma.
fn fenced(node: Node<'_, '_>, version: Version) -> String {
    let separators: Vec<char> = node
        .attribute("separators")
        .unwrap_or_default()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    let mut out = node.attribute("open").unwrap_or("(").to_owned();
    for (index, part) in parts(node, version).iter().enumerate() {
        if index > 0
            && let Some(separator) = separators.get(index - 1).or(separators.last())
        {
            out.push(*separator);
        }
        out.push_str(part);
    }
    out.push_str(node.attribute("close").unwrap_or(")"));
    out
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
        text(document.root_element(), Version::V1)
    }

    fn flatten_2(xml: &str) -> String {
        let document = roxmltree::Document::parse(xml).unwrap();
        text(document.root_element(), Version::V2)
    }

    fn flatten_3(xml: &str) -> String {
        let document = roxmltree::Document::parse(xml).unwrap();
        text(document.root_element(), Version::V3)
    }

    #[test]
    fn version_3_writes_small_capitals_as_capitals() {
        let xml = "<p>IC<sub>50</sub> = 471 n<sc>m</sc>, 1.80 µ<sc>m</sc>, and 2 m<sc>m</sc> <sc>l</sc>-glutamine</p>";
        assert_eq!(
            flatten_3(xml),
            "IC₅₀ = 471 nM, 1.80 µM, and 2 mM L-glutamine"
        );
        assert_eq!(
            flatten_2(xml),
            "IC₅₀ = 471 nm, 1.80 µm, and 2 mm l-glutamine"
        );
        assert_eq!(flatten_3("<p><sc>Dna µ</sc></p>"), "DNA µ");
    }

    const MATHML: &str = r#"xmlns:mml="http://www.w3.org/1998/Math/MathML""#;

    #[test]
    fn version_2_writes_scripts_as_script_characters() {
        assert_eq!(
            flatten_2("<p>9 × 10<sup>3</sup> cells</p>"),
            "9 × 10³ cells"
        );
        assert_eq!(
            flatten_2("<p>The IC<sub>50</sub> was 3.2 μM.</p>"),
            "The IC₅₀ was 3.2 μM."
        );
        assert_eq!(
            flatten_2("<p>1 cm<sup>−2</sup> and Ca<sup>2+</sup></p>"),
            "1 cm⁻² and Ca²⁺"
        );
        assert_eq!(
            flatten_2("<p>0.24 ± 0.02<sup>a</sup>, λ<sub>max</sub>, refs<sup>12,13</sup></p>"),
            "0.24 ± 0.02^{a}, λ_{max}, refs^{12,13}"
        );
        assert_eq!(
            flatten_2("<p>cells<sup>1</sup><sup>,</sup><sup>2</sup></p>"),
            "cells¹,²"
        );
    }

    #[test]
    fn version_2_writes_the_parts_of_mathml_formulas() {
        let math = |body: &str| flatten_2(&format!("<p {MATHML}><mml:math>{body}</mml:math></p>"));
        assert_eq!(
            math("<mml:msup><mml:mi>x</mml:mi><mml:mn>2</mml:mn></mml:msup>"),
            "x²"
        );
        assert_eq!(
            math(
                "<mml:mfrac><mml:mn>1</mml:mn><mml:mrow><mml:mi>d</mml:mi><mml:mo>+</mml:mo><mml:mn>1</mml:mn></mml:mrow></mml:mfrac>"
            ),
            "1/(d+1)"
        );
        assert_eq!(
            math(
                "<mml:msqrt><mml:mn>2</mml:mn></mml:msqrt><mml:mfenced open=\"[\" close=\"]\" separators=\",\"><mml:mi>a</mml:mi><mml:mi>b</mml:mi></mml:mfenced>"
            ),
            "√2[a,b]"
        );
        assert_eq!(
            math(
                "<mml:mfenced><mml:mi>n</mml:mi><mml:mo>+</mml:mo><mml:mi>k</mml:mi></mml:mfenced>"
            ),
            "(n+k)"
        );
        assert_eq!(
            math(
                "<mml:msubsup><mml:mi>d</mml:mi><mml:mn>111</mml:mn><mml:mn>2</mml:mn></mml:msubsup>"
            ),
            "d₁₁₁²"
        );
    }

    #[test]
    fn version_2_writes_a_formula_that_has_only_tex() {
        let xml = r"<p>Here <inline-formula><tex-math>\documentclass[12pt]{minimal}\begin{document}$$\text{IC}_{50}=2.3\pm 0.4$$\end{document}</tex-math></inline-formula> ends.</p>";
        assert_eq!(flatten_2(xml), "Here IC₅₀=2.3±0.4 ends.");
        assert!(flatten(xml).contains("documentclass"));
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
