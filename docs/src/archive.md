# Archive and rebuild

The archive is a set of files from which arity can make the same database
again. The repository [arity-data](https://github.com/collide2954/arity-data)
holds the public archive, under CC0 1.0.

The archive holds no file of a source. For each file, it holds the address and
the SHA-256 digest. A rebuild downloads each file again and makes sure that it
is the same file.

## Write the archive

```sh
arity archive ../arity-data
```

The archive has one folder for each work, `works/<identifier>/`. The
identifier is the PMCID of the work, or its DOI or PMID.

- `work.json` holds the record of the work in the format `arity-archive/1`: the
  place of the work in the catalogue, its metadata, its queue entry, the files
  that arity downloaded, the digests of its canonical texts, its notices, and
  its runs with the checks of their reports.
- `run-<sequence>.json` holds each submission, byte for byte. The sequence is
  the place of the run among all runs, and the number of the run in the
  database. The digest of the file is in `work.json`.

A run does not change. If the archive has another file at the place of a run,
`arity archive` stops with an error.

## Rebuild the database

```sh
arity --data new-data rebuild ../arity-data
```

The data folder must have no works. A rebuild:

1. adds the works in the order of the catalogue,
2. downloads each file from its address, and stops if a digest is not the
   archived digest,
3. makes each canonical text again with the version of the parser that the
   archive names, and stops if its digest is not the archived digest,
4. adds the notices,
5. stores the runs in their first order, with the checks of this version of
   arity, and stops if a submission does not pass or if a run does not get
   its number in the archive (a [correction](submissions.md#corrections)
   names the run that it replaces by this number),
6. records the checks of the reports, with their persons and times,
7. puts back the queue entries and the times of the works.

The rebuilt database has the same records. When you write its archive again,
you get the same files.

## When a rebuild stops

- A source can change a file at its address. The rebuild then stops, and names
  the address.
- The archive can name a version of the parser that this arity does not have.
  The rebuild then stops, and names that version. Each version of the parser
  always makes the same text from the same file, so an archived text never
  needs a new parser.
- A new version of the checks can refuse an old submission. The rebuild then
  stops, and lists the issues.

In each case, the archive is not the problem: it records what was true. A
person must decide what to do, for example read the work again.

## The public website

The workflow `.github/workflows/pages.yml` of arity makes the public website.
It clones arity-data, rebuilds the database, exports the website data, builds
the website and this book, and publishes them on GitHub Pages. The workflow
runs each day, after each change to the main branch of arity, and when a
maintainer starts it:

```sh
gh workflow run pages.yml --repo collide2954/arity
```

So a change to arity-data is on the website within one day, or at once when a
maintainer starts the workflow.

A second workflow, in arity-data, reads the notices of each work again each
Monday and commits the new ones. See [Notices](notices.md).
