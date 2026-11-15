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

## Versions of the parser

Each text records the version of the parser that made it, and the version is
part of the digest. A version always makes the same text from the same file,
so a rebuild makes each archived text again with its own version.

| Version | What it writes |
| --- | --- |
| `arity-jats 0.1.0` | Superscripts and subscripts as plain characters, so `10<sup>4</sup>` becomes "104". A formula that has only TeX as its TeX source. |
| `arity-jats 0.2.0` | The digits and the signs of superscripts and subscripts as script characters: "10⁴", "cm⁻²", "Ca²⁺", "IC₅₀". Other scripts with a mark and braces: "^{a}" for the footnote mark a, "λ_{max}". A script of only symbols, for example "*" or "®", as it is. Fractions as "a/b", roots as "√", and the brackets of MathML formulas. A formula that has only TeX as plain text, for example "IC₅₀=2.3±0.4 μM", without the TeX preamble. |
| `arity-jats 0.3.0` | The text of `arity-jats 0.2.0`, and the variant Greek letters of TeX as their letters: `\varDelta` becomes "Δ", and `\varrho` becomes "ϱ". Version 0.2.0 writes the name of the command, for example "varDelta". |

New texts use the latest version. A sign becomes a script character only at
the start or at the end of a script, so that `C<sub>1–2</sub>` keeps the
meaning of a range and becomes "C_{1–2}". A MathML formula gets the separators
of its brackets only when it names them.

## Move readings to a new version

```sh
arity update-text --dry-run
arity update-text
```

`arity update-text` makes the text of each work again with the latest version
of the parser, from the same files. Then it moves each current run of the work
to the new text, as a new run that [replaces](submissions.md#corrections) it:

- Each quote gets the words of the new text at the same place. arity aligns
  the old block with the new block, so "IC50 (μM) a" becomes
  "IC₅₀ (μM) ^{a}".
- A skipped quote that has no number in the new text leaves the run. For
  example, the old text wrote "CoV₂O₆" as "CoV2O6", so a reader had to skip
  the 2 and the 6.
- The values, the entities, and the conditions stay the same. The note of the
  new run tells what changed.
- The new run passes the same checks as each other submission.

When one run of a work cannot move, no run of the work moves, and arity lists
the problems. For example, a reader could read `10<sup>4</sup>` in the old text
as the number 104. The new text writes "10⁴", so the check of that value fails,
and a reader must correct the run first. Without works, the command updates
each work. `--dry-run` shows what would change, and changes nothing.

A new version can change the words of only some texts. For example, version
0.3.0 changes only the texts with variant Greek letters in TeX. The other texts
can stay on their version, because a rebuild makes each text with the version
that made it. To move only the works whose words change, name them:

```sh
arity update-text --dry-run PMC13694592
arity update-text PMC13694592
```

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

A superscript after a number is a power of ten, with one reading: "2 × 10⁻³"
is 0.002, and "10⁴" is 10000. The same form in plain characters, "2 × 10−3",
can also be 2, 10, and 3, so arity keeps each of those readings. A superscript
after a letter, for example the "−2" of "cm⁻²" or the "13" of "¹³C", a
subscript, for example the "50" of "IC₅₀", and a script with a mark, for
example the citations "^{59,60}", are not numbers.
