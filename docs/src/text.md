# Canonical text

arity reads each work from its canonical text, in the format `arity-text/1`.
The canonical text divides the work into blocks, and each block has a stable
key.

| Key | Block |
| --- | --- |
| `title` | The title. |
| `abstract/p1` | The first paragraph of the abstract. |
| `s1.2/p3` | The third paragraph of section 1.2. |
| `t1/caption` | The caption of table 1. |
| `t1/r3c5` | The cell in row 3, column 5 of table 1. |
| `t1/foot1` | The first footnote of table 1. |
| `f3/caption` | The caption of figure 3. |
| `supp1/caption` | The caption of the first supplementary file. |
| `supp1/s1/r2c3` | The cell in row 2, column 3 of the first sheet of that file. |

arity makes the blocks from the JATS XML of the work. It also reads the
supplementary data files (XLSX, XLS, ODS, and CSV), and adds each cell as a
block. The text lists each of these files as an attachment, with its digest.

## The digest of a text

The SHA-256 digest of the canonical JSON identifies the text. A submission
names this digest. The checks refuse a submission that names another text, so
each quote refers to the text that the reader read.

## Normalization

Before arity compares a quote with a block, it normalizes both:

- It applies NFKC to each character. For example, "µ" (micro sign) becomes "μ"
  (mu), and "ﬁ" becomes "fi".
- It changes each dash and minus sign to "-", and each curly quotation mark to a
  straight one.
- It removes soft hyphens and zero-width characters.
- It changes each run of white space to one space.

arity keeps a map from each normalized position back to the original text, so
that it can mark the exact part of the source.

The Rust code and the TypeScript code of the website use the same rules. The
file `testdata/text/normalization.json` holds the test vectors that both use.

## Numbers

The checks find each number in the quotes. They read scientific notation
(×10⁻³, e-notation, and forms such as 10-3), thousands separators, decimal
commas, and signs. A hyphen is a minus sign only where a sign can occur: in
"IL-6", the number is 6, not −6. When a form has more than one reading, arity
keeps each reading.
