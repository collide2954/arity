use std::collections::HashSet;

use arity_text::{Block, BlockKind};
use roxmltree::Node;

use crate::Supplement;
use crate::flatten::{child, children, href, is, name, non_empty, text};
use crate::table;

pub(crate) struct Built {
    pub(crate) blocks: Vec<Block>,
    pub(crate) supplements: Vec<Supplement>,
}

#[derive(Default)]
struct Counters {
    paragraphs: u32,
    items: u32,
    formulas: u32,
    sections: u32,
}

#[derive(Default)]
struct Builder {
    blocks: Vec<Block>,
}

impl Builder {
    fn push(&mut self, key: String, kind: BlockKind, section: &[String], text: String) {
        let mut block = Block::new(key, kind, text);
        block.section = section.to_vec();
        self.blocks.push(block);
    }

    fn walk(&mut self, node: Node<'_, '_>, scope: &str, section: &[String]) {
        let mut counters = Counters::default();
        self.walk_children(node, scope, section, &mut counters);
    }

    fn walk_children(
        &mut self,
        node: Node<'_, '_>,
        scope: &str,
        section: &[String],
        counters: &mut Counters,
    ) {
        for element in node.children().filter(Node::is_element) {
            match name(element) {
                "title" => {
                    if let Some(title) = non_empty(text(element)) {
                        self.push(format!("{scope}/title"), BlockKind::Heading, section, title);
                    }
                }
                "sec" => {
                    counters.sections += 1;
                    let child_scope = section_scope(scope, counters.sections);
                    let mut child_section = section.to_vec();
                    if let Some(title) = child(element, "title").map(text).and_then(non_empty) {
                        child_section.push(title);
                    }
                    self.walk(element, &child_scope, &child_section);
                }
                "p" => {
                    counters.paragraphs += 1;
                    let body = text(element);
                    if !body.is_empty() {
                        let key = format!("{scope}/p{}", counters.paragraphs);
                        self.push(key, BlockKind::Paragraph, section, body);
                    }
                }
                "list" => {
                    for item in element.descendants().filter(|node| is(*node, "list-item")) {
                        counters.items += 1;
                        let body = text(item);
                        if !body.is_empty() {
                            let key = format!("{scope}/li{}", counters.items);
                            self.push(key, BlockKind::ListItem, section, body);
                        }
                    }
                }
                "disp-formula" => {
                    counters.formulas += 1;
                    let body = text(element);
                    if !body.is_empty() {
                        let key = format!("{scope}/eq{}", counters.formulas);
                        self.push(key, BlockKind::Formula, section, body);
                    }
                }
                "boxed-text" | "disp-quote" | "statement" | "def-list" | "glossary" => {
                    self.walk_children(element, scope, section, counters);
                }
                _ => {}
            }
        }
    }
}

fn section_scope(scope: &str, index: u32) -> String {
    let is_section = scope
        .strip_prefix('s')
        .is_some_and(|rest| rest.starts_with(|c: char| c.is_ascii_digit()));
    match scope {
        "body" => format!("s{index}"),
        _ if is_section => format!("{scope}.{index}"),
        _ => format!("{scope}.s{index}"),
    }
}

pub(crate) fn build(article: Node<'_, '_>) -> Built {
    let mut builder = Builder::default();
    let meta = child(article, "front").and_then(|front| child(front, "article-meta"));

    if let Some(title) = meta
        .and_then(|meta| child(meta, "title-group"))
        .and_then(|group| child(group, "article-title"))
        .map(text)
        .and_then(non_empty)
    {
        builder.push("title".to_owned(), BlockKind::Title, &[], title);
    }

    if let Some(meta) = meta {
        for (index, abstract_node) in children(meta, "abstract").enumerate() {
            let scope = if index == 0 {
                "abstract".to_owned()
            } else {
                format!("abstract{}", index + 1)
            };
            let heading = child(abstract_node, "title")
                .map(text)
                .and_then(non_empty)
                .unwrap_or_else(|| "Abstract".to_owned());
            let start = builder.blocks.len();
            builder.walk(abstract_node, &scope, &[heading]);
            for block in &mut builder.blocks[start..] {
                if block.kind == BlockKind::Paragraph {
                    block.kind = BlockKind::Abstract;
                }
            }
        }
    }

    if let Some(body) = child(article, "body") {
        builder.walk(body, "body", &[]);
    }

    if let Some(back) = child(article, "back") {
        walk_back(&mut builder, back);
    }

    let mut blocks = builder.blocks;
    blocks.extend(tables(article));
    blocks.extend(figures(article));
    let (supplement_blocks, supplements) = supplements(article);
    blocks.extend(supplement_blocks);
    Built {
        blocks,
        supplements,
    }
}

