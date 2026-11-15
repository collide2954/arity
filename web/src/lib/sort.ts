export type SortValue = string | number | null;
export type Order = "ascending" | "descending";

const COLLATOR = new Intl.Collator("en", { numeric: true, sensitivity: "base" });

/** Numbers by size, before text in natural order: "3b" goes before "10a". */
export function compareValues(a: string | number, b: string | number): number {
  if (typeof a === "number" && typeof b === "number") return a - b;
  if (typeof a === "number") return -1;
  if (typeof b === "number") return 1;
  return COLLATOR.compare(a, b);
}

/** A sorted copy of the rows. Equal rows keep their order, and rows without a value go last. */
export function sortBy<T>(rows: readonly T[], key: (row: T) => SortValue, order: Order): T[] {
  const sign = order === "ascending" ? 1 : -1;
  return rows
    .map((row, position) => ({ row, position, value: key(row) }))
    .sort((a, b) => {
      if (a.value === null || b.value === null) {
        if (a.value === b.value) return a.position - b.position;
        return a.value === null ? 1 : -1;
      }
      return sign * compareValues(a.value, b.value) || a.position - b.position;
    })
    .map(({ row }) => row);
}
