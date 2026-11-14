import { describe, expect, it } from "vitest";

import { coverageLine, unquotedOf } from "./coverage";
import type { Coverage } from "./types";

const coverage: Coverage = {
  cells: { numbers: 340, quoted: 6, skipped: 0 },
  text: { numbers: 127, quoted: 6, skipped: 4 },
  unquoted: [
    { block: "t1/r3c5", number: "19.53", start: 0, end: 5 },
    { block: "t1/r3c5", number: "1.05", start: 8, end: 12 },
    { block: "s2/p1", number: "37", start: 40, end: 42 },
  ],
};

describe("coverageLine", () => {
  it("tells how many numbers the reports quote and the readers skipped", () => {
    expect(coverageLine(coverage)).toBe(
      "Reports quote 6 of 340 numbers in table cells, and 6 of 127 numbers in the text (4 skipped).",
    );
  });
});

describe("unquotedOf", () => {
  it("gives the unquoted ranges of one block", () => {
    expect(unquotedOf(coverage, "t1/r3c5")).toEqual([
      { start: 0, end: 5 },
      { start: 8, end: 12 },
    ]);
    expect(unquotedOf(null, "t1/r3c5")).toEqual([]);
  });
});
