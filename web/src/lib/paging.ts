/** The rows of one page, and the place of the page in all rows. */
export interface Page<T> {
  readonly rows: T[];
  /** The number of the page, from 1. */
  readonly page: number;
  readonly pages: number;
  /** The positions of the first and the last row of the page, from 1, or 0 when there are no rows. */
  readonly first: number;
  readonly last: number;
  readonly total: number;
}

export function pageOf<T>(rows: readonly T[], page: number, size: number): Page<T> {
  const total = rows.length;
  const pages = Math.max(1, Math.ceil(total / size));
  const current = Math.min(Math.max(1, Math.trunc(page)), pages);
  const start = (current - 1) * size;
  const end = Math.min(total, start + size);
  return {
    rows: rows.slice(start, end),
    page: current,
    pages,
    first: total === 0 ? 0 : start + 1,
    last: end,
    total,
  };
}
