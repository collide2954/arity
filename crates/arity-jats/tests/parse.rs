#![cfg(test)]

use arity_core::{CcElements, Digest, License};
use arity_jats::{JatsError, Version, parse, parse_as};
use arity_text::BlockKind;

const SYNTHETIC: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE article PUBLIC "-//NLM//DTD JATS (Z39.96) Journal Archiving and Interchange DTD v1.3 20210610//EN" "JATS-archivearticle1-3.dtd">
<article xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:ali="http://www.niso.org/schemas/ali/1.0/" article-type="research-article">
<front>
<journal-meta><journal-title-group><journal-title>Journal of Tests</journal-title></journal-title-group><publisher><publisher-name>Test Press</publisher-name></publisher></journal-meta>
<article-meta>
<article-id pub-id-type="pmcid">PMC123</article-id>
<article-id pub-id-type="pmid">456</article-id>
<article-id pub-id-type="doi">10.1000/TEST.1</article-id>
<title-group><article-title>Effects of <italic>A</italic> on B</article-title></title-group>
<contrib-group>
<contrib contrib-type="author"><contrib-id contrib-id-type="orcid">https://orcid.org/0000-0002-1825-0097</contrib-id><name><surname>Carberry</surname><given-names>Josiah</given-names></name></contrib>
<contrib contrib-type="author"><collab>The Test Consortium</collab></contrib>
<contrib contrib-type="editor"><name><surname>Editor</surname><given-names>Ed</given-names></name></contrib>
</contrib-group>
<pub-date pub-type="collection"><year>2020</year></pub-date>
<pub-date pub-type="epub"><day>04</day><month>03</month><year>2021</year></pub-date>
<permissions><copyright-statement>© 2021 The authors</copyright-statement><license><ali:license_ref>https://creativecommons.org/licenses/by/4.0/</ali:license_ref><license-p>This is an open access article.</license-p></license></permissions>
<abstract><sec><title>Background</title><p>We tested A.</p></sec><sec><title>Results</title><p>No effect of A on B was found (p = 0.34).</p></sec></abstract>
<kwd-group><kwd>A</kwd><kwd>B</kwd></kwd-group>
<related-article related-article-type="corrected-article" ext-link-type="doi" xlink:href="10.1000/test.0">Original article</related-article>
</article-meta>
</front>
<body>
<p>Opening paragraph.</p>
<sec id="s1"><title>Methods</title>
<p>Cells were grown at 37 °C.</p>
<sec><title>Assay</title><p>The IC<sub>50</sub> was measured.<fig id="F1"><label>Figure 1</label><caption><title>Dose response.</title><p>Viability of cells.</p></caption><graphic xlink:href="f1.jpg"/></fig></p>
<list><list-item><p>Step one.</p></list-item><list-item><p>Step two.</p></list-item></list>
<disp-formula id="E1"><mml:math xmlns:mml="http://www.w3.org/1998/Math/MathML"><mml:mi>y</mml:mi><mml:mo>=</mml:mo><mml:mn>2</mml:mn></mml:math></disp-formula>
</sec>
</sec>
<sec><title>Results</title>
<table-wrap id="T1"><label>Table 1</label><caption><p>IC<sub>50</sub> values.</p></caption>
<table><thead><tr><th>Compound</th><th>IC<sub>50</sub> (μM)<sup>a</sup></th></tr></thead>
<tbody><tr><td>Doxorubicin</td><td>0.24 ± 0.02</td></tr></tbody></table>
<table-wrap-foot><fn><p><sup>a</sup> Mean ± SD, n = 3.</p></fn></table-wrap-foot>
</table-wrap>
<supplementary-material id="S1"><label>Table S1</label><caption><p>Raw data.</p></caption><media xlink:href="data.xlsx" mimetype="application" mime-subtype="vnd.openxmlformats-officedocument.spreadsheetml.sheet"/></supplementary-material>
<supplementary-material id="S1b"><media xlink:href="data.xlsx" mimetype="application" mime-subtype="vnd.openxmlformats-officedocument.spreadsheetml.sheet"/></supplementary-material>
</sec>
</body>
<back>
<ack><p>We thank the reviewers.</p></ack>
<ref-list><ref><mixed-citation>Reference one.</mixed-citation></ref></ref-list>
<fn-group><fn><p>Footnote text.</p></fn></fn-group>
</back>
</article>"#;

