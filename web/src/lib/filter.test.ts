import { describe, expect, it } from "vitest";

import {
  emptySelection,
  filter,
  isEmpty,
  options,
  readSelection,
  toggle,
  writeSelection,
  type Item,
  type Selection,
} from "./filter";
import { searchForm } from "./search";

const FACETS = ["color", "size"] as const;
type Facet = (typeof FACETS)[number];

const items: Item<Facet>[] = [
  { text: searchForm("Red apple"), values: { color: ["red"], size: ["small"] }, number: 3 },
  { text: searchForm("Green apple"), values: { color: ["green"], size: ["small"] }, number: 5 },
  { text: searchForm("Red melon"), values: { color: ["red"], size: ["large"] }, number: null },
  { text: searchForm("Red and green pepper"), values: { color: ["red", "green"] }, number: 1 },
];

function selection(parts: Partial<Selection<Facet>>): Selection<Facet> {
  return { ...emptySelection<Facet>(), ...parts };
}

function written(chosen: Selection<Facet>): string {
  const params = new URLSearchParams();
  writeSelection(params, chosen, FACETS);
  return params.toString();
}

describe("readSelection and writeSelection", () => {
  it("keep a selection in the address", () => {
    const chosen = selection({
      text: "red apple",
      facets: { color: ["red", "green"], size: ["small"] },
      min: 1,
      max: 4.5,
    });
    const text = written(chosen);
    expect(text).toBe("q=red+apple&color=red&color=green&size=small&min=1&max=4.5");
    expect(readSelection(new URLSearchParams(text), FACETS)).toEqual(chosen);
  });

  it("ignore other names, empty values, repeated values, and numbers that are not valid", () => {
    expect(
      readSelection(
        new URLSearchParams("shape=round&color=&color=red&color=red&min=x&max=4"),
        FACETS,
      ),
    ).toEqual(selection({ facets: { color: ["red"] }, max: 4 }));
  });

  it("write nothing for an empty selection", () => {
    expect(written(emptySelection())).toBe("");
    expect(isEmpty(emptySelection())).toBe(true);
    expect(isEmpty(selection({ min: 0 }))).toBe(false);
  });
});

describe("toggle", () => {
  it("adds a value that is not in the facet, and removes a value that is", () => {
    const red = toggle(emptySelection<Facet>(), "color", "red");
    expect(red.facets).toEqual({ color: ["red"] });
    expect(toggle(red, "color", "red").facets).toEqual({});
  });
});

describe("filter", () => {
  it("needs each term, one value of each chosen facet, and a number in the range", () => {
    const kept = (chosen: Selection<Facet>) =>
      filter(items, chosen, FACETS).items.map((item) => item.text);
    expect(kept(selection({ text: "apple" }))).toEqual(["red apple", "green apple"]);
    expect(kept(selection({ facets: { color: ["red", "green"], size: ["small"] } }))).toEqual([
      "red apple",
      "green apple",
    ]);
    expect(kept(selection({ min: 2 }))).toEqual(["red apple", "green apple"]);
    expect(kept(selection({ max: 3 }))).toEqual(["red apple", "red and green pepper"]);
  });

  it("counts the values of each facet with the other parts of the selection", () => {
    const { counts } = filter(items, selection({ facets: { color: ["green"] } }), FACETS);
    expect(counts.get("color")).toEqual(
      new Map([
        ["red", 3],
        ["green", 2],
      ]),
    );
    expect(counts.get("size")).toEqual(new Map([["small", 1]]));
  });
});

describe("options", () => {
  it("puts the chosen values first, then the values by count and by label", () => {
    const counts = new Map([
      ["b", 2],
      ["a", 2],
      ["c", 5],
    ]);
    const chosen = selection({ facets: { color: ["z", "b"] } });
    expect(
      options("color", counts, chosen, (value) => value.toUpperCase()).map((option) => [
        option.label,
        option.count,
        option.selected,
      ]),
    ).toEqual([
      ["Z", 0, true],
      ["B", 2, true],
      ["C", 5, false],
      ["A", 2, false],
    ]);
  });
});
