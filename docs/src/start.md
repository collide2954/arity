# Start

## Build arity

You need Rust. The file `rust-toolchain.toml` selects the version, and
`rustup` installs it.

```sh
cargo build --release
```

The program is `target/release/arity`.

For the website, you need Node.js 24 or later and pnpm.

```sh
pnpm --dir web install
pnpm --dir web dev
```

## Make a data folder

arity keeps its database and the downloaded files in a data folder. The
default folder is `data`. To use another folder, give `--data` or set the
variable `ARITY_DATA`.

```sh
arity init --name "Your Name" --orcid 0000-0000-0000-0000
```

arity records this name with each check that you make. The settings are in
`arity.toml` in the data folder:

```toml
[person]
name = "Your Name"
orcid = "0000-0000-0000-0000"

[http]
contact = "you@example.org"
```

arity sends the contact address only to Crossref, which asks its clients for
one. If you do not set it, arity sends no address.

## Find and download works

```sh
arity search 'doxorubicin AND "MCF-7" AND IC50' --limit 5
arity add PMC6274071
arity queue
arity fetch --next 1
```

- `search` sends the query to Europe PMC. It adds the open-access works that it
  finds to the queue, in the order that it finds them.
- `add` adds works by PMCID, DOI, or PMID.
- `fetch` downloads the JATS XML, the PDF, and the supplementary files from the
  PMC open-access store. Then it makes the canonical text.

## Read a work

```sh
arity show PMC6274071
```

`show` prints the canonical text with the key of each block, for example
`[t1/r24c5] 7.30 ± 0.84`. A submission uses these keys and quotes. See
[Submissions](submissions.md).

```sh
arity submit submission.json --dry-run
arity submit submission.json
```

A dry run does the checks and stores nothing.

```sh
arity coverage PMC6274071
```

`coverage` lists each number of the text that no report quotes yet. See
[Coverage](checks.md#coverage).

## Check a report

```sh
arity check 12 confirm --note "The value agrees with Table 1."
arity check 13 reject --note "The source states mg/mL."
```

## Read the notices again

```sh
arity notices
```

## Publish

```sh
arity export web/public/data
arity archive ../arity-data
```

`export` writes the data files of the website. `archive` writes the archive.
See [Website](website.md) and [Archive and rebuild](archive.md).

## Look up a vocabulary

```sh
arity vocab kinds
arity vocab prefixes
arity vocab units
arity vocab properties
arity vocab notices
arity vocab issues
arity schema
```

`schema` prints the JSON Schema of the submission format.
