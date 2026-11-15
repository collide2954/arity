import { describe, expect, it } from "vitest";

import { pageOf } from "./paging";

describe("pageOf", () => {
  const rows = Array.from({ length: 120 }, (_, position) => position + 1);

  it("gives the rows of a page and the place of the page", () => {
    expect(pageOf(rows, 2, 50)).toEqual({
      rows: rows.slice(50, 100),
      page: 2,
      pages: 3,
      first: 51,
      last: 100,
      total: 120,
    });
    expect(pageOf(rows, 3, 50).rows).toEqual(rows.slice(100));
  });

  it("keeps the page number in the range of the pages", () => {
    expect(pageOf(rows, 9, 50).page).toBe(3);
    expect(pageOf(rows, 0, 50).page).toBe(1);
  });

  it("has one empty page when there are no rows", () => {
    expect(pageOf([], 1, 50)).toEqual({ rows: [], page: 1, pages: 1, first: 0, last: 0, total: 0 });
  });
});
