import { matchesAll, terms } from "./search";
import { compareValues } from "./sort";

/** What a person chooses to see: words to find, values of facets, and a range of numbers. */
export interface Selection<F extends string> {
  readonly text: string;
  /** For each facet, the values to keep. An item needs one of them. A facet without values keeps all items. */
  readonly facets: Readonly<Partial<Record<F, readonly string[]>>>;
  readonly min: number | null;
  readonly max: number | null;
}

/** A thing to filter: its text in search form, its values for each facet, and its number. */
export interface Item<F extends string> {
  readonly text: string;
  readonly values: Readonly<Partial<Record<F, readonly string[]>>>;
  readonly number: number | null;
}

/** One value of a facet, with the number of items that have it. */
export interface Option {
  readonly value: string;
  readonly label: string;
  readonly count: number;
  readonly selected: boolean;
}

export interface Filtered<F extends string, T> {
  readonly items: T[];
  /** For each facet, the number of items with each value, when the other parts of the selection apply. */
  readonly counts: ReadonlyMap<F, ReadonlyMap<string, number>>;
}

export function emptySelection<F extends string>(): Selection<F> {
  const facets: Partial<Record<F, readonly string[]>> = {};
  return { text: "", facets, min: null, max: null };
}

export function isEmpty<F extends string>(selection: Selection<F>): boolean {
  return (
    selection.text.trim() === "" &&
    Object.keys(selection.facets).length === 0 &&
    selection.min === null &&
    selection.max === null
  );
}

function numberOf(text: string | null): number | null {
  if (text === null || text.trim() === "") return null;
  const value = Number(text);
  return Number.isFinite(value) ? value : null;
}

/** Reads a selection from the parameters of an address: `q`, one parameter for each facet, `min`, and `max`. */
export function readSelection<F extends string>(
  params: URLSearchParams,
  facets: readonly F[],
): Selection<F> {
  const chosen: Partial<Record<F, readonly string[]>> = {};
  for (const facet of facets) {
    const values = [...new Set(params.getAll(facet))].filter((value) => value.length > 0);
    if (values.length > 0) chosen[facet] = values;
  }
  return {
    text: params.get("q") ?? "",
    facets: chosen,
    min: numberOf(params.get("min")),
    max: numberOf(params.get("max")),
  };
}

export function writeSelection<F extends string>(
  params: URLSearchParams,
  selection: Selection<F>,
  facets: readonly F[],
): void {
  if (selection.text.trim() !== "") params.set("q", selection.text);
  for (const facet of facets) {
    for (const value of selection.facets[facet] ?? []) params.append(facet, value);
  }
  if (selection.min !== null) params.set("min", String(selection.min));
  if (selection.max !== null) params.set("max", String(selection.max));
}

/** The selection with these values for the facet. No values remove the facet. */
export function withValues<F extends string>(
  selection: Selection<F>,
  facet: F,
  values: readonly string[],
): Selection<F> {
  const facets: Partial<Record<F, readonly string[]>> = {};
  for (const key of Object.keys(selection.facets) as F[]) {
    const kept = selection.facets[key];
    if (key !== facet && kept !== undefined) facets[key] = kept;
  }
  if (values.length > 0) facets[facet] = values;
  return { ...selection, facets };
}

/** Adds a value that is not in the facet, or removes a value that is. */
export function toggle<F extends string>(
  selection: Selection<F>,
  facet: F,
  value: string,
): Selection<F> {
  const current = selection.facets[facet] ?? [];
  return withValues(
    selection,
    facet,
    current.includes(value) ? current.filter((other) => other !== value) : [...current, value],
  );
}

function inRange(number: number | null, min: number | null, max: number | null): boolean {
  if (min === null && max === null) return true;
  if (number === null) return false;
  return (min === null || number >= min) && (max === null || number <= max);
}

/**
 * The items that the selection keeps, and the counts of the facet values.
 * An item that fails one facet only counts for that facet, so that each
 * count tells how many items a change of that facet adds.
 */
export function filter<F extends string, T extends Item<F>>(
  items: readonly T[],
  selection: Selection<F>,
  facets: readonly F[],
): Filtered<F, T> {
  const words = terms(selection.text);
  const wanted = facets.flatMap((facet) => {
    const values = selection.facets[facet] ?? [];
    return values.length > 0 ? [{ facet, values: new Set(values) }] : [];
  });
  const counts = new Map(facets.map((facet) => [facet, new Map<string, number>()]));
  const kept: T[] = [];
  for (const item of items) {
    if (!inRange(item.number, selection.min, selection.max) || !matchesAll(item.text, words)) {
      continue;
    }
    const failed = wanted
      .filter(({ facet, values }) => !(item.values[facet] ?? []).some((value) => values.has(value)))
      .map(({ facet }) => facet);
    if (failed.length === 0) kept.push(item);
    const counted = failed.length === 0 ? facets : failed.length === 1 ? failed : [];
    for (const facet of counted) {
      const count = counts.get(facet);
      if (!count) continue;
      for (const value of item.values[facet] ?? []) count.set(value, (count.get(value) ?? 0) + 1);
    }
  }
  return { items: kept, counts };
}

/** The values of a facet: the chosen values first, in the order of choice, then the others by count and by label. */
export function options<F extends string>(
  facet: F,
  counts: ReadonlyMap<string, number>,
  selection: Selection<F>,
  label: (value: string) => string,
): Option[] {
  const chosen = selection.facets[facet] ?? [];
  const first = chosen.map((value) => ({
    value,
    label: label(value),
    count: counts.get(value) ?? 0,
    selected: true,
  }));
  const rest = [...counts]
    .filter(([value]) => !chosen.includes(value))
    .map(([value, count]) => ({ value, label: label(value), count, selected: false }))
    .sort((a, b) => b.count - a.count || compareValues(a.label, b.label));
  return [...first, ...rest];
}
