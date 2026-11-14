# Test data

This folder holds the files that the tests read.

## `text/normalization.json`

The shared test vectors for text normalization and quote search. The Rust
tests and the website tests read the same file. If you change the rules, change
this file first.

## `jats/PMC6274071.xml`

The full text of an open-access article, in JATS XML, as Europe PMC supplied it
on 2026-11-14. The tests use it as a real example of an article with tables.

- Article: Eldehna WM, Fares M, Ibrahim HS, et al. "Synthesis and Cytotoxic
  Activity of Biphenylurea Derivatives Containing Indolin-2-one Moieties."
  *Molecules* 21(6):762, 2016.
- DOI: [10.3390/molecules21060762](https://doi.org/10.3390/molecules21060762)
- Source: <https://www.ebi.ac.uk/europepmc/webservices/rest/PMC6274071/fullTextXML>
- SHA-256: `c8d401499f07a287734f10614ae425d60c3d7d4feec22ad3b02559e9e7b3e3ac`
- Copyright: © 2016 by the authors.
- License: [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/). We did
  not change the file.
