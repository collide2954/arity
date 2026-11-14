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
| `entities` | The entities that the reports are about. |
| `properties` | The properties that the reports use. |
| `reports` | The value reports and the effect reports. |

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
| `read_from_figure` | For a value that the reader read from a plot: the `figure` block, the `method`, and the `resolution`. |

## Effect reports

An effect report has `type: "effect"`, a local `id`, the `agents` (each an
entity with an optional `dose`), the `target` (an entity, with an optional
property), the `direction`, and the `evidence`. Optional fields:

| Field | Meaning |
| --- | --- |
| `relation` | `effect` (the default) or `association`. |
| `magnitude` | A `measure` such as `fold_change` or `percent_change`, a `number`, and a unit. |
| `comparator` | What the source compares with, for example a control. |
| `significance` | A `statement` (`significant` or `not_significant`), a `p` value with its `comparison`, and the `test`. |
| `n` | The number of samples. |
| `conditions`, `method`, `note` | As for a value report. |

## Conditions

A condition has a `role` with an `entity` or a `text`, or a `property` with a
`value`. The roles are `organism`, `cell_line`, `cell_type`,
`anatomical_site`, `sample`, `target`, `solvent`, `medium`, `co_treatment`,
`comparator`, `population`, `device`, `environment`, and `other`.

## Evidence

Each item of evidence is a `block` key and a `quote`. When the quote occurs more
than one time in the block, give `occurrence` (1 is the first).

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
