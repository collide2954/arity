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
  dose, the value of each condition, the magnitude, and the p-value.
- They find the unit text in the quotes, read it as a unit, and make sure that
  it is the same unit as the UCUM unit of the value.
- They make sure that each unit measures the dimension of its property.
- They make sure that each entity and each property that a report names is in
  the submission or in the database.
- They make sure that a registry identifier can identify an entity of that
  kind.
- They make sure that a value read from a plot quotes the caption of its figure.

## Cue words

Some parts of a report do not have a number to find. For these parts, the
checks look for cue words in the quotes:

- For a bound such as `gt`, a quote must have a sign such as ">" or a word such
  as "more than".
- For a direction such as `decrease`, a quote must have a word such as
  "decreased", "reduced", or "lower". For `no_change`, a quote must have a word
  such as "no", "not", "unchanged", or "similar".

When a quote has no cue word, the checks give a warning, and a person must
check the report.

## After the checks

A submission that passes is stored as a run, with its warnings. Its reports
are unchecked. People compare them with their sources, and confirm or reject
them with `arity check`.
