use std::fmt;
use std::str::FromStr;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The kind of a thing that a report is about.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    Chemical,
    Material,
    Gene,
    Protein,
    NucleicAcid,
    Organism,
    CellLine,
    CellType,
    AnatomicalEntity,
    Disease,
    Phenotype,
    Process,
    PhysicalSystem,
    Device,
    Environment,
    Population,
    Procedure,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0:?} is not a known entity kind")]
pub struct UnknownEntityKind(pub String);

impl EntityKind {
    pub const ALL: [Self; 18] = [
        Self::Chemical,
        Self::Material,
        Self::Gene,
        Self::Protein,
        Self::NucleicAcid,
        Self::Organism,
        Self::CellLine,
        Self::CellType,
        Self::AnatomicalEntity,
        Self::Disease,
        Self::Phenotype,
        Self::Process,
        Self::PhysicalSystem,
        Self::Device,
        Self::Environment,
        Self::Population,
        Self::Procedure,
        Self::Other,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Chemical => "chemical",
            Self::Material => "material",
            Self::Gene => "gene",
            Self::Protein => "protein",
            Self::NucleicAcid => "nucleic_acid",
            Self::Organism => "organism",
            Self::CellLine => "cell_line",
            Self::CellType => "cell_type",
            Self::AnatomicalEntity => "anatomical_entity",
            Self::Disease => "disease",
            Self::Phenotype => "phenotype",
            Self::Process => "process",
            Self::PhysicalSystem => "physical_system",
            Self::Device => "device",
            Self::Environment => "environment",
            Self::Population => "population",
            Self::Procedure => "procedure",
            Self::Other => "other",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Chemical => "An element, molecule, ion, drug, mixture, or solution.",
            Self::Material => {
                "A solid material or a material sample, for example an alloy, a polymer, or a thin film."
            }
            Self::Gene => "A gene or a genetic locus.",
            Self::Protein => "A protein, peptide, enzyme, or antibody.",
            Self::NucleicAcid => {
                "An RNA or DNA molecule that is not a gene, for example an mRNA, a microRNA, or a plasmid."
            }
            Self::Organism => "A taxon, species, strain, or cultivar.",
            Self::CellLine => "A named cell line, for example HeLa.",
            Self::CellType => "A type of cell, for example a hepatocyte.",
            Self::AnatomicalEntity => "An organ, tissue, body part, or cell component.",
            Self::Disease => "A disease or disorder.",
            Self::Phenotype => "An observable trait, sign, or symptom.",
            Self::Process => {
                "A process or phenomenon, for example a pathway, a reaction, or a phase transition."
            }
            Self::PhysicalSystem => {
                "A physical system, for example a particle, a nucleus, a plasma, or a star."
            }
            Self::Device => "An instrument, apparatus, sensor, or implant.",
            Self::Environment => {
                "An environmental material or setting, for example soil or seawater."
            }
            Self::Population => "A group of people or animals, for example a cohort.",
            Self::Procedure => {
                "An intervention or procedure, for example surgery, a diet, or irradiation."
            }
            Self::Other => "A thing that no other kind describes.",
        }
    }
}

impl fmt::Display for EntityKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for EntityKind {
    type Err = UnknownEntityKind;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.as_str() == text)
            .ok_or_else(|| UnknownEntityKind(text.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip() {
        for kind in EntityKind::ALL {
            assert_eq!(kind.as_str().parse::<EntityKind>(), Ok(kind));
            assert_eq!(
                serde_json::to_string(&kind).unwrap(),
                format!("\"{}\"", kind.as_str())
            );
        }
    }

    #[test]
    fn rejects_unknown_names() {
        assert_eq!(
            "Chemical".parse::<EntityKind>(),
            Err(UnknownEntityKind("Chemical".to_owned()))
        );
    }

    #[test]
    fn each_kind_has_a_description_that_ends_with_a_period() {
        for kind in EntityKind::ALL {
            assert!(kind.description().ends_with('.'), "{kind}");
        }
    }
}
