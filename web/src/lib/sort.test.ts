import { describe, expect, it } from "vitest";

import { compareValues, sortBy } from "./sort";

describe("compareValues", () => {
  it("puts text in natural order, without regard to case", () => {
    expect(["10a", "3b", "ab", "3a", "Ac"].sort(compareValues)).toEqual([
      "3a",
      "3b",
      "10a",
      "ab",
      "Ac",
    ]);
  });

  it("puts numbers by size, before text", () => {
    expect([10, "x", 2, -1].sort(compareValues)).toEqual([-1, 2, 10, "x"]);
  });
});

describe("sortBy", () => {
  const rows = [
    { name: "b", value: 2 },
    { name: "none", value: null },
    { name: "a", value: 1 },
    { name: "c", value: 2 },
  ];

  it("keeps the order of equal rows and puts rows without a value last", () => {
    const names = (order: "ascending" | "descending") =>
      sortBy(rows, (row) => row.value, order).map((row) => row.name);
    expect(names("ascending")).toEqual(["a", "b", "c", "none"]);
    expect(names("descending")).toEqual(["b", "c", "a", "none"]);
  });

  it("does not change the rows that it sorts", () => {
    sortBy(rows, (row) => row.value, "ascending");
    expect(rows.map((row) => row.name)).toEqual(["b", "none", "a", "c"]);
  });
});
