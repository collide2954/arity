use std::fmt;
use std::str::FromStr;
use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::EntityKind;
use crate::EntityKind::{
    AnatomicalEntity, CellLine, CellType, Chemical, Disease, Environment, Gene, Material,
    NucleicAcid, Organism, Phenotype, Process, Protein,
};

/// Where a prefix can identify things.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// The prefix identifies entities of these kinds only.
    Entities(&'static [EntityKind]),
    /// The prefix can identify an entity of any kind.
    AnyEntity,
    /// The prefix identifies methods, properties, or units, not entities.
    Terms,
}

/// A known identifier namespace.
#[derive(Debug)]
pub struct Prefix {
    pub name: &'static str,
    pub title: &'static str,
    pub scope: Scope,
    pattern: &'static str,
}

const fn prefix(
    name: &'static str,
    title: &'static str,
    pattern: &'static str,
    scope: Scope,
) -> Prefix {
    Prefix {
        name,
        title,
        scope,
        pattern,
    }
}

const UNIPROT: &str =
    r"(?:[A-NR-Z][0-9](?:[A-Z][A-Z0-9]{2}[0-9]){1,2}|[OPQ][0-9][A-Z0-9]{3}[0-9])(?:-\d+)?";

pub static PREFIXES: &[Prefix] = &[
    prefix(
        "CHEBI",
        "Chemical Entities of Biological Interest",
        r"\d+",
        Scope::Entities(&[Chemical]),
    ),
    prefix(
        "PUBCHEM.COMPOUND",
        "PubChem Compound",
        r"\d+",
        Scope::Entities(&[Chemical]),
    ),
    prefix(
        "INCHIKEY",
        "InChIKey",
        r"[A-Z]{14}-[A-Z]{10}-[A-Z]",
        Scope::Entities(&[Chemical]),
    ),
    prefix(
        "CAS",
        "CAS Registry Number",
        r"\d{2,7}-\d{2}-\d",
        Scope::Entities(&[Chemical, Material]),
    ),
    prefix(
        "CHEMBL.COMPOUND",
        "ChEMBL compound",
        r"CHEMBL\d+",
        Scope::Entities(&[Chemical]),
    ),
    prefix(
        "DRUGBANK",
        "DrugBank",
        r"DB\d{5}",
        Scope::Entities(&[Chemical]),
    ),
    prefix(
        "UNII",
        "Unique Ingredient Identifier",
        r"[0-9A-Z]{10}",
        Scope::Entities(&[Chemical, Material]),
    ),
    prefix(
        "NCBITaxon",
        "NCBI Taxonomy",
        r"\d+",
        Scope::Entities(&[Organism]),
    ),
    prefix("NCBIGene", "NCBI Gene", r"\d+", Scope::Entities(&[Gene])),
    prefix(
        "HGNC",
        "HUGO Gene Nomenclature Committee",
        r"\d+",
        Scope::Entities(&[Gene]),
    ),
    prefix(
        "MGI",
        "Mouse Genome Informatics",
        r"\d+",
        Scope::Entities(&[Gene]),
    ),
    prefix(
        "RGD",
        "Rat Genome Database",
        r"\d+",
        Scope::Entities(&[Gene]),
    ),
    prefix(
        "FB",
        "FlyBase",
        r"FB[a-z]{2}\d{7}",
        Scope::Entities(&[Gene]),
    ),
    prefix("WB", "WormBase", r"WBGene\d{8}", Scope::Entities(&[Gene])),
    prefix(
        "ZFIN",
        "Zebrafish Information Network",
        r"ZDB-[A-Z]+-\d{6}-\d+",
        Scope::Entities(&[Gene]),
    ),
    prefix(
        "SGD",
        "Saccharomyces Genome Database",
        r"S\d{9}",
        Scope::Entities(&[Gene]),
    ),
    prefix(
        "ENSEMBL",
        "Ensembl",
        r"ENS[A-Z]*[EFGPRT]\d{11}(?:\.\d+)?",
        Scope::Entities(&[Gene, NucleicAcid, Protein]),
    ),
    prefix(
        "UniProtKB",
        "UniProt Knowledgebase",
        UNIPROT,
        Scope::Entities(&[Protein]),
    ),
    prefix(
        "PR",
        "Protein Ontology",
        r"(?:\d{9}|[A-Z][0-9][A-Z0-9]{3}[0-9](?:-\d+)?)",
        Scope::Entities(&[Protein]),
    ),
    prefix(
        "PDB",
        "Protein Data Bank",
        r"[0-9][A-Za-z0-9]{3}",
        Scope::Entities(&[Protein, NucleicAcid, Chemical]),
    ),
    prefix(
        "MIRBASE",
        "miRBase",
        r"MI(?:MAT)?\d{7}",
        Scope::Entities(&[NucleicAcid]),
    ),
    prefix(
        "RNACENTRAL",
        "RNAcentral",
        r"URS[0-9A-F]{10}(?:_\d+)?",
        Scope::Entities(&[NucleicAcid]),
    ),
    prefix(
        "SO",
        "Sequence Ontology",
        r"\d{7}",
        Scope::Entities(&[NucleicAcid]),
    ),
    prefix(
        "CL",
        "Cell Ontology",
        r"\d{7}",
        Scope::Entities(&[CellType]),
    ),
    prefix(
        "CLO",
        "Cell Line Ontology",
        r"\d{7}",
        Scope::Entities(&[CellLine]),
    ),
    prefix(
        "CELLOSAURUS",
        "Cellosaurus",
        r"CVCL_[A-Z0-9]{4}",
        Scope::Entities(&[CellLine]),
    ),
    prefix(
        "UBERON",
        "Uberon anatomy ontology",
        r"\d{7}",
        Scope::Entities(&[AnatomicalEntity]),
    ),
    prefix(
        "FMA",
        "Foundational Model of Anatomy",
        r"\d+",
        Scope::Entities(&[AnatomicalEntity]),
    ),
    prefix(
        "MONDO",
        "Mondo Disease Ontology",
        r"\d{7}",
        Scope::Entities(&[Disease]),
    ),
    prefix(
        "DOID",
        "Human Disease Ontology",
        r"\d+",
        Scope::Entities(&[Disease]),
    ),
    prefix(
        "HP",
        "Human Phenotype Ontology",
        r"\d{7}",
        Scope::Entities(&[Phenotype]),
    ),
    prefix(
        "MP",
        "Mammalian Phenotype Ontology",
        r"\d{7}",
        Scope::Entities(&[Phenotype]),
    ),
    prefix(
        "GO",
        "Gene Ontology",
        r"\d{7}",
        Scope::Entities(&[Process, AnatomicalEntity]),
    ),
    prefix("RHEA", "Rhea", r"\d{5,6}", Scope::Entities(&[Process])),
    prefix(
        "REACTOME",
        "Reactome",
        r"R-[A-Z]{3}-\d+(?:-\d+)?(?:\.\d+)?",
        Scope::Entities(&[Process]),
    ),
    prefix(
        "ENVO",
        "Environment Ontology",
        r"\d{7,8}",
        Scope::Entities(&[Environment, Material]),
    ),
    prefix(
        "MESH",
        "Medical Subject Headings",
        r"[A-Z]\d{6,9}",
        Scope::AnyEntity,
    ),
    prefix("NCIT", "NCI Thesaurus", r"[CRP]\d+", Scope::AnyEntity),
    prefix(
        "EFO",
        "Experimental Factor Ontology",
        r"\d{7}",
        Scope::AnyEntity,
    ),
    prefix("WIKIDATA", "Wikidata", r"[QP]\d+", Scope::AnyEntity),
    prefix(
        "RRID",
        "Research Resource Identifier",
        r"[A-Z]+_[A-Za-z0-9_:\-]+",
        Scope::AnyEntity,
    ),
    prefix(
        "OBI",
        "Ontology for Biomedical Investigations",
        r"\d{7}",
        Scope::Terms,
    ),
    prefix("CHMO", "Chemical Methods Ontology", r"\d{7}", Scope::Terms),
    prefix("BAO", "BioAssay Ontology", r"\d{7}", Scope::Terms),
    prefix(
        "PATO",
        "Phenotype And Trait Ontology",
        r"\d{7}",
        Scope::Terms,
    ),
    prefix(
        "OBA",
        "Ontology of Biological Attributes",
        r"(?:VT)?\d{7}",
        Scope::Terms,
    ),
    prefix(
        "ECO",
        "Evidence and Conclusion Ontology",
        r"\d{7}",
        Scope::Terms,
    ),
    prefix(
        "UO",
        "Units of Measurement Ontology",
        r"\d{7}",
        Scope::Terms,
    ),
];

