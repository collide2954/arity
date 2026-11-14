# arity

arity collects the values and the effects that open-access research articles
report. It keeps each value with the words of its source, and with a full
record of who read it, when, and how.

With arity, you can compare one property across papers, experiments, and
conditions. For example, you can see each IC50 of doxorubicin in MCF-7 cells
that a set of articles reports. Each value has the conditions of its
experiment and a link to the sentence or the table cell that states it.

arity is for the biomedical, chemical, and physical sciences. It takes any kind
of measured value, in any unit that UCUM can express.

## The parts of arity

- The `arity` command, in Rust. It finds works, downloads their files, makes a
  canonical text, checks submissions, and stores them in a SQLite database.
- The website. It shows each work with a mark on each quote in its PDF, the
  values as tables and figures, and the effects as a matrix.
- The archive, in the repository arity-data. It holds the submissions and the
  checks. arity can make the same database again from the archive.

## Who reads the articles

A reader reads a work and writes a submission. A submission is a JSON file
that states the entities, the values, and the effects of the work, each with
quotes from the source. The reader can be a person or software. In the pilot,
Claude, a language model from Anthropic, reads the works. Each submission names
the person who is responsible for it.

The `arity` command does not trust a submission. It finds each quote in the
text, each number and each unit in the quotes, and it makes sure that each
unit measures the dimension of its property. Then people compare the reports
with their sources, and confirm or reject them. The website labels each report
that no person checked yet.
