# Read a work

This page is for each reader of arity: a person, or a language model that a
person runs. It tells the reader what to do, step by step, so that each
submission states only what its source states, and passes the checks.
`arity guide` prints this page.

## The rules

- **Record what the source states.** A report is one statement of one source.
  Do not add a judgement, a confidence, a correction, or a value that the
  source does not state. The format refuses fields such as `confidence`.
- **Quote exactly.** Each report has the exact words of the canonical text as
  its evidence. Copy the words from `arity show`, with their script
  characters, for example "10⁴", "IC₅₀", and "^{a}".
- **Give the conditions.** A value without its conditions can mean a
  different thing. Give each setting of the method that the source states for
  the value.
- **Do not combine.** Do not average, convert, or round a value. Give the
  number and the unit as the source writes them. arity converts the units.
- **Read the whole work.** A complete reading leaves no number of the text
  without a report or a skipped quote.
- **A person is responsible.** Each submission names its reader and the person
  who is responsible for it. A language model can do the reading. The
  responsible person answers for the submission, and people check its reports.

## Get the texts

Read the canonical text that arity makes, not the PDF or the page of the
publisher. The checks compare each quote with the canonical text.

To read a work that the archive already has, make the database of arity on
your computer from the archive. Then your texts are the same texts, with the
same digests:

```sh
git clone https://github.com/collide2954/arity-data
arity --data data init --name "Your Name"
arity --data data rebuild arity-data
```

To read a new work, add it and fetch its files:

```sh
arity add PMC6274071
arity fetch PMC6274071
```

Then print its text with the key of each block, and the entities that earlier
runs of the work name:

```sh
arity show PMC6274071
arity coverage PMC6274071
```

`arity coverage` lists each number of the text that no report quotes yet.
`arity vocab properties`, `arity vocab units`, `arity vocab kinds`, and
`arity vocab prefixes` list the stored properties, the units, the kinds of
entities, and the prefixes of the registries. `arity schema` prints the JSON
Schema of the format.

## Write the submission

A submission is one JSON file. See [Submissions](submissions.md) for each
field.

1. **Read the work.** Read the abstract, the methods, the results, each table,
   each caption, and the cells of the data files.
2. **List the entities.** Give each entity a local `id`, a `kind`, a `label`,
   and a mention: a quote of a place where the source names it.
   - Give a registry identifier in `xrefs` only when it identifies exactly
     this entity. arity links each entity with the same identifier to one
     stored entity. Thus a species identifier, for example `NCBITaxon:10116`,
     on two strains of the rat makes one entity of the two strains. Give a
     strain no identifier, and tell the species in its `note`.
   - Within one work, an id names the same entity in each run. Use the ids that
     `arity show` lists for the same entities, and new ids for other entities.
3. **List the properties.** Use a stored property when one fits. For a new
   property, give its `label`, its `value_type`, its UCUM `unit`, and a
   `description`.
4. **Write a value report for each value.** Give the `subject`, the `property`,
   the `value` with its UCUM `unit` and its `unit_text` as the source writes
   it, the uncertainty, the statistic, the number of samples, the `conditions`,
   the `method`, and the `evidence`.
   - Give each setting of the method as a condition: for example the cell line,
     the medium, the time of exposure, the temperature, the range of
     concentrations, the substrate, or the annealing.
   - When the source states the value in two places, for example in the
     abstract and in a table, give both quotes as the evidence of one report.
   - "10⁴" is the number 10000. "IC₅₀", "CO₂", and "cm⁻²" have no number
     other than the value before them.
   - Read a value from a figure only when the text and the tables do not state
     it. Then give `read_from_figure`.
5. **Write an effect report for each effect.** Give the `agents`, the
   `target`, the `direction`, the `comparator`, the `significance` with its
   test statistic, its correction, and its q-value, the `groups`, the
   `conditions`, and the `evidence`.
   - A statement of no effect is true only for the doses and the times that
     the source tested. Give them as conditions, for example the range of
     concentrations and the time of exposure.
6. **Skip the other numbers.** Skip a value that the source cites from another
   work with the reason `cited`, and a value that a report already quotes in
   another place with the reason `restated`. For another reason, give the
   reason `other` and a `note`.
7. **Check the submission.** Correct each error, and look at each warning:

   ```sh
   arity submit --dry-run submission.json
   ```

8. **Give the submission.** When the dry run passes, `arity submit` stores the
   submission in your database. To add it to the data of arity, give the file
   to a person who keeps the data of arity. That person submits it again, and
   archives it.

## Correct a run

arity never changes a stored run. When a person or a check finds an error, the
reader writes a new run of the same work with `replaces` and a `note`. See
[Corrections](submissions.md#corrections). Correct each report with the same
error, not only the report that a person found. To correct an entity, give the
correct entity a new id.

## Mistakes in earlier readings

These mistakes are in runs that later runs corrected:

- A value without the settings of its method, for example a band gap without
  the annealing of the film.
- A statement of no effect without the range of concentrations that the source
  tested.
- One entity for two strains of one species, because both had the identifier
  of the species.
- A digit of a superscript or of a subscript as a number, for example 104 for
  10⁴ in an old version of the text.

## When readers disagree

Two readers can give different reports for the same place of a source. arity
keeps both, and people decide with the source. See
[When readers disagree](disagreements.md).

## Checklist

- Each number of the text is in a report or in a skipped quote, so
  `arity coverage` lists no number.
- Each report gives the conditions that the source states for it.
- Each quote is the exact text of its block.
- Each entity has its correct identifier, or no identifier.
- `arity submit --dry-run` gives no errors, and each warning has a reason.
