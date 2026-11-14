import { describe, expect, it } from "vitest";

import { finder, type Quote } from "./anchor";

function quote(text: string, before = "", after = "", at = 0): Quote {
  return { text, before, after, at };
}

describe("finder", () => {
  it("finds a quote in one item", () => {
    expect(finder([["The IC50 was 3.2 \u{b5}M."]])(quote("IC50 was 3.2 \u{3bc}M"))).toEqual({
      page: 0,
      start: { item: 0, offset: 4 },
      end: { item: 0, offset: 19 },
    });
  });

  it("finds a quote across items, line breaks, and hyphens", () => {
    const find = finder([["The inhi-", "bition of IC", "50 was clear"]]);
    expect(find(quote("inhibition of IC50"))).toEqual({
      page: 0,
      start: { item: 0, offset: 4 },
      end: { item: 2, offset: 2 },
    });
  });

  it("reads ligatures and other compatibility characters", () => {
    expect(finder([["e\u{fb03}cacy"]])(quote("efficacy"))).not.toBeNull();
  });

  it("ignores case and the form of minus signs", () => {
    expect(finder([["P = 0.34 AND \u{2212}0.5"]])(quote("p = 0.34 and -0.5"))).not.toBeNull();
  });

  it("finds nothing for an empty quote or a missing quote", () => {
    const find = finder([["abc"]]);
    expect(find(quote("   "))).toBeNull();
    expect(find(quote("abd"))).toBeNull();
  });

  it("ignores symbols, because the font of a PDF can give the wrong character for a symbol", () => {
    expect(finder([["m.p. > 300 \u{2dd}C;"]])(quote("m.p. > 300 \u{b0}C"))).toEqual({
      page: 0,
      start: { item: 0, offset: 0 },
      end: { item: 0, offset: 13 },
    });
  });

  it("counts offsets in UTF-16 code units", () => {
    expect(finder([["\u{1d6fc} is 2"]])(quote("is 2"))).toEqual({
      page: 0,
      start: { item: 0, offset: 3 },
      end: { item: 0, offset: 7 },
    });
  });

  it("gives the index of the page, and finds no quote across two pages", () => {
    const find = finder([["The first page ends"], ["and the value was 4.2"]]);
    expect(find(quote("the value was 4.2"))?.page).toBe(1);
    expect(find(quote("page ends and the"))).toBeNull();
  });

  it("chooses the place where the text around the quote agrees", () => {
    const find = finder([["5a H Me 12.3", "5b F H 8.1"]]);
    expect(find(quote("H", "12.3 5b F ", " 8.1"))).toEqual({
      page: 0,
      start: { item: 1, offset: 5 },
      end: { item: 1, offset: 6 },
    });
  });

  it("chooses the place nearest to the position of the quote when the text around agrees equally", () => {
    const find = finder([["No effect at 24 h."], ["No effect at 24 h."]]);
    expect(find(quote("No effect at 24 h", "", "", 0.1))?.page).toBe(0);
    expect(find(quote("No effect at 24 h", "", "", 0.9))?.page).toBe(1);
  });

  it("finds a short quote that occurs one time, also when the text around it does not agree", () => {
    expect(finder([["Cl and Br"]])(quote("Cl", "R2 = ", " in 5c"))).toEqual({
      page: 0,
      start: { item: 0, offset: 0 },
      end: { item: 0, offset: 2 },
    });
  });

  it("finds no place for a short quote that occurs more than one time when the text around it does not agree", () => {
    expect(finder([["H, F, and H"]])(quote("H", "R1 = ", " in 5c"))).toBeNull();
  });
});
