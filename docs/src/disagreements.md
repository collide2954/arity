# When readers disagree

Two readers can read the same place of a source and give different reports.
For example, a second language model reads a table again and gives 0.42 μM
where an earlier run gave 0.24 μM. This page tells what such a difference
means, and what arity and the people who keep its data do.

arity does not decide which reader is correct. It keeps each report with its
quotes and its provenance, and a person decides with the source.

## How a disagreement starts

- **A second reader reads the same places.** The second reader does not submit
  a place that a current run already reports, because the same value would
  then count two times. It compares its reading with the stored reports. For
  each report where it reads something else, it gives the place, both
  readings, and its quotes to a person who keeps the data of arity.
- **A checker compares reports with their sources.** The checker can be a
  person, or a language model that a person runs. The result of a language
  model is not a check by a person: arity records only the checks of people,
  and a person must make each check.
- **A person sees two reports of one place**, for example on the page of a
  work.

## What arity does not do

- It does not choose a report by a majority of readers, by the model of the
  reader, by the time of the run, or by a score.
- It does not average two values, or keep only one of them.
- It does not change or delete a stored run. A correction is a new run.

## The decision tree

Start with two current reports of one work about the same thing: the same
property of the same subject, or the same agents and target, from the same
place of the text.

1. **Do the two reports give the same value, in the unit of the property?**
   - Yes: the readers agree. A later run must not state the same value again.
     It skips the place with the reason `restated`, or a correction leaves the
     duplicate out.
   - No: go to step 2.
2. **Do the two reports give the same conditions?**
   - No: both reports can be correct, because the source can state one value
     for each condition. A person compares the conditions with the source. When
     a report does not give a condition that the source states, its run gets a
     correction (step 5).
   - Yes: go to step 3.
3. **Find the cause.** The checks make sure that the quotes of each report
   contain its numbers, so the difference is in how a reader reads the quotes:
   - The value comes from another row, column, or subject of a table. One
     reader made an error.
   - A reader read a superscript or a subscript as digits, for example 104 for
     10⁴. One reader made an error, or the text was an old version; see
     [Versions of the parser](text.md#versions-of-the-parser).
   - The units are not the same. One unit is wrong, or the source states the
     value in two units.
   - One reader read the value from a figure, and the other from the text. A
     reader reads a figure only when the text does not state the value, so the
     reading of the figure gets a correction.
   - The source states two different values for the same thing, for example in
     the abstract and in a table. Then no reader made an error. Both reports
     stay, and each has its quote and a note that tells the difference.
4. **A person checks.** A person compares each report with the source, and
   confirms or rejects it with a note: with `arity check`, or in the check mode
   of the website. A rejected report stays in the database. The website shows
   it pale, and the statistics, the boxes of the figures, and the coverage
   leave it out.
5. **The run gets a correction.** The reader of the incorrect report writes a
   new run with `replaces`; see [Corrections](submissions.md#corrections). The
   new run corrects each report with the same error. The responsible person of
   the run answers for the correction.

## An example

On 2026-11-15, two checkers, language models that a person ran, compared 50
reports with their sources. They agreed on 47 reports.

- For three reports, one checker rejected and one confirmed. The checker that
  rejected named the conditions in the methods that the reports left out: the
  annealing of a film (step 2), and the range of concentrations of a statement
  of no effect. On the request of the responsible person, the reader read the
  sources again, found that the rejections were correct, and corrected the
  runs. The correction of the statement of no effect also corrected 27 other
  reports with the same error.
- For one report, a checker confirmed the report, and noted that its entity
  held two strains of the rat. The cause was the identifier of the species on
  both strains. The reader corrected the run with one entity for each strain.

The earlier runs stay in the archive, and the website keeps the page of each
earlier report, with a link to the report that replaces it.