static PATTERNS: LazyLock<Vec<Option<Regex>>> = LazyLock::new(|| {
    PREFIXES
        .iter()
        .map(|prefix| Regex::new(&format!("^(?:{})$", prefix.pattern)).ok())
        .collect()
});

impl Prefix {
    pub fn lookup(name: &str) -> Option<&'static Self> {
        PREFIXES
            .iter()
            .find(|prefix| prefix.name.eq_ignore_ascii_case(name))
    }

    fn accepts(&'static self, local: &str) -> bool {
        PREFIXES
            .iter()
            .position(|candidate| std::ptr::eq(candidate, self))
            .and_then(|index| PATTERNS.get(index))
            .and_then(Option::as_ref)
            .is_some_and(|pattern| pattern.is_match(local))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CurieError {
    #[error("{0:?} has no prefix; use the form PREFIX:ID")]
    MissingPrefix(String),
    #[error("{0:?} is not a known prefix")]
    UnknownPrefix(String),
    #[error("{local:?} is not a valid {prefix} identifier")]
    InvalidLocal { prefix: &'static str, local: String },
}

/// A compact identifier, for example `CHEBI:15377`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Curie {
    prefix: &'static str,
    local: String,
}

impl Curie {
    pub fn prefix(&self) -> &'static Prefix {
        Prefix::lookup(self.prefix).unwrap_or(&PREFIXES[0])
    }

    pub fn prefix_name(&self) -> &'static str {
        self.prefix
    }

    pub fn local(&self) -> &str {
        &self.local
    }

    pub fn url(&self) -> String {
        format!("https://bioregistry.io/{self}")
    }

    pub fn fits_entity_kind(&self, kind: EntityKind) -> bool {
        match self.prefix().scope {
            Scope::Entities(kinds) => kinds.contains(&kind),
            Scope::AnyEntity => true,
            Scope::Terms => false,
        }
    }
}

