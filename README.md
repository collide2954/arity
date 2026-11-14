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

## License

arity is licensed under the Apache License, Version 2.0. See
[LICENSE](LICENSE) and [NOTICE](NOTICE).
