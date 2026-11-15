# Submissions

A submission is a JSON file in the format `arity-submission/1`. A reader writes
one submission for one reading of one work. The file
`schemas/submission.schema.json` is the full JSON Schema, and `arity schema`
prints it.

The format accepts only the fields that it defines. A field such as
`confidence` is an error: arity records what a source states, and a submission
cannot add a judgement.

## The parts of a submission

| Field | Meaning |
| --- | --- |
| `format` | Always `arity-submission/1`. |
| `work` | The PMCID, DOI, or PMID of the work. |
| `text` | The digest of the canonical text that the reader read. |
| `reader` | The reader: `kind` (`software` or `person`), `name`, `version`, and an optional `orcid`. |
| `responsible` | The person who is responsible for the submission: `name` and an optional `orcid`. |
| `note` | An optional note about the reading. |
| `replaces` | The number of an earlier run of the same work that this submission corrects. See [Corrections](#corrections). |
| `entities` | The entities that the reports are about. |
| `properties` | The properties that the reports use. |
| `reports` | The value reports and the effect reports. |
| `skipped` | The quotes with numbers that the reader does not report, each with the reason. |

## Entities

Each entity has a local `id`, a `kind`, a `label`, optional registry
identifiers in `xrefs`, and `mentions`. A mention is a quote of a place where
the source names the entity.

## Properties

A submission lists each property that its reports use. For a property that is
in the database, give only its `id`. For a new property, give its `label`, its
`value_type` (`quantity`, `boolean`, or `category`), its `unit` for a quantity,
its `categories` for a category, and a `description`.

## Value reports

A value report has `type: "value"`, a local `id`, the `subject` entities, the
`property`, the `value`, optional `conditions`, an optional `method`, an
optional `note`, and the `evidence`.

The `value` holds one of `number`, `low` with `high`, `boolean`, or `category`.
A quantity also has `unit` (UCUM) and `unit_text` (the unit as the source writes
it). Other fields of a value:

| Field | Meaning |
| --- | --- |
| `bound` | `lt`, `le`, `gt`, `ge`, or `approx`, for values such as "> 100" or "about 1.20". |
| `uncertainty` | `kind` (`sd`, `sem`, `ci`, `range`, `iqr`, or `other`), with `number` or `low` and `high`, and `level` for an interval. |
| `statistic` | `mean`, `median`, `geometric_mean`, `mode`, `single`, `minimum`, `maximum`, or `other`. |
| `n` | The number of samples. |
| `read_from_figure` | For a value that the reader read from an image: the `figure` key of the caption (of a figure, or of a table that the source gives only as an image, for example `t1/caption`), the `method`, and the `resolution`. |

## Effect reports

An effect report has `type: "effect"`, a local `id`, the `agents` (each an
entity with an optional `dose`), the `target` (an entity, with an optional
property), the `direction`, and the `evidence`. Optional fields:

| Field | Meaning |
| --- | --- |
| `relation` | `effect` (the default) or `association`. |
| `magnitude` | A `measure` such as `fold_change` or `percent_change`, a `number`, and a unit. |
| `comparator` | What the source compares with, for example a control. |
| `significance` | A `statement` (`significant` or `not_significant`), a `p` value with its `comparison`, the `test`, the test `statistic`, the `correction` for multiple tests, and a `q` value. See [Statistics of a test](#statistics-of-a-test). |
| `n` | The number of samples. |
| `groups` | The groups that the source compares, each with its `label` as the source writes it and its number of samples `n`. |
| `conditions`, `method`, `note` | As for a value report. |

### Statistics of a test

Record the statistics of a test as the source states them:

```json
"groups": [
  { "label": "treated cells", "n": 8 },
  { "label": "control cells", "n": 6 }
],
"significance": {
  "statement": "significant", "comparison": "eq", "p": "0.04",
  "statistic": { "kind": "t", "number": "2.31", "df": ["12"] },
  "correction": "bonferroni", "q": "0.08"
}
```

| Field | Meaning |
| --- | --- |
| `statistic.kind` | `t`, `f`, `chi_squared`, `u` (Mann–Whitney), `w` (Wilcoxon), `h` (Kruskal–Wallis), `z`, `r` (a tested correlation), or `other`. |
| `statistic.number` | The value of the statistic. |
| `statistic.df` | The degrees of freedom: one number for t or χ², and two numbers for F, for example `["2", "27"]`. |
| `correction` | `bonferroni`, `holm`, `sidak`, `tukey`, `dunnett`, `scheffe`, `benjamini_hochberg`, or `other`. |
| `q` | A p-value that the source corrects for the false discovery rate. |

A quote must contain each number: the statistic, each degree of freedom, the
`q` value, and the `n` of each group, and also the label of each group. A
significance needs a statement, a p-value, or a q-value. When no quote names
the correction, the checks give a warning, and a person must look.

## Conditions

A condition has a `role` with an `entity` or a `text`, or a `property` with a
`value`. The roles are `organism`, `cell_line`, `cell_type`,
`anatomical_site`, `sample`, `target`, `solvent`, `medium`, `co_treatment`,
`comparator`, `population`, `device`, `environment`, and `other`.

## Evidence

Each item of evidence is a `block` key and a `quote`. When the quote occurs more
than one time in the block, give `occurrence` (1 is the first).

## Skipped quotes

A submission can list quotes with numbers that the reader does not report, in
`skipped`. Each item has a `block`, a `quote`, an optional `occurrence`, a
`reason`, and a `note`:

| Reason | Meaning |
| --- | --- |
| `cited` | The source states a value that another work reports. arity records that value from its own source, so that the value does not count two times. |
| `restated` | The source states again a value that it states in another place, for example in the abstract, and a report quotes that other place. |
| `other` | Another reason. The `note` must tell it. |

```json
"skipped": [
  { "block": "s1/p2", "quote": "Ki = 4.5 and 45 nM, respectively) [15]", "reason": "cited" }
]
```

The checks find each skipped quote in its block, like the quote of a report,
and the quote must contain a number. Coverage counts skipped numbers apart
from quoted numbers. A submission can hold only skipped quotes and no reports.

## Corrections

arity does not change a run after it stores it. To correct a run, submit a new
run of the same work. Give `replaces`, the number of the earlier run, and a
`note` that tells what the new run corrects:

```json
"replaces": 18,
"note": "The conditions now include the time and the temperature of the annealing."
```

The new run holds all of its reports again, not only the changed reports.
arity checks it like each other submission. Then arity:

- keeps the earlier run, its reports, and the checks by people of those
  reports,
- shows only the reports of the current runs in the lists, the figures, the
  counts, and the coverage,
- keeps the page of each earlier report, with a link to the report that has the
  same local `id` in the current run.

The number of a run is in the table of runs on the page of its work, and in
the name of its file in the archive. A run can replace one earlier run of the
same work, and only a run that no other run replaces. To correct a correction,
replace the newest run.

The checks by people stay with the earlier reports. A new report is a new
statement, so a person must check it again.

arity finds the stored entity of each entity in the new run as it does for each
run: by a registry identifier, or by the same entity id in an earlier run of
the work. Thus, to correct an entity, for example an entity that holds two
strains, give each correct entity a new id, and do not give it an identifier of
the incorrect entity. When only replaced runs name an entity, the lists leave
it out, and its page tells you so.

## An example

This value report is from a reading of PMC6274071, table 1:

```json
{
  "type": "value",
  "id": "dox_mcf7",
  "subject": ["dox"],
  "property": "ic50",
  "value": {
    "number": "7.30",
    "unit": "umol/L",
    "unit_text": "μM",
    "uncertainty": { "kind": "sd", "number": "0.84" },
    "statistic": "mean"
  },
  "conditions": [
    { "role": "cell_line", "entity": "mcf7" },
    { "property": "exposure_time", "value": { "number": "72", "unit": "h", "unit_text": "h" } }
  ],
  "method": { "label": "sulforhodamine B (SRB) colorimetric assay" },
  "evidence": [
    { "block": "t1/r24c5", "quote": "7.30 ± 0.84" },
    { "block": "t1/r1c5", "quote": "IC50 (μM)" },
    { "block": "t1/r2c5", "quote": "MCF-7" },
    { "block": "t1/r24c1", "quote": "Dox." },
    { "block": "t1/foot1", "quote": "IC50 values are presented as mean ± S.D. of three separate experiments" },
    { "block": "s3.3.2/p1", "quote": "doxorubicin as a reference drug for further 72 h" },
    { "block": "abstract/p1", "quote": "sulforhodamine B (SRB) colorimetric assay" }
  ]
}
```

Numbers are strings, so that arity keeps each digit that the source writes.