impl FromStr for Curie {
    type Err = CurieError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let trimmed = text.trim();
        let (name, local) = trimmed
            .split_once(':')
            .filter(|(name, _)| !name.is_empty())
            .ok_or_else(|| CurieError::MissingPrefix(text.to_owned()))?;
        let prefix =
            Prefix::lookup(name).ok_or_else(|| CurieError::UnknownPrefix(name.to_owned()))?;
        if !prefix.accepts(local) {
            return Err(CurieError::InvalidLocal {
                prefix: prefix.name,
                local: local.to_owned(),
            });
        }
        Ok(Self {
            prefix: prefix.name,
            local: local.to_owned(),
        })
    }
}

impl fmt::Display for Curie {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.prefix, self.local)
    }
}

impl TryFrom<String> for Curie {
    type Error = CurieError;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        text.parse()
    }
}

impl From<Curie> for String {
    fn from(curie: Curie) -> Self {
        curie.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pattern_compiles() {
        for (prefix, pattern) in PREFIXES.iter().zip(PATTERNS.iter()) {
            assert!(pattern.is_some(), "{}", prefix.name);
        }
    }

    #[test]
    fn prefix_names_are_unique_without_case() {
        for (index, prefix) in PREFIXES.iter().enumerate() {
            let first = PREFIXES
                .iter()
                .position(|other| other.name.eq_ignore_ascii_case(prefix.name));
            assert_eq!(first, Some(index), "{}", prefix.name);
        }
    }

    #[test]
    fn parses_valid_identifiers() {
        for text in [
            "CHEBI:15377",
            "PUBCHEM.COMPOUND:2244",
            "INCHIKEY:BSYNRYMUTXBXSQ-UHFFFAOYSA-N",
            "NCBITaxon:9606",
            "HGNC:11998",
            "ENSEMBL:ENSG00000141510",
            "UniProtKB:P04637",
            "UniProtKB:A0A023GPI8",
            "CL:0000236",
            "CELLOSAURUS:CVCL_0030",
            "UBERON:0002107",
            "MONDO:0005148",
            "GO:0008150",
            "MESH:D000077192",
            "WIKIDATA:Q42",
        ] {
            let curie: Curie = text.parse().unwrap_or_else(|e| panic!("{text}: {e}"));
            assert_eq!(curie.to_string(), text);
        }
    }

    #[test]
    fn normalizes_the_case_of_the_prefix() {
        let curie: Curie = "ncbitaxon:9606".parse().unwrap();
        assert_eq!(curie.to_string(), "NCBITaxon:9606");
        assert_eq!(curie.prefix_name(), "NCBITaxon");
        assert_eq!(curie.local(), "9606");
    }

    #[test]
    fn rejects_malformed_identifiers() {
        assert_eq!(
            "15377".parse::<Curie>(),
            Err(CurieError::MissingPrefix("15377".to_owned()))
        );
        assert_eq!(
            ":15377".parse::<Curie>(),
            Err(CurieError::MissingPrefix(":15377".to_owned()))
        );
        assert_eq!(
            "FOO:1".parse::<Curie>(),
            Err(CurieError::UnknownPrefix("FOO".to_owned()))
        );
        for text in [
            "CHEBI:abc",
            "CHEBI:",
            "GO:123",
            "UniProtKB:p04637",
            "CL:00002361",
        ] {
            assert!(
                matches!(text.parse::<Curie>(), Err(CurieError::InvalidLocal { .. })),
                "{text}"
            );
        }
    }

    #[test]
    fn checks_the_entity_kind() {
        let chebi: Curie = "CHEBI:15377".parse().unwrap();
        assert!(chebi.fits_entity_kind(Chemical));
        assert!(!chebi.fits_entity_kind(Organism));

        let mesh: Curie = "MESH:D000077192".parse().unwrap();
        assert!(mesh.fits_entity_kind(Disease));
        assert!(mesh.fits_entity_kind(EntityKind::Other));

        let obi: Curie = "OBI:0000070".parse().unwrap();
        assert!(!obi.fits_entity_kind(EntityKind::Procedure));
    }

    #[test]
    fn resolves_through_bioregistry() {
        let curie: Curie = "CHEBI:15377".parse().unwrap();
        assert_eq!(curie.url(), "https://bioregistry.io/CHEBI:15377");
    }

    #[test]
    fn serializes_as_a_string() {
        let curie: Curie = serde_json::from_str("\"chebi:15377\"").unwrap();
        assert_eq!(serde_json::to_string(&curie).unwrap(), "\"CHEBI:15377\"");
        assert!(serde_json::from_str::<Curie>("\"CHEBI:x\"").is_err());
    }
}
