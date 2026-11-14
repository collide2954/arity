# Registries

<!-- The tests of arity-protocol write this file. After a change, run them with ARITY_UPDATE_DOCS=1. -->

An identifier of an entity, a method, or a property has the form `PREFIX:ID`, for example `CHEBI:28748`. arity accepts these prefixes. `arity vocab prefixes` prints this list.

| Prefix | Registry | Identifies |
| --- | --- | --- |
| `CHEBI` | Chemical Entities of Biological Interest | `chemical` |
| `PUBCHEM.COMPOUND` | PubChem Compound | `chemical` |
| `INCHIKEY` | InChIKey | `chemical` |
| `CAS` | CAS Registry Number | `chemical`, `material` |
| `CHEMBL.COMPOUND` | ChEMBL compound | `chemical` |
| `DRUGBANK` | DrugBank | `chemical` |
| `UNII` | Unique Ingredient Identifier | `chemical`, `material` |
| `NCBITaxon` | NCBI Taxonomy | `organism` |
| `NCBIGene` | NCBI Gene | `gene` |
| `HGNC` | HUGO Gene Nomenclature Committee | `gene` |
| `MGI` | Mouse Genome Informatics | `gene` |
| `RGD` | Rat Genome Database | `gene` |
| `FB` | FlyBase | `gene` |
| `WB` | WormBase | `gene` |
| `ZFIN` | Zebrafish Information Network | `gene` |
| `SGD` | Saccharomyces Genome Database | `gene` |
| `ENSEMBL` | Ensembl | `gene`, `nucleic_acid`, `protein` |
| `UniProtKB` | UniProt Knowledgebase | `protein` |
| `PR` | Protein Ontology | `protein` |
| `PDB` | Protein Data Bank | `protein`, `nucleic_acid`, `chemical` |
| `MIRBASE` | miRBase | `nucleic_acid` |
| `RNACENTRAL` | RNAcentral | `nucleic_acid` |
| `SO` | Sequence Ontology | `nucleic_acid` |
| `CL` | Cell Ontology | `cell_type` |
| `CLO` | Cell Line Ontology | `cell_line` |
| `CELLOSAURUS` | Cellosaurus | `cell_line` |
| `UBERON` | Uberon anatomy ontology | `anatomical_entity` |
| `FMA` | Foundational Model of Anatomy | `anatomical_entity` |
| `MONDO` | Mondo Disease Ontology | `disease` |
| `DOID` | Human Disease Ontology | `disease` |
| `HP` | Human Phenotype Ontology | `phenotype` |
| `MP` | Mammalian Phenotype Ontology | `phenotype` |
| `GO` | Gene Ontology | `process`, `anatomical_entity` |
| `RHEA` | Rhea | `process` |
| `REACTOME` | Reactome | `process` |
| `ENVO` | Environment Ontology | `environment`, `material` |
| `MESH` | Medical Subject Headings | entities of any kind |
| `NCIT` | NCI Thesaurus | entities of any kind |
| `EFO` | Experimental Factor Ontology | entities of any kind |
| `WIKIDATA` | Wikidata | entities of any kind |
| `RRID` | Research Resource Identifier | entities of any kind |
| `OBI` | Ontology for Biomedical Investigations | methods and properties |
| `CHMO` | Chemical Methods Ontology | methods and properties |
| `BAO` | BioAssay Ontology | methods and properties |
| `PATO` | Phenotype And Trait Ontology | methods and properties |
| `OBA` | Ontology of Biological Attributes | methods and properties |
| `ECO` | Evidence and Conclusion Ontology | methods and properties |
| `UO` | Units of Measurement Ontology | methods and properties |