#[test]
fn reads_the_bibliographic_record() {
    let article = parse(SYNTHETIC.as_bytes()).unwrap();
    let work = &article.metadata;
    assert_eq!(work.title, "Effects of A on B");
    assert_eq!(work.pmcid.as_ref().unwrap().as_str(), "PMC123");
    assert_eq!(work.pmid.as_ref().unwrap().as_str(), "456");
    assert_eq!(work.doi.as_ref().unwrap().as_str(), "10.1000/test.1");
    assert_eq!(work.article_type.as_deref(), Some("research-article"));
    assert!(!work.is_preprint);
    assert_eq!(work.journal.as_deref(), Some("Journal of Tests"));
    assert_eq!(work.publisher.as_deref(), Some("Test Press"));
    assert_eq!(work.published, Some(jiff::civil::date(2021, 3, 4)));
    assert_eq!(work.year, Some(2021));
    let names: Vec<&str> = work.authors.iter().map(|a| a.name.as_str()).collect();
    assert_eq!(names, ["Josiah Carberry", "The Test Consortium"]);
    assert_eq!(
        work.authors[0].orcid.as_ref().unwrap().as_str(),
        "0000-0002-1825-0097"
    );
    assert_eq!(
        work.license,
        Some(License::CreativeCommons {
            elements: CcElements::By,
            version: Some("4.0".to_owned())
        })
    );
    assert_eq!(
        work.license_statement.as_deref(),
        Some("This is an open access article.")
    );
    assert_eq!(work.copyright.as_deref(), Some("© 2021 The authors"));
    assert_eq!(work.keywords, ["A", "B"]);
}

#[test]
fn reads_links_to_related_articles() {
    let article = parse(SYNTHETIC.as_bytes()).unwrap();
    assert_eq!(article.related.len(), 1);
    let related = &article.related[0];
    assert_eq!(related.relation, "corrected-article");
    assert_eq!(related.doi.as_ref().unwrap().as_str(), "10.1000/test.0");
    assert_eq!(related.citation, "Original article");
}

#[test]
fn reads_the_text_as_keyed_blocks() {
    let article = parse(SYNTHETIC.as_bytes()).unwrap();
    let blocks: Vec<(&str, BlockKind, &str)> = article
        .document
        .blocks
        .iter()
        .map(|block| (block.key.as_str(), block.kind, block.text.as_str()))
        .collect();
    assert_eq!(
        blocks,
        [
            ("title", BlockKind::Title, "Effects of A on B"),
            ("abstract.s1/title", BlockKind::Heading, "Background"),
            ("abstract.s1/p1", BlockKind::Abstract, "We tested A."),
            ("abstract.s2/title", BlockKind::Heading, "Results"),
            (
                "abstract.s2/p1",
                BlockKind::Abstract,
                "No effect of A on B was found (p = 0.34)."
            ),
            ("body/p1", BlockKind::Paragraph, "Opening paragraph."),
            ("s1/title", BlockKind::Heading, "Methods"),
            ("s1/p1", BlockKind::Paragraph, "Cells were grown at 37 °C."),
            ("s1.1/title", BlockKind::Heading, "Assay"),
            ("s1.1/p1", BlockKind::Paragraph, "The IC₅₀ was measured."),
            ("s1.1/li1", BlockKind::ListItem, "Step one."),
            ("s1.1/li2", BlockKind::ListItem, "Step two."),
            ("s1.1/eq1", BlockKind::Formula, "y=2"),
            ("s2/title", BlockKind::Heading, "Results"),
            ("ack/p1", BlockKind::Paragraph, "We thank the reviewers."),
            ("fn1", BlockKind::Footnote, "Footnote text."),
            ("t1/caption", BlockKind::TableCaption, "IC₅₀ values."),
            ("t1/r1c1", BlockKind::TableCell, "Compound"),
            ("t1/r1c2", BlockKind::TableCell, "IC₅₀ (μM)^{a}"),
            ("t1/r2c1", BlockKind::TableCell, "Doxorubicin"),
            ("t1/r2c2", BlockKind::TableCell, "0.24 ± 0.02"),
            (
                "t1/foot1",
                BlockKind::TableFootnote,
                "^{a} Mean ± SD, n = 3."
            ),
            (
                "f1/caption",
                BlockKind::FigureCaption,
                "Dose response. Viability of cells."
            ),
            ("supp1/caption", BlockKind::SupplementCaption, "Raw data."),
        ]
    );
}

#[test]
fn records_sections_labels_and_cells() {
    let document = parse(SYNTHETIC.as_bytes()).unwrap().document;
    assert_eq!(
        document.block("s1.1/p1").unwrap().section,
        ["Methods", "Assay"]
    );
    assert_eq!(
        document.block("abstract.s2/p1").unwrap().section,
        ["Abstract", "Results"]
    );
    assert_eq!(
        document.block("t1/caption").unwrap().label.as_deref(),
        Some("Table 1")
    );
    assert_eq!(
        document.block("f1/caption").unwrap().label.as_deref(),
        Some("Figure 1")
    );
    let header = document.block("t1/r1c2").unwrap().cell.clone().unwrap();
    assert!(header.header);
    assert_eq!(
        (header.table.as_str(), header.row, header.column),
        ("t1", 1, 2)
    );
    assert!(
        !document
            .block("t1/r2c2")
            .unwrap()
            .cell
            .clone()
            .unwrap()
            .header
    );
}

#[test]
fn lists_each_supplementary_file_once() {
    let article = parse(SYNTHETIC.as_bytes()).unwrap();
    assert_eq!(article.supplements.len(), 1);
    let supplement = &article.supplements[0];
    assert_eq!(supplement.key, "supp1");
    assert_eq!(supplement.href, "data.xlsx");
    assert_eq!(supplement.label.as_deref(), Some("Table S1"));
    assert_eq!(
        supplement.media_type.as_deref(),
        Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
    );
}

