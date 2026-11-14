/**
 * Finds quotes in the text of a PDF page. The text of a page comes in items,
 * and the items can split words, break lines with hyphens, and use
 * ligatures. So the search ignores white space, hyphens, and case.
 */

import { fold } from "./normalize";

export interface Position {
  /** The index of the text item. */
  readonly item: number;
  /** The offset in UTF-16 code units in the text of the item. */
  readonly offset: number;
}

export interface Match {
  readonly start: Position;
  /** The position after the last character of the match. */
  readonly end: Position;
}

function loose(character: string): string {
  let out = "";
  for (const decomposed of character.normalize("NFKC")) {
    const folded = fold(decomposed);
    if (folded === " " || folded === "-") continue;
    out += folded.toLowerCase();
  }
  return out;
}

function looseText(text: string): string {
  let out = "";
  for (const character of text) out += loose(character);
  return out;
}

interface Page {
  readonly characters: string[];
  readonly starts: Position[];
  readonly ends: Position[];
}

function index(items: readonly string[]): Page {
  const characters: string[] = [];
  const starts: Position[] = [];
  const ends: Position[] = [];
  items.forEach((text, item) => {
    let offset = 0;
    for (const character of text) {
      const width = character.length;
      for (const part of Array.from(loose(character))) {
        characters.push(part);
        starts.push({ item, offset });
        ends.push({ item, offset: offset + width });
      }
      offset += width;
    }
  });
  return { characters, starts, ends };
}

/** Finds each place where the quote occurs in the text items of a page. */
export function anchor(items: readonly string[], quote: string): Match[] {
  const needle = Array.from(looseText(quote));
  if (needle.length === 0) return [];
  const page = index(items);
  const matches: Match[] = [];
  let position = 0;
  while (position + needle.length <= page.characters.length) {
    let same = true;
    for (let offset = 0; offset < needle.length; offset += 1) {
      if (page.characters[position + offset] !== needle[offset]) {
        same = false;
        break;
      }
    }
    const start = page.starts[position];
    const end = page.ends[position + needle.length - 1];
    if (same && start && end) {
      matches.push({ start, end });
      position += needle.length;
    } else {
      position += 1;
    }
  }
  return matches;
}
