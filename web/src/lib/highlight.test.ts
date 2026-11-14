import { describe, expect, it } from "vitest";

import type { Match } from "./anchor";
import {
  byBlock,
  labelsOf,
  lines,
  mainMark,
  place,
  quotedParts,
  segments,
  type Box,
  type Highlight,
  type Quoted,
} from "./highlight";
import type { Block } from "./types";

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

function highlight(
  report: number,
  index: number,
  quote: string,
  block: string,
  start: number,
  end: number,
): Highlight {
  return { report, index, block, quote, start, end, label: `report ${String(report)}` };
}

describe("byBlock", () => {
  it("gives the highlights of each block, in their order", () => {
    const first = highlight(1, 0, "a", "s1/p1", 0, 1);
    const second = highlight(2, 0, "b", "s1/p2", 0, 1);
    const third = highlight(2, 1, "c", "s1/p1", 2, 3);
    expect(byBlock([first, second, third])).toEqual(
      new Map([
        ["s1/p1", [first, third]],
        ["s1/p2", [second]],
      ]),
    );
  });
});

describe("labelsOf", () => {
  it("gives the label of each report", () => {
    expect(
      labelsOf([
        highlight(1, 0, "a", "s1/p1", 0, 1),
        highlight(1, 1, "b", "s1/p1", 2, 3),
        highlight(2, 0, "c", "s1/p2", 0, 1),
      ]),
    ).toEqual(
      new Map([
        [1, "report 1"],
        [2, "report 2"],
      ]),
    );
  });
});

function block(key: string, text: string): Block {
  return { key, kind: "paragraph", text };
}

describe("quotedParts", () => {
  it("groups the reports that quote one part of the document, with the text around the part", () => {
    const blocks = [
      block("s1/p1", "The IC50 was 0.24 \u{3bc}M."),
      block("s1/p2", "It was 0.30 \u{3bc}M in MCF-7."),
    ];
    const parts = quotedParts(blocks, [
      highlight(1, 0, "0.24", "s1/p1", 13, 17),
      highlight(2, 1, "0.24", "s1/p1", 13, 17),
      highlight(2, 0, "0.30", "s1/p2", 7, 11),
    ]);
    expect(parts).toEqual([
      {
        quote: {
          text: "0.24",
          before: "The IC50 was ",
          after: " \u{3bc}M. It was 0.30 \u{3bc}M in MCF-7.",
          at: 13 / 46,
        },
        reports: [1, 2],
        main: [1],
      },
      {
        quote: {
          text: "0.30",
          before: "The IC50 was 0.24 \u{3bc}M. It was ",
          after: " \u{3bc}M in MCF-7.",
          at: 29 / 46,
        },
        reports: [2],
        main: [2],
      },
    ]);
  });

  it("counts the range of a quote in code points", () => {
    const [part] = quotedParts(
      [block("s1/p1", "\u{1d6fc} = 2 and \u{1d6fd} = 3")],
      [highlight(1, 0, "\u{1d6fd} = 3", "s1/p1", 10, 15)],
    );
    expect(part?.quote).toEqual({
      text: "\u{1d6fd} = 3",
      before: "\u{1d6fc} = 2 and ",
      after: "",
      at: 11 / 17,
    });
  });

  it("gives no text around a quote whose block is not in the document", () => {
    const [part] = quotedParts([], [highlight(1, 0, "0.24", "s1/p1", 13, 17)]);
    expect(part?.quote).toEqual({ text: "0.24", before: "", after: "", at: 0 });
  });
});

describe("place", () => {
  function match(page: number, offset: number): Match {
    return { page, start: { item: 0, offset }, end: { item: 0, offset: offset + 1 } };
  }

  function part(text: string, reports: number[], main: number[]): Quoted {
    return { quote: { text, before: "", after: "", at: 0 }, reports, main };
  }

  it("gives one mark for each place in the PDF, and the page of the main mark of each report", () => {
    const found = new Map([
      ["a", match(0, 0)],
      ["b", match(1, 5)],
      ["c", match(0, 0)],
    ]);
    const placed = place(
      [part("a", [1, 2], [1]), part("b", [2], [2]), part("c", [3], [3]), part("d", [4], [4])],
      (quote) => found.get(quote.text) ?? null,
    );
    expect(placed.marks).toEqual(
      new Map([
        [0, [{ match: match(0, 0), reports: [1, 2, 3], main: [1, 3] }]],
        [1, [{ match: match(1, 5), reports: [2], main: [2] }]],
      ]),
    );
    expect(placed.pages).toEqual(
      new Map([
        [1, 0],
        [2, 1],
        [3, 0],
      ]),
    );
  });
});

describe("lines", () => {
  const box = (x: number, y: number, width: number, height: number): Box => ({
    x,
    y,
    width,
    height,
  });

  it("joins the boxes of one line, also a box that is lower, as a subscript is", () => {
    expect(lines([box(10, 100, 30, 16), box(43, 100, 20, 16), box(63, 108, 8, 10)])).toEqual([
      box(10, 100, 61, 18),
    ]);
  });

  it("joins a box with a box at the same place", () => {
    expect(lines([box(10, 100, 30, 16), box(10, 99, 30, 18)])).toEqual([box(10, 99, 30, 18)]);
  });

  it("starts a new box on the next line", () => {
    expect(lines([box(300, 100, 50, 16), box(40, 120, 60, 16)])).toEqual([
      box(300, 100, 50, 16),
      box(40, 120, 60, 16),
    ]);
  });

  it("starts a new box after a gap that is wider than the height of the text", () => {
    expect(lines([box(10, 100, 30, 16), box(80, 100, 20, 16)])).toEqual([
      box(10, 100, 30, 16),
      box(80, 100, 20, 16),
    ]);
  });
});

describe("mainMark", () => {
  it("finds the mark of the main quote of a report, or else its first mark", () => {
    const container = document.createElement("div");
    container.innerHTML =
      '<button class="mark" data-reports="1 2" data-main="1"></button>' +
      '<button class="mark" data-reports="2 3" data-main="3"></button>';
    const [first, second] = [...container.children];
    expect(mainMark(container, 1)).toBe(first);
    expect(mainMark(container, 3)).toBe(second);
    expect(mainMark(container, 2)).toBe(first);
    expect(mainMark(container, 4)).toBeNull();
  });
});
