import { describe, expect, it } from "vitest";

import { histogram, summary } from "./stats";

describe("summary", () => {
  it("gives the count, the extremes, the quartiles, and the median", () => {
    expect(summary([7, 1, 3, 5, 9])).toEqual({
      count: 5,
      min: 1,
      q1: 3,
      median: 5,
      q3: 7,
      max: 9,
    });
  });

  it("puts a quartile between two values", () => {
    expect(summary([1, 2, 3, 4])).toMatchObject({ q1: 1.75, median: 2.5, q3: 3.25 });
  });

  it("gives nothing for no values", () => {
    expect(summary([])).toBeNull();
  });
});

describe("histogram", () => {
  it("divides the range into bins of the same width and counts the values in each bin", () => {
    expect(histogram([0, 1, 2, 3, 4, 10], false, 5)).toEqual([
      { start: 0, end: 2, count: 2 },
      { start: 2, end: 4, count: 2 },
      { start: 4, end: 6, count: 1 },
      { start: 6, end: 8, count: 0 },
      { start: 8, end: 10, count: 1 },
    ]);
  });

  it("uses bins of the same width in powers of ten on a logarithmic scale", () => {
    const bins = histogram([1, 10, 100, 1000], true, 3);
    expect(bins.map((bin) => [bin.start, bin.end, bin.count])).toEqual([
      [1, 10, 1],
      [10, 100, 1],
      [100, 1000, 2],
    ]);
  });

  it("gives one bin when all values are the same, and no bins for no values", () => {
    expect(histogram([2, 2], false)).toEqual([{ start: 2, end: 2, count: 2 }]);
    expect(histogram([], false)).toEqual([]);
  });
});
