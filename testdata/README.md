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

## `sources/`

Responses of the services that arity reads, as they were on 2026-11-14. The
tests use them so that no test needs the network.

- `europepmc-search.json`: a Europe PMC search for two works: PMC10538757,
  and PubMed 9500320, a retracted article with retraction and concern links.
  We removed the fields that arity does not read, the abstracts, all authors
  after the third, and all "Comment in" links after the first.
- `crossref-work.json`: the Crossref record of
  `10.1016/S0140-6736(97)11096-0`, with only the fields that arity reads.
  Crossref metadata is open under CC0.
- `pmc-record.json` and `pmc-list.xml`: the record of PMC10538757 in the PMC
  open-access store of NCBI, and the listing of its versions.
