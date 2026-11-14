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
| Report | One report: its value or effect, its conditions, its quotes, its checks, and its provenance. |
| Properties | Each property with its unit and its dimension. |
| Property | A figure and a table of all values of the property, in the unit of the property. |
| Entities | Each entity with its kind and its registry identifiers. |
| Entity | The reports about one entity. |
| Effects | A matrix of agents and targets, with one mark for each reported effect. |

Each table can download its rows as a CSV file.

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
  dot is a confirmed report. A hollow dot is an unchecked report. A diamond is
  a value that the reader read from a plot. A line shows the uncertainty.
- In the effect matrix, the shape and the color show the direction: up and red
  for an increase, down and blue for a decrease, a gray dot for no change (0),
  and a diamond for a mixed result.
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
