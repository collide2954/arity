# Notices

A notice is a statement about a work: a retraction, a correction, an
expression of concern, and others. See [Notice kinds](reference/notices.md).

## Notices do not remove data

arity does not remove or change a report because of a notice. A retracted work
keeps its reports, and the website shows each notice next to them:

- The page of the work lists each notice, with its source and its date.
- The list of works shows a label for each kind of notice.
- In the tables, each report has the notices of its source.
- In the figures, the label of a value starts with the notice, for example
  "⚠ Retraction". In the effect matrix, a sign shows above the mark.

A reader of the website can then decide what the notice means for the data.

## Sources of notices

arity reads notices from three sources:

- Crossref: the `updated-by` relations of a DOI. These include the retractions
  that Retraction Watch records.
- Europe PMC: the comments and corrections of a work, for example "Retraction
  in" or "Erratum in".
- The PMC open-access store: the `is_retracted` field of each article.

## Each notice as its source states it

arity stores each notice with its source and with the type name that the source
uses, for example `retraction` from Crossref or "Retraction in" from Europe PMC.
It does not merge the notices of different sources: when three sources report
one retraction, the work has three notices. arity also records when it read
each notice.

`arity fetch` reads the notices of the PMC store. `arity notices` reads the
notices of all three sources again, for each work or for the works that you
name.

## New notices on the public website

Each Monday, the workflow `.github/workflows/notices.yml` of
[arity-data](https://github.com/collide2954/arity-data) rebuilds the database
and reads the notices of each work again. When a source states a new notice,
the workflow commits it to arity-data, and the next daily build of the website
shows it. So the website shows a new retraction within about one week after a
source states it. The workflow commits only new notices: when no source states
a new notice, nothing changes.
