const KINDS = new Map([
  ["retraction", "Retraction"],
  ["partial_retraction", "Partial retraction"],
  ["withdrawal", "Withdrawal"],
  ["removal", "Removal"],
  ["expression_of_concern", "Expression of concern"],
  ["correction", "Correction"],
  ["erratum", "Erratum"],
  ["corrigendum", "Corrigendum"],
  ["addendum", "Addendum"],
  ["clarification", "Clarification"],
  ["new_version", "New version"],
  ["new_edition", "New edition"],
  ["reinstatement", "Reinstatement"],
  ["other", "Other notice"],
]);

const SOURCES = new Map([
  ["crossref", "Crossref"],
  ["europe_pmc", "Europe PMC"],
  ["pmc", "PubMed Central"],
  ["retraction_watch", "Retraction Watch"],
  ["publisher", "the publisher"],
]);

const SERIOUS = new Set(["retraction", "partial_retraction", "withdrawal", "removal"]);

const SERIOUS_SIGN = "\u{26a0}\u{fe0e}";
const OTHER_SIGN = "\u{2139}\u{fe0e}";

/** The name of a notice kind. An unknown kind shows as its key. */
export function noticeLabel(kind: string): string {
  return KINDS.get(kind) ?? kind.replaceAll("_", " ");
}

/** The name of a source of notices. An unknown source shows as its key. */
export function sourceLabel(source: string): string {
  return SOURCES.get(source) ?? source.replaceAll("_", " ");
}

/** True for the kinds that withdraw all or part of a work. */
export function isSerious(kind: string): boolean {
  return SERIOUS.has(kind);
}

/** The sign that the site shows before a notice of this kind. */
export function noticeSign(kind: string): string {
  return isSerious(kind) ? SERIOUS_SIGN : OTHER_SIGN;
}

/** The kinds with the serious kinds first. The order is otherwise the same. */
export function seriousFirst(kinds: readonly string[]): string[] {
  return [...kinds.filter(isSerious), ...kinds.filter((kind) => !isSerious(kind))];
}

/** The names of the kinds, with the serious kinds first. */
export function noticeNames(kinds: readonly string[]): string {
  return seriousFirst(kinds).map(noticeLabel).join(", ");
}

/** The sign of the most serious kind. It is empty if there are no kinds. */
export function markerSign(kinds: readonly string[]): string {
  const first = seriousFirst(kinds)[0];
  return first === undefined ? "" : noticeSign(first);
}

/**
 * The text that a figure shows before a report whose source has notices: a
 * sign and the names of the kinds. It is empty for a source without notices.
 */
export function noticeMarker(kinds: readonly string[]): string {
  return kinds.length === 0 ? "" : `${markerSign(kinds)} ${noticeNames(kinds)}`;
}

/** A lookup from a work to the kinds of its notices. */
export function noticesOfWorks(
  works: readonly { id: number; notices: readonly string[] }[],
): (work: number) => readonly string[] {
  const kinds = new Map(works.map((work) => [work.id, work.notices]));
  return (work) => kinds.get(work) ?? [];
}
