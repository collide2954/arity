# Website

The website is a static site in Svelte and TypeScript. It reads the files that
`arity export` writes, so it needs no server. The public website is at
<https://collide2954.github.io/arity/>. See
[Archive and rebuild](archive.md#the-public-website) for how arity publishes it.

## Pages

| Page | What it shows |
| --- | --- |
| Works | The works in the order that arity found them, with their licenses, notices, and number of reports. |
| Work | The PDF, or the canonical text when there is no PDF, with a mark on each quote. The list of reports, the notices, and the files with their digests. |
| Reports | All reports in one table, with a search, filters, a sort, a choice of columns, and downloads. |
| Report | One report: its value or effect, its conditions, its quotes, its checks, and its provenance. |
| Properties | Each property with its unit and its dimension. |
| Property | A figure and a table of all values of the property, in the unit of the property, with the filters of the Reports page. |
| Entities | Each entity with its kind and its registry identifiers. |
| Entity | The reports about one entity. |
| Effects | A matrix of agents and targets, with one mark for each reported effect. |
| Compare | The reports of one property in a matrix. You select what the rows and the columns are. |

Each table can download its rows as a CSV file. The pages of works,
properties, entities, and effects also have a search box, filters, and columns
that you can sort, as on the Reports page. On the Effects page, the filters
change the matrix and the table.

## Find and filter reports

On the Reports page, type words in the search box. A report agrees with the
search when each word is in its property, its entities, its conditions, its
value, its method, its source, or its quotes. Put a phrase in double quotes to
find its words together. The search ignores case and accents. It also ignores
the differences between the forms of a character, for example "µ" and "μ", or
"IC₅₀" and "IC50".

The filters divide the reports by property, entity, condition, work, kind of
report, check, notice, origin of the value, and direction of the effect. A
report must agree with one of the values that you select in each filter. The
number next to a value is the number of reports that you see when you also
select that value. When you select one property, you can also give a range for
the converted value.

Select the triangle at the start of a row to see the quotes of the report, its
conditions, its method, and the note of the reader, without leaving the page.
The page of a report also lists the reports of other works about the same
thing: for a value, the same property and a subject in common; for an effect,
the same target and an agent in common.

The search on the start page uses the same rules. It finds works, properties,
entities, and reports, and it can open the Reports page with the same words.
The page of a work has a box that filters its list of reports.

Select the head of a column to sort the table by that column. Select it again
to sort in the opposite direction. The address of the page keeps the words, the
filters, the sort, the page, and the columns, so you can share a selection as a
link. A download gives all the rows that you select, not only the rows of the
page: a CSV file with all columns, or a JSON file with each report as arity
exports it.

## Check mode

A person compares each report with its source, and confirms or rejects it.
Select "Check mode" at the top of the website. Then the page of each report,
and the list of reports of each work and of each entity, have the buttons
Confirm and Reject, with a field for a note. The browser keeps your marks. On
the page "Your checks", give your name and, if you have one, your ORCID iD.
Then download the check file, and give it to a person who keeps the data of
arity. That person records it with `arity check --file`. See
[Check a report](start.md#check-a-report).

## Keys

| Key | What it does |
| --- | --- |
| Ctrl+K, or Cmd+K on a Mac | Opens a box that finds and opens a work, a property, an entity, a report, or a page. In the box, ↑ and ↓ select, Enter opens, and Esc closes. |
| / | Goes to the search box of the page. |
| ↑ and ↓ | In a table, go to the row above or below. |

## Compare reports in a matrix

On the Compare page, select a property. Then select what the rows and the
columns of the matrix show: the subject, the work, the method, or a condition,
for example the cell line or the exposure time. Each cell lists each report of
its row and its column, with a link to the report. arity does not average or
combine the reports of a cell. A report that does not state a value for a row
or a column goes into "not stated". The cells can show the values as the
sources report them, or the converted values in the unit of the property. The
filters of the Reports page also apply, and the CSV file has one line for each
report.

## Marks on the PDF

On the page of a work, each quote has a mark on the PDF. Put the pointer on a
mark to see its report. Select a mark to go to its report in the list. Select a
report in the list to go to its main quote in the PDF.

The text of a PDF is not always the same as the canonical text. A PDF can split
words, use ligatures, and give the wrong character for a symbol. Thus arity
ignores white space, hyphens, symbols, and case when it finds a quote in the
PDF. When the PDF has the quote in more than one place, arity compares the text
around each place with the text around the quote in the canonical text. The
mark goes on the place where the most text agrees. A short quote that is in
more than one place, with too little text around it that agrees, gets no mark.
Then the list of reports tells you that arity cannot find the quote in the PDF,
and the text view shows the quote.

## Figures

- In the figure of a property, each dot is one report of one source. A filled
  dot is a confirmed report. A hollow dot is an unchecked report. A pale dot is
  a rejected report. A diamond is a value that the reader read from a plot. A
  line shows the uncertainty.
- The figure of a property can show one row for each report, for each
  subject, or for each work. In a row of a subject or of a work, each report
  keeps its own dot, and dots that are near each other go to separate places
  in the row. When you do not select the rows, the figure shows one row for
  each report if the property has 60 reports or fewer. Else it shows one row
  for each subject if there are 40 subjects or fewer, and one row for each
  work if there are more.
- Drag along the axis of values to select a range. Then the figure and the
  table show only the values in that range. Put the pointer on a row of the
  table to make the dot of its report larger.
- The figure of effects has three forms. The tiles give one square for each
  report, in the order of the directions, so the length of a row shows its
  number of reports. The list gives each direction its own column. The matrix
  puts the agents against the targets. In the tiles and the list, you select
  what a row is: an agent, a target, an agent and a target, a work, or a
  condition, for example the cell line.
- In the figures of effects, the color shows the direction: red for an
  increase, blue for a decrease, and gray for no change (0) or a mixed result.
  In the list and the matrix, the shape also shows the direction: up for an
  increase, down for a decrease, a dot for no change, and a diamond for a mixed
  result. In the tiles, a square with a dark outline comes from a source with a
  notice.
- When a source has a notice, the figures show it. See [Notices](notices.md).

The website has a light theme and a dark theme, and it follows the setting of
your system.

## Build the website

```sh
arity export web/public/data
pnpm --dir web build
```

The site is in `web/dist`. To serve it from a folder, for example `/arity/` on
GitHub Pages, set the variable `ARITY_BASE` to that folder before the build.
