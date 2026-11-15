# Checks

The checks compare a submission with the canonical text of its work. They do
not decide whether a source is correct. They make sure that each report states
only what its quotes state.

Each problem has a stable code, a JSON Pointer to the part of the submission,
and a message. An error stops the submission. A warning lets the submission go
into the database, and a person must look at the part that the warning names.
See [Issue codes](reference/issues.md) for the full list.

## What the checks do

- They make sure that the submission names the current text of the work.
- They find each quote in its block, after [normalization](text.md).
- They find each number of a report in its quotes: the value or the ends of
  its interval, the uncertainty and its level, the number of samples, each
  dose, the value of each condition, the magnitude, the p-value, the q-value,
  the statistic of a test and its degrees of freedom, and the number of
  samples of each group.
- They find the unit text in the quotes, read it as a unit, and make sure that
  it is the same unit as the UCUM unit of the value.
- They make sure that each unit measures the dimension of its property.
- They make sure that each entity and each property that a report names is in
  the submission or in the database.
- They make sure that a registry identifier can identify an entity of that
  kind.
- They make sure that a value read from an image quotes the caption of its
  figure or its table. A value can come from the image of a table only when
  the text has no cells of that table. For a value from an image, the checks do
  not look for its numbers in the quotes, and a unit that no quote contains
  gives a warning, so that a person checks it against the image.

## Cue words

Some parts of a report do not have a number to find. For these parts, the
checks look for cue words in the quotes:

- For a bound such as `gt`, a quote must have a sign such as ">" or a word such
  as "more than".
- For a direction such as `decrease`, a quote must have a word such as
  "decreased", "reduced", or "lower". For `no_change`, a quote must have a word
  such as "no", "not", "unchanged", or "similar".
- For a correction for multiple tests such as `bonferroni`, a quote must name
  it, for example "Bonferroni".

When a quote has no cue word, the checks give a warning, and a person must
check the report.

## Coverage

The checks make sure that each report is correct. Coverage shows what is left
to read. `arity coverage PMC6274071` lists each number of the text that no
report quotes:

```text
Table cells: 340 numbers. Reports quote 6, readers skipped 0, and 334 are left.
Text with a unit or a sign: 127 numbers. Reports quote 6, readers skipped 0, and 121 are left.

[t1/r3c5] 19.53 ± 1.05  (19.53, 1.05)
```

arity counts these numbers:

- each number in a table cell or a cell of a data file, other than a header
  cell,
- each number in the other blocks that has a unit, a percent sign, or an
  uncertainty after it, or a sign such as `=`, `<`, or `~` before it.

arity does not count a number that is a part of a name or a code, for example
`IC50`, `CO2`, `5o`, or `MCF-7`. A number is quoted when the quote of a report
that no person rejected contains it. The page of each work on the website shows
the same counts, and the text view marks each number that is left with a dotted
line.

A complete reading leaves no number. Record each value that the work measured,
with the settings of its method as conditions, for example a temperature or a
time of incubation. When the work states the same value two times, for example
in the abstract and in a table, give both quotes as the evidence of one report.
When an earlier run already reports the value, skip the second place with the
reason `restated`. A value that the work cites from another work is not a
report of this work: skip it with the reason `cited`, so that it does not count
two times. See [Skipped quotes](submissions.md#skipped-quotes).

A reading can take more than one run. Within one work, an entity id names the
same entity in each run, so a later run can add the values of an entity that an
earlier run named. `arity show` lists these ids.

## After the checks

A submission that passes is stored as a run, with its warnings. Its reports
are unchecked. People compare them with their sources, and confirm or reject
them with `arity check`.
