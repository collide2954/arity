/**
 * Finds quotes in the text of a PDF. The text of a page comes in items, and
 * the items can split words, break lines with hyphens, and use ligatures. So
 * the search ignores white space, hyphens, and case.
 */

import { fold } from "./normalize";

export interface Position {
  /** The index of the text item on its page. */
  readonly item: number;
  /** The offset in UTF-16 code units in the text of the item. */
  readonly offset: number;
}

export interface Match {
  /** The index of the page, from 0. */
  readonly page: number;
  readonly start: Position;
  /** The position after the last character of the match. */
  readonly end: Position;
}

/** A quote, with the text around it in its document. */
export interface Quote {
  readonly text: string;
  readonly before: string;
  readonly after: string;
  /** The position of the quote in its document, from 0 at the start to 1 at the end. */
  readonly at: number;
}

/**
 * The number of characters, of a quote and of the text around it, that must
 * agree with the PDF when the quote occurs more than one time in the PDF.
 */
const CERTAIN = 8;

function loose(character: string): string {
  const decomposed = character.charCodeAt(0) < 0x80 ? character : character.normalize("NFKC");
  let out = "";
  for (const part of decomposed) {
    const folded = fold(part);
    if (folded !== " " && folded !== "-") out += folded.toLowerCase();
  }
  return out;
}

function looseText(text: string): string {
  let out = "";
  for (const character of text) out += loose(character);
  return out;
}

/** The loose text of all pages, with the page and the positions of each UTF-16 code unit. */
interface Pages {
  readonly text: string;
  readonly pages: readonly number[];
  readonly starts: readonly Position[];
  readonly ends: readonly Position[];
}

function read(items: readonly (readonly string[])[]): Pages {
  const parts: string[] = [];
  const pages: number[] = [];
  const starts: Position[] = [];
  const ends: Position[] = [];
  items.forEach((texts, page) => {
    texts.forEach((text, item) => {
      let offset = 0;
      for (const character of text) {
        const folded = loose(character);
        for (let unit = 0; unit < folded.length; unit += 1) {
          pages.push(page);
          starts.push({ item, offset });
          ends.push({ item, offset: offset + character.length });
        }
        parts.push(folded);
        offset += character.length;
      }
    });
  });
  return { text: parts.join(""), pages, starts, ends };
}

/** The number of characters on the two sides of a match that agree with the text around the quote. */
function agreement(
  text: string,
  start: number,
  end: number,
  before: string,
  after: string,
): number {
  let back = 0;
  while (back < before.length && text[start - 1 - back] === before[before.length - 1 - back]) {
    back += 1;
  }
  let ahead = 0;
  while (ahead < after.length && text[end + ahead] === after[ahead]) ahead += 1;
  return back + ahead;
}

/**
 * A search of the text items of each page of a PDF. When a quote occurs more
 * than one time, the search chooses the place where the most text around it
 * agrees with the text around the quote, and then the place nearest to the
 * position of the quote in its document. A short quote that occurs more than
 * one time and has too little text around it that agrees has no place.
 */
export function finder(items: readonly (readonly string[])[]): (quote: Quote) => Match | null {
  const all = read(items);
  return (quote) => {
    const needle = looseText(quote.text);
    if (needle === "") return null;
    const before = looseText(quote.before);
    const after = looseText(quote.after);
    let best: { at: number; score: number; distance: number } | null = null;
    let count = 0;
    for (let at = all.text.indexOf(needle); at !== -1; at = all.text.indexOf(needle, at + 1)) {
      const end = at + needle.length;
      if (all.pages[at] !== all.pages[end - 1]) continue;
      count += 1;
      const score = agreement(all.text, at, end, before, after);
      const distance = Math.abs(at / all.text.length - quote.at);
      if (!best || score > best.score || (score === best.score && distance < best.distance)) {
        best = { at, score, distance };
      }
    }
    if (!best || (count > 1 && needle.length + best.score < CERTAIN)) return null;
    const page = all.pages[best.at];
    const start = all.starts[best.at];
    const end = all.ends[best.at + needle.length - 1];
    return page === undefined || !start || !end ? null : { page, start, end };
  };
}
