# arity

arity collects the values and effects that open-access research articles
report. It keeps each value with its source: the article, the location in the
article, and the exact text. You can compare values across articles,
experiments, and conditions. You can open the article at the location of each
value.

arity does not make judgements. It does not rate, rank, combine, or correct the
data that it collects. It shows what each source reports. It also shows the
status of each source, for example a retraction or a correction.

## Status

arity is in early development. The data model and the interfaces can change.

## Principles

- **Reports only.** Each record is a report of what one source states. arity
  does not calculate consensus values or pooled estimates.
- **Provenance for each value.** Each report keeps the source article, the
  location in the text, the exact quote, the reader, the responsible person,
  and the software version.
- **Checks before trust.** The software checks each submission against the
  source text. Then a person confirms or rejects each report. The website shows
  the check state of each report.
- **Notices stay visible.** arity records retractions, corrections, and
  expressions of concern. It does not remove data. It shows each notice with
  the data.
- **Open data.** You can download the full database as one SQLite file.

## How it works

1. **Catalogue.** arity finds open-access articles and adds them to a queue.
2. **Fetch.** arity downloads the full text and the PDF of each article. It
   records the URL, the date, and the SHA-256 hash of each file.
3. **Read.** A reader reads the article and writes a submission file. The
   reader can be a person or a software agent.
4. **Check.** arity checks the submission against the source text. Each quote
   must occur in the source. Each number must occur in its quote. Each unit
   must agree with its property.
5. **Confirm.** A person compares each report with its highlighted source text,
   and confirms or rejects the report.
6. **Archive.** arity writes the submissions and the checks to an archive.
   From the archive, arity can make the same database again.

## Website and data

- The website: <https://collide2954.github.io/arity/>
- The documentation: <https://collide2954.github.io/arity/docs/>
- The data: [arity-data](https://github.com/collide2954/arity-data), the
  archive of the submissions and the checks, under CC0 1.0.

GitHub Actions makes the database again from arity-data and publishes the
website each day, and after each change to arity.

## Build and run

You need Rust (the file `rust-toolchain.toml` selects the version), and for
the website, Node.js 24 or later and pnpm.

```sh
cargo build --release
target/release/arity init --name "Your Name"
target/release/arity search 'doxorubicin AND "MCF-7" AND IC50' --limit 5
target/release/arity fetch --next 1
target/release/arity show PMC6274071
```

To see the website with your data:

```sh
target/release/arity export web/public/data
pnpm --dir web install
pnpm --dir web dev
```

To run all checks:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm --dir web lint
pnpm --dir web check
pnpm --dir web test
```

## Documentation

The documentation is in [docs](docs/src/SUMMARY.md). To read it as a book,
install [mdBook](https://rust-lang.github.io/mdBook/) and run
`mdbook serve docs`. It tells how arity works, the submission format, the
checks, the units, the notices, the provenance, the archive, and the website.

## License

The code of arity is licensed under the Apache License, Version 2.0. See
[LICENSE](LICENSE) and [NOTICE](NOTICE). The data that arity makes is under
CC0 1.0. Each source keeps its own license. See
[Licenses](docs/src/licenses.md).
