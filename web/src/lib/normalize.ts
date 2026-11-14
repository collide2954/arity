/**
 * The normal form of text for quote search. This is the same rule as
 * `arity_text::normalize` in Rust; the shared vectors in
 * `testdata/text/normalization.json` keep the two equal.
 */

const REMOVED = new Set([0xad, 0x200b, 0x200c, 0x200d, 0x2060, 0xfeff]);
const SINGLE_QUOTES = new Set([0x2018, 0x2019, 0x201a, 0x201b, 0x2032, 0x2035]);
const DOUBLE_QUOTES = new Set([0x201c, 0x201d, 0x201e, 0x201f]);

/** The White_Space characters of Unicode, as Rust's `char::is_whitespace` has them. */
function isWhitespace(code: number): boolean {
  return (
    (code >= 0x09 && code <= 0x0d) ||
    code === 0x20 ||
    code === 0x85 ||
    code === 0xa0 ||
    code === 0x1680 ||
    (code >= 0x2000 && code <= 0x200a) ||
    code === 0x2028 ||
    code === 0x2029 ||
    code === 0x202f ||
    code === 0x205f ||
    code === 0x3000
  );
}

function isDash(code: number): boolean {
  return (
    (code >= 0x2010 && code <= 0x2015) ||
    code === 0x2212 ||
    code === 0xfe58 ||
    code === 0xfe63 ||
    code === 0xff0d
  );
}

export function fold(character: string): string {
  const code = character.codePointAt(0) ?? 0;
  if (isWhitespace(code)) return " ";
  if (isDash(code)) return "-";
  if (SINGLE_QUOTES.has(code)) return "'";
  if (DOUBLE_QUOTES.has(code)) return '"';
  return character;
}

export interface Normalized {
  /** The text in normal form. */
  readonly text: string;
  /** For each code point of `text`, the code point index in the original. */
  readonly origins: readonly number[];
}

export function normalize(input: string): Normalized {
  let text = "";
  const origins: number[] = [];
  let pending: number | null = null;
  let index = 0;
  for (const character of input) {
    if (!REMOVED.has(character.codePointAt(0) ?? 0)) {
      for (const decomposed of character.normalize("NFKC")) {
        const folded = fold(decomposed);
        if (folded === " ") {
          if (text.length > 0 && pending === null) pending = index;
          continue;
        }
        if (pending !== null) {
          text += " ";
          origins.push(pending);
          pending = null;
        }
        text += folded;
        origins.push(index);
      }
    }
    index += 1;
  }
  return { text, origins };
}

/** Finds each place where the quote occurs, as code point ranges of the original text. */
export function find(haystack: Normalized, quote: string): [number, number][] {
  const needle = Array.from(normalize(quote).text);
  if (needle.length === 0) return [];
  const points = Array.from(haystack.text);
  const ranges: [number, number][] = [];
  let index = 0;
  while (index + needle.length <= points.length) {
    let same = true;
    for (let offset = 0; offset < needle.length; offset += 1) {
      if (points[index + offset] !== needle[offset]) {
        same = false;
        break;
      }
    }
    if (same) {
      const start = haystack.origins[index];
      const last = haystack.origins[index + needle.length - 1];
      if (start !== undefined && last !== undefined) ranges.push([start, last + 1]);
      index += needle.length;
    } else {
      index += 1;
    }
  }
  return ranges;
}
