use arity_core::{Author, Doi, License, Pmid, WorkMetadata};
use jiff::civil::Date;
use roxmltree::Node;

use crate::flatten::{self, child, children, href, is, name, non_empty};
use crate::{RelatedArticle, Version};

/// The text of an element of the bibliographic record. The record is not part
/// of the digest of the text, so it uses the latest version of the parser.
fn text(node: Node<'_, '_>) -> String {
    flatten::text(node, Version::LATEST)
}

pub(crate) fn read(article: Node<'_, '_>) -> WorkMetadata {
    let mut work = WorkMetadata {
        article_type: article.attribute("article-type").map(str::to_owned),
        ..WorkMetadata::default()
    };
    work.is_preprint = work.article_type.as_deref() == Some("preprint");
    let front = child(article, "front");
    if let Some(journal) = front.and_then(|front| child(front, "journal-meta")) {
        work.journal = child(journal, "journal-title-group")
            .and_then(|group| child(group, "journal-title"))
            .or_else(|| child(journal, "journal-title"))
            .map(text)
            .and_then(non_empty);
        work.publisher = child(journal, "publisher")
            .and_then(|publisher| child(publisher, "publisher-name"))
            .map(text)
            .and_then(non_empty);
    }
    let Some(meta) = front.and_then(|front| child(front, "article-meta")) else {
        return work;
    };
    work.title = child(meta, "title-group")
        .and_then(|group| child(group, "article-title"))
        .map_or_default(text);
    read_identifiers(meta, &mut work);
    (work.published, work.year) = publication_date(meta);
    work.authors = authors(meta);
    read_permissions(meta, &mut work);
    work.keywords = children(meta, "kwd-group")
        .flat_map(|group| children(group, "kwd"))
        .map(text)
        .filter(|keyword| !keyword.is_empty())
        .collect();
    work
}

fn read_identifiers(meta: Node<'_, '_>, work: &mut WorkMetadata) {
    for id in children(meta, "article-id") {
        let value = text(id);
        match id.attribute("pub-id-type") {
            Some("doi") if work.doi.is_none() => work.doi = value.parse().ok(),
            Some("pmid") if work.pmid.is_none() => work.pmid = value.parse().ok(),
            Some("pmcid") if work.pmcid.is_none() => work.pmcid = value.parse().ok(),
            Some("pmc") if work.pmcid.is_none() => {
                work.pmcid = format!("PMC{}", value.trim_start_matches("PMC"))
                    .parse()
                    .ok();
            }
            _ => {}
        }
    }
}

fn publication_date(meta: Node<'_, '_>) -> (Option<Date>, Option<i16>) {
    let rank = |date: &Node<'_, '_>| -> u8 {
        let pub_type = date.attribute("pub-type");
        let date_type = date.attribute("date-type");
        let format = date.attribute("publication-format");
        match (pub_type, date_type, format) {
            (Some("epub"), _, _) | (_, Some("pub"), Some("electronic")) => 0,
            (Some("ppub"), _, _) | (_, Some("pub"), Some("print")) => 1,
            (_, Some("pub"), _) => 2,
            (Some("collection"), _, _) | (_, Some("collection"), _) => 3,
            _ => 4,
        }
    };
    let Some(best) = children(meta, "pub-date").min_by_key(rank) else {
        return (None, None);
    };
    let part = |tag: &str| child(best, tag).map(text);
    let year = part("year").and_then(|year| year.parse::<i16>().ok());
    let month = part("month").and_then(|month| month.parse::<i8>().ok());
    let day = part("day").and_then(|day| day.parse::<i8>().ok());
    let date = match (year, month, day) {
        (Some(year), Some(month), Some(day)) => Date::new(year, month, day).ok(),
        _ => None,
    };
    (date, year)
}

fn authors(meta: Node<'_, '_>) -> Vec<Author> {
    let mut authors = Vec::new();
    for group in children(meta, "contrib-group") {
        let group_is_authors = group
            .attribute("content-type")
            .is_none_or(|kind| kind == "author" || kind == "authors");
        for contrib in children(group, "contrib") {
            let is_author = contrib
                .attribute("contrib-type")
                .map_or(group_is_authors, |kind| kind == "author");
            if !is_author {
                continue;
            }
            let Some(name) = person_name(contrib) else {
                continue;
            };
            let orcid = children(contrib, "contrib-id")
                .find(|id| id.attribute("contrib-id-type") == Some("orcid"))
                .and_then(|id| text(id).parse().ok());
            authors.push(Author { name, orcid });
        }
    }
    authors
}

fn person_name(contrib: Node<'_, '_>) -> Option<String> {
    let name = child(contrib, "name").or_else(|| {
        child(contrib, "name-alternatives").and_then(|alternatives| child(alternatives, "name"))
    });
    if let Some(name) = name {
        let parts: Vec<String> = ["given-names", "surname", "suffix"]
            .iter()
            .filter_map(|tag| child(name, tag).map(text).and_then(non_empty))
            .collect();
        return non_empty(parts.join(" "));
    }
    child(contrib, "collab")
        .or_else(|| child(contrib, "string-name"))
        .map(text)
        .and_then(non_empty)
}

fn read_permissions(meta: Node<'_, '_>, work: &mut WorkMetadata) {
    let Some(permissions) = child(meta, "permissions") else {
        return;
    };
    work.copyright = child(permissions, "copyright-statement")
        .map(text)
        .and_then(non_empty);
    let Some(license) = child(permissions, "license") else {
        return;
    };
    let statements: Vec<String> = children(license, "license-p")
        .map(text)
        .filter(|statement| !statement.is_empty())
        .collect();
    work.license_statement = non_empty(statements.join(" "));
    let references = license
        .descendants()
        .filter(|node| is(*node, "license_ref"))
        .map(text);
    let own_link = href(license).map(str::to_owned);
    let links = license
        .descendants()
        .filter(|node| is(*node, "ext-link") || is(*node, "uri"))
        .filter_map(|node| href(node).map(str::to_owned).or_else(|| Some(text(node))))
        .filter(|link| link.contains("creativecommons.org"));
    work.license = references
        .chain(own_link)
        .chain(links)
        .filter_map(|candidate| candidate.parse::<License>().ok())
        .find(|license| !matches!(license, License::Unknown(_)));
}

pub(crate) fn related(article: Node<'_, '_>) -> Vec<RelatedArticle> {
    article
        .descendants()
        .filter(|node| node.is_element() && name(*node) == "related-article")
        .map(|node| {
            let target = href(node).map(str::to_owned);
            let link_type = node.attribute("ext-link-type").unwrap_or_default();
            let doi: Option<Doi> = target.as_deref().and_then(|t| t.parse().ok());
            let pmid: Option<Pmid> = match link_type {
                "pubmed" | "pmid" => target.as_deref().and_then(|t| t.parse().ok()),
                _ => None,
            };
            RelatedArticle {
                relation: node
                    .attribute("related-article-type")
                    .unwrap_or_default()
                    .to_owned(),
                doi: if pmid.is_some() { None } else { doi },
                pmid,
                href: target,
                citation: text(node),
            }
        })
        .collect()
}
