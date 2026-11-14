import { describe, expect, it } from "vitest";

import { segments } from "./highlight";

describe("segments", () => {
  it("marks the quoted part of a block", () => {
    expect(segments("The IC50 was 0.24 \u{3bc}M.", [{ start: 13, end: 20, report: 7 }])).toEqual([
      { text: "The IC50 was ", reports: [] },
      { text: "0.24 \u{3bc}M", reports: [7] },
      { text: ".", reports: [] },
    ]);
  });

  it("marks the numbers that no report quotes", () => {
    expect(
      segments(
        "IC50 = 0.24 and 0.30 \u{3bc}M",
        [{ start: 7, end: 11, report: 1 }],
        [{ start: 16, end: 20 }],
      ),
    ).toEqual([
      { text: "IC50 = ", reports: [] },
      { text: "0.24", reports: [1] },
      { text: " and ", reports: [] },
      { text: "0.30", reports: [], unquoted: true },
      { text: " \u{3bc}M", reports: [] },
    ]);
  });

  it("marks parts that more than one report quotes", () => {
    expect(
      segments("abcdef", [
        { start: 0, end: 4, report: 1 },
        { start: 2, end: 6, report: 2 },
      ]),
    ).toEqual([
      { text: "ab", reports: [1] },
      { text: "cd", reports: [1, 2] },
      { text: "ef", reports: [2] },
    ]);
  });

  it("counts code points, not UTF-16 units", () => {
    expect(segments("\u{1d6fc}=2", [{ start: 1, end: 3, report: 1 }])).toEqual([
      { text: "\u{1d6fc}", reports: [] },
      { text: "=2", reports: [1] },
    ]);
  });

  it("keeps ranges inside the text", () => {
    expect(segments("ab", [{ start: 1, end: 9, report: 1 }])).toEqual([
      { text: "a", reports: [] },
      { text: "b", reports: [1] },
    ]);
    expect(segments("", [])).toEqual([]);
  });
});
