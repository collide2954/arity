# Provenance

For each report, arity can tell where it came from: which file, which text,
which reader, which person, and which software. The records follow the pattern
of W3C PROV: entities, the activities that make them, and the agents that do
the activities.

## Entities, activities, and agents

| PROV | In arity |
| --- | --- |
| Entity | A source file, with its address, its size, and its SHA-256 digest. |
| Entity | A canonical text, with its digest and its parser. |
| Entity | A submission, byte for byte, with its digest. |
| Entity | A report, with its quotes and their positions. |
| Activity | A retrieval: arity downloads a file at a time. |
| Activity | A parse: arity makes a canonical text from a file. |
| Activity | A run: arity checks a submission and stores its reports. |
| Activity | A check: a person compares a report with its source. |
| Agent | The reader, a person or software, with its name and version. |
| Agent | The responsible person, with a name and an optional ORCID. |
| Agent | The arity software, with its version. |

The reader acts on behalf of the responsible person. In PROV terms, the reader
`actedOnBehalfOf` the responsible person.

## The chain of a report

1. A retrieval records the address of the JATS XML, its digest, and the time.
2. The canonical text records the digest of that file and the parser.
3. The submission names the digest of the canonical text. The checks refuse a
   submission that names another text.
4. Each quote of a report names a block of that text, and arity records the
   position of the quote in the block.
5. Each check names the report, the person, the time, and the state.

So each report leads back to the exact bytes that its source published.

## What arity does not record

arity does not record a confidence or a quality score for a report. A
submission cannot hold one. arity records only what a source states and who
recorded it.
