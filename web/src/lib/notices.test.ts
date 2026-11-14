import { describe, expect, it } from "vitest";

import {
  isSerious,
  noticeLabel,
  noticeMarker,
  noticesOfWorks,
  seriousFirst,
  sourceLabel,
} from "./notices";

describe("noticeLabel", () => {
  it("names each kind, and shows an unknown kind as its key", () => {
    expect(noticeLabel("retraction")).toBe("Retraction");
    expect(noticeLabel("expression_of_concern")).toBe("Expression of concern");
    expect(noticeLabel("other")).toBe("Other notice");
    expect(noticeLabel("future_kind")).toBe("future kind");
  });
});

describe("sourceLabel", () => {
  it("names each source", () => {
    expect(sourceLabel("pmc")).toBe("PubMed Central");
    expect(sourceLabel("retraction_watch")).toBe("Retraction Watch");
    expect(sourceLabel("new_source")).toBe("new source");
  });
});

describe("isSerious", () => {
  it("is true only for the kinds that withdraw all or part of a work", () => {
    expect(["retraction", "partial_retraction", "withdrawal", "removal"].every(isSerious)).toBe(
      true,
    );
    expect(["correction", "erratum", "expression_of_concern"].some(isSerious)).toBe(false);
  });
});

describe("seriousFirst", () => {
  it("puts the serious kinds first and keeps the order of the others", () => {
    expect(seriousFirst(["correction", "erratum", "retraction"])).toEqual([
      "retraction",
      "correction",
      "erratum",
    ]);
  });
});

describe("noticeMarker", () => {
  it("is empty for a work without notices", () => {
    expect(noticeMarker([])).toBe("");
  });

  it("names the kinds after a sign for the most serious kind", () => {
    expect(noticeMarker(["correction"])).toBe("\u{2139}\u{fe0e} Correction");
    expect(noticeMarker(["correction", "retraction"])).toBe(
      "\u{26a0}\u{fe0e} Retraction, Correction",
    );
  });
});

describe("noticesOfWorks", () => {
  it("finds the notice kinds of each work, and none for an unknown work", () => {
    const notices = noticesOfWorks([
      { id: 1, notices: ["retraction"] },
      { id: 2, notices: [] },
    ]);
    expect(notices(1)).toEqual(["retraction"]);
    expect(notices(2)).toEqual([]);
    expect(notices(3)).toEqual([]);
  });
});
