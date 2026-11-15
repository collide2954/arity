import { fold } from "./normalize";

/**
 * The form of text for search: lower case, without accents, with the
 * compatibility forms of Unicode, and with one form of each dash, quote, and
 * space. For example "IC₅₀" becomes "ic50", and "µ" and "μ" become the same.
 */
export function searchForm(text: string): string {
  return text
    .normalize("NFKD")
    .replace(/\p{M}+/gu, "")
    .replace(/[^ -~]/gu, (character) => fold(character))
    .toLowerCase()
    .replace(/\s+/g, " ")
    .trim();
}

/** The terms of a search, in search form: each word, and each phrase in double quotes. */
export function terms(search: string): string[] {
  const text = Array.from(search, (character) => fold(character)).join("");
  const found: string[] = [];
  for (const match of text.matchAll(/"([^"]*)"?|([^\s"]+)/g)) {
    const term = searchForm(match[1] ?? match[2] ?? "");
    if (term.length > 0) found.push(term);
  }
  return found;
}

/** True when the text, in search form, has each term. */
export function matchesAll(text: string, search: readonly string[]): boolean {
  return search.every((term) => text.includes(term));
}
