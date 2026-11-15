import { describe, expect, it } from "vitest";

import { matchesAll, searchForm, terms } from "./search";

describe("searchForm", () => {
  it("ignores case, accents, and the forms of a character", () => {
    expect(searchForm("D\u{f3}xorubicin  IC\u{2085}\u{2080}")).toBe("doxorubicin ic50");
    expect(searchForm("5 \u{b5}M")).toBe(searchForm("5 \u{3bc}M"));
  });

  it("writes each dash, quote, and space in one form", () => {
    expect(searchForm("\u{2018}MCF\u{2013}7\u{2019}\u{a0}\u{a0}cells\n")).toBe("'mcf-7' cells");
  });
});

describe("terms", () => {
  it("splits the words and keeps a phrase in quotes together", () => {
    expect(terms('IC50 "cell  viability" MCF-7')).toEqual(["ic50", "cell viability", "mcf-7"]);
  });

  it("accepts curved quotes and a phrase without its last quote", () => {
    expect(terms("\u{201c}cell viability\u{201d} 48 h")).toEqual(["cell viability", "48", "h"]);
    expect(terms('dose "48 h')).toEqual(["dose", "48 h"]);
  });

  it("gives no terms for a search without words", () => {
    expect(terms('  "" ')).toEqual([]);
  });
});

describe("matchesAll", () => {
  it("needs each term", () => {
    const text = searchForm("IC50 of doxorubicin in MCF-7 cells");
    expect(matchesAll(text, terms("mcf dox"))).toBe(true);
    expect(matchesAll(text, terms("mcf hela"))).toBe(false);
    expect(matchesAll(text, [])).toBe(true);
  });
});
