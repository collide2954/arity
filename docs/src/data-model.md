# Data model

The database is one SQLite file. This page tells what it holds.

## Works

A work is one article. arity finds it by its PMCID, DOI, or PMID. The record of
a work holds its metadata: the title, the authors, the journal, the year, and
the license. A preprint has a label.

Each work has an entry in the reading queue. The state of the entry is
`queued`, `fetched`, `read`, `failed`, or `skipped`. The entry also holds the
query that found the work.

## Files and texts

A retrieval is one downloaded file: the JATS XML, the PDF, or a supplementary
file. arity records its address, its SHA-256 digest, its size, its media type,
the software, and the time. The file store keeps each file under its digest.

The canonical text is the text of a work in blocks. arity makes it from the
JATS XML and from the supplementary data files. See
[Canonical text](text.md).

## Entities

An entity is a thing that a report is about, for example a chemical, a cell
line, a gene, a material, or a procedure. See [Entity kinds](reference/kinds.md).

An entity can have registry identifiers, for example `CHEBI:28748` for
doxorubicin or `CELLOSAURUS:CVCL_0031` for MCF-7. arity uses these identifiers
to link the same entity across works. An entity without an identifier stays
with its own submission. See [Registries](reference/registries.md).

## Properties

A property is what a value measures, for example `ic50` or `band_gap`. It has a
value type: `quantity`, `boolean`, or `category`. A quantity property has a
UCUM unit, and that unit sets its dimension. arity refuses a value with a unit
of another dimension. For the tables and the figures, arity converts each
value into the unit of its property. See [Units](units.md).

## Reports

A report is one statement of a source. There are two types of report.

A value report states the value of a property for one or more subjects. For
example: the IC50 of doxorubicin in MCF-7 cells after 72 h is 7.30 ± 0.84 µM.
The value can be a number, an interval, a bound such as "> 100", a boolean, or
a category. It can have an uncertainty, a statistic, and a number of samples.
A value that the reader read from an image has a label, with the caption of
the image, the method of the reading, and its resolution. The image is a plot,
or a table that the source gives only as an image.

An effect report states that agents change a target, or that they associate
with it. For example: compound 4d had no effect on MCF-7 cells. The direction
is `increase` (+1), `decrease` (−1), `no_change` (0), or `mixed`. An effect can
have a magnitude, for example a 20-fold decrease, a comparator, and a
significance.

## Conditions

A condition tells the circumstances of a report. It is an entity in a role (for
example the cell line), a text, or the value of a property (for example an
exposure time of 72 h). Two values of a property with different conditions
stay apart in the tables.

## Evidence

Each report and each entity has quotes from its source. arity records the
block, the quote, and the position of the quote in the block.

## Runs and checks

A run is one stored submission. It records the submission file byte for byte,
its digest, the reader, the responsible person, the software that checked it,
and the warnings of the checks.

A check records that a person compared a report with its source and confirmed
it or rejected it. It has the person, the time, and an optional note. The
state of a report is the state of its last check, or `unchecked`.

## Notices

A notice is a retraction, a correction, or another statement about a work. See
[Notices](notices.md).
