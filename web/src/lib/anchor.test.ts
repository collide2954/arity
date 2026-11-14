import { describe, expect, it } from "vitest";

import { anchor } from "./anchor";

describe("anchor", () => {
  it("finds a quote in one item", () => {
    expect(anchor(["The IC50 was 3.2 \u{b5}M."], "IC50 was 3.2 \u{3bc}M")).toEqual([
      { start: { item: 0, offset: 4 }, end: { item: 0, offset: 19 } },
    ]);
  });

  it("finds a quote across items, line breaks, and hyphens", () => {
    const items = ["The inhi-", "bition of IC", "50 was clear"];
    expect(anchor(items, "inhibition of IC50")).toEqual([
      { start: { item: 0, offset: 4 }, end: { item: 2, offset: 2 } },
    ]);
  });

  it("reads ligatures and other compatibility characters", () => {
    expect(anchor(["e\u{fb03}cacy"], "efficacy")).toHaveLength(1);
  });

  it("ignores case and the form of minus signs", () => {
    expect(anchor(["P = 0.34 AND \u{2212}0.5"], "p = 0.34 and -0.5")).toHaveLength(1);
  });

  it("gives every place where the quote occurs", () => {
    expect(anchor(["no effect; no effect"], "no effect")).toHaveLength(2);
  });

  it("finds nothing for an empty quote or a missing quote", () => {
    expect(anchor(["abc"], "   ")).toEqual([]);
    expect(anchor(["abc"], "abd")).toEqual([]);
  });

  it("counts offsets in UTF-16 code units", () => {
    expect(anchor(["\u{1d6fc} = 2"], "= 2")).toEqual([
      { start: { item: 0, offset: 3 }, end: { item: 0, offset: 6 } },
    ]);
  });
});