fn walk_back(builder: &mut Builder, back: Node<'_, '_>) {
    let mut appendices = 0;
    let mut notes = 0;
    let mut sections = 0;
    let mut footnotes = 0;
    for element in back.children().filter(Node::is_element) {
        match name(element) {
            "ack" => builder.walk(element, "ack", &["Acknowledgments".to_owned()]),
            "app-group" | "app" => {
                let apps: Vec<Node<'_, '_>> = if name(element) == "app" {
                    vec![element]
                } else {
                    children(element, "app").collect()
                };
                for app in apps {
                    appendices += 1;
                    let title = child(app, "title")
                        .map(text)
                        .and_then(non_empty)
                        .unwrap_or_else(|| format!("Appendix {appendices}"));
                    builder.walk(app, &format!("app{appendices}"), &[title]);
                }
            }
            "notes" => {
                notes += 1;
                let title = child(element, "title")
                    .map(text)
                    .and_then(non_empty)
                    .unwrap_or_else(|| "Notes".to_owned());
                builder.walk(element, &format!("notes{notes}"), &[title]);
            }
            "sec" => {
                sections += 1;
                let title = child(element, "title").map(text).and_then(non_empty);
                builder.walk(
                    element,
                    &format!("back.s{sections}"),
                    &title.into_iter().collect::<Vec<_>>(),
                );
            }
            "fn-group" => {
                for footnote in children(element, "fn") {
                    footnotes += 1;
                    if let Some(body) = non_empty(text(footnote)) {
                        builder.push(
                            format!("fn{footnotes}"),
                            BlockKind::Footnote,
                            &["Footnotes".to_owned()],
                            body,
                        );
                    }
                }
            }
            _ => {}
        }
    }
}

fn outside_references(node: Node<'_, '_>) -> bool {
    !node.ancestors().any(|ancestor| is(ancestor, "ref-list"))
}

fn caption_text(node: Node<'_, '_>) -> String {
    child(node, "caption").map_or_default(text)
}

fn label(node: Node<'_, '_>) -> Option<String> {
    child(node, "label").map(text).and_then(non_empty)
}

fn tables(article: Node<'_, '_>) -> Vec<Block> {
    let mut blocks = Vec::new();
    let wraps = article
        .descendants()
        .filter(|node| is(*node, "table-wrap") && outside_references(*node));
    for (index, wrap) in wraps.enumerate() {
        let key = format!("t{}", index + 1);
        let mut caption = Block::new(
            format!("{key}/caption"),
            BlockKind::TableCaption,
            caption_text(wrap),
        );
        caption.label = label(wrap);
        blocks.push(caption);
        let mut last_row = 0;
        for table_node in wrap.descendants().filter(|node| is(*node, "table")) {
            if table_node
                .ancestors()
                .skip(1)
                .any(|ancestor| is(ancestor, "table"))
            {
                continue;
            }
            let (cells, row) = table::cells(table_node, &key, last_row);
            blocks.extend(cells);
            last_row = row;
        }
        if let Some(foot) = child(wrap, "table-wrap-foot") {
            let in_footnote = |node: &Node<'_, '_>| node.ancestors().skip(1).any(|a| is(a, "fn"));
            let notes = foot
                .descendants()
                .filter(|node| is(*node, "fn") || (is(*node, "p") && !in_footnote(node)));
            for (note_index, note) in notes.enumerate() {
                if let Some(body) = non_empty(text(note)) {
                    blocks.push(Block::new(
                        format!("{key}/foot{}", note_index + 1),
                        BlockKind::TableFootnote,
                        body,
                    ));
                }
            }
        }
    }
    blocks
}

fn figures(article: Node<'_, '_>) -> Vec<Block> {
    article
        .descendants()
        .filter(|node| is(*node, "fig") && outside_references(*node))
        .enumerate()
        .map(|(index, figure)| {
            let mut block = Block::new(
                format!("f{}/caption", index + 1),
                BlockKind::FigureCaption,
                caption_text(figure),
            );
            block.label = label(figure);
            block
        })
        .collect()
}

fn supplements(article: Node<'_, '_>) -> (Vec<Block>, Vec<Supplement>) {
    let mut seen = HashSet::new();
    let mut blocks = Vec::new();
    let mut found = Vec::new();
    for material in article
        .descendants()
        .filter(|node| is(*node, "supplementary-material") && outside_references(*node))
    {
        let media = child(material, "media");
        let Some(target) = media.and_then(href).or_else(|| href(material)) else {
            continue;
        };
        if !seen.insert(target.to_owned()) {
            continue;
        }
        let key = format!("supp{}", found.len() + 1);
        let caption = [Some(material), media]
            .into_iter()
            .flatten()
            .map(caption_text)
            .find(|caption| !caption.is_empty())
            .unwrap_or_default();
        let label = label(material).or_else(|| media.and_then(label));
        let media_type = media.and_then(|media| {
            let main = media.attribute("mimetype")?;
            Some(match media.attribute("mime-subtype") {
                Some(sub) if !main.contains('/') => format!("{main}/{sub}"),
                _ => main.to_owned(),
            })
        });
        let mut block = Block::new(
            format!("{key}/caption"),
            BlockKind::SupplementCaption,
            caption.clone(),
        );
        block.label.clone_from(&label);
        blocks.push(block);
        found.push(Supplement {
            key,
            href: target.to_owned(),
            media_type,
            label,
            caption,
        });
    }
    (blocks, found)
}