#[test]
fn records_the_source_digest_and_the_parser() {
    let article = parse(SYNTHETIC.as_bytes()).unwrap();
    assert_eq!(article.document.source, Digest::of(SYNTHETIC.as_bytes()));
    assert_eq!(article.document.parser, Version::LATEST.name());
}

#[test]
fn finds_each_version_by_its_name() {
    assert_eq!(Version::from_name("arity-jats 0.1.0"), Some(Version::V1));
    assert_eq!(Version::from_name("arity-jats 0.2.0"), Some(Version::V2));
    assert_eq!(Version::from_name("arity-jats 0.3.0"), Some(Version::V3));
    assert_eq!(
        Version::from_name(Version::LATEST.name()),
        Some(Version::LATEST)
    );
    assert_eq!(Version::from_name("arity-jats 9.9.9"), None);
}

#[test]
fn version_1_keeps_the_texts_that_it_made() {
    assert_eq!(
        digest(SYNTHETIC.as_bytes(), Version::V1),
        "fd7d57ab07d76a0fc10ae2ae67e729880bdb9dcf81712a1bc1109a3b4b3240fe"
    );
    assert_eq!(
        digest(REAL, Version::V1),
        "19a1880236721d7169ac5058af326f7f048af76c2b5a7986d45f17557d52abd5"
    );
}

#[test]
fn version_2_keeps_the_texts_that_it_made() {
    assert_eq!(
        digest(SYNTHETIC.as_bytes(), Version::V2),
        "6abaee6776da113ef1e84d5c6ba29046db4ed2ab1a577f2ef3e7d356fec04751"
    );
    assert_eq!(
        digest(REAL, Version::V2),
        "1b3765109cfe9a4399d004eb11c0bdbec7faab228dc27800041bf52b64977e54"
    );
}

#[test]
fn version_3_keeps_the_texts_that_it_made() {
    assert_eq!(
        digest(SYNTHETIC.as_bytes(), Version::V3),
        "c3ad9cd2afa744da4c208e5e49896582106e3c9dda2bf004f8c5818c4c900ad4"
    );
    assert_eq!(
        digest(REAL, Version::V3),
        "94721ff47dc82c9b08d301325ec3a59f9f60af04477538e8c5dd7327f665bdf5"
    );
}

#[test]
fn version_3_writes_the_blocks_of_version_2_without_variant_letters() {
    for bytes in [SYNTHETIC.as_bytes(), REAL] {
        assert_eq!(
            parse_as(bytes, Version::V3).unwrap().document.blocks,
            parse_as(bytes, Version::V2).unwrap().document.blocks
        );
    }
}

const REAL: &[u8] = include_bytes!("../../../testdata/jats/PMC6274071.xml");

fn digest(bytes: &[u8], version: Version) -> String {
    parse_as(bytes, version)
        .unwrap()
        .document
        .digest()
        .unwrap()
        .as_str()
        .to_owned()
}

#[test]
fn rejects_files_that_are_not_articles() {
    assert!(matches!(parse(b"<html/>"), Err(JatsError::NotAnArticle(name)) if name == "html"));
    assert!(matches!(parse(b"<article>"), Err(JatsError::Xml(_))));
    assert!(matches!(parse(&[0xff, 0xfe]), Err(JatsError::Encoding)));
}

#[test]
fn reads_a_real_article_with_tables() {
    let bytes = include_bytes!("../../../testdata/jats/PMC6274071.xml");
    let article = parse(bytes).unwrap();
    let work = &article.metadata;
    assert_eq!(
        work.title,
        "Synthesis and Cytotoxic Activity of Biphenylurea Derivatives Containing Indolin-2-one Moieties"
    );
    assert_eq!(
        work.doi.as_ref().unwrap().as_str(),
        "10.3390/molecules21060762"
    );
    assert_eq!(work.authors[0].name, "Wagdy M Eldehna");
    assert_eq!(
        work.license,
        Some(License::CreativeCommons {
            elements: CcElements::By,
            version: Some("4.0".to_owned())
        })
    );
    assert_eq!(
        article.document.source.as_str(),
        "c8d401499f07a287734f10614ae425d60c3d7d4feec22ad3b02559e9e7b3e3ac"
    );
    let document = &article.document;
    assert_eq!(
        document.block("t1/caption").unwrap().label.as_deref(),
        Some("Table 1")
    );
    assert_eq!(document.block("t1/r1c5").unwrap().text, "IC₅₀ (μM) ^{a}");
    assert_eq!(document.block("t1/r2c5").unwrap().text, "MCF-7");
    assert_eq!(document.block("t1/r3c5").unwrap().text, "19.53 ± 1.05");
    assert_eq!(article.supplements.len(), 1);
    assert_eq!(
        article.supplements[0].media_type.as_deref(),
        Some("application/pdf")
    );
}
