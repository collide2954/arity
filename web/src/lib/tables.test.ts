import { describe, expect, it } from "vitest";

import { EFFECT_FACETS, effectItems, effectLabels, effectRows, matrixRows } from "./effects";
import { emptySelection, filter } from "./filter";
import type { Index, Report } from "./types";

const index: Index = {
  format: "arity-site/1",
  software: "arity 0.1.0",
  generated_at: "2026-11-14T10:00:00Z",
  database: "arity.sqlite",
  works: [
    {
      id: 1,
      title: "A",
      citation: "Lafi et al. 2023, PLOS ONE",
      year: 2023,
      journal: "PLOS ONE",
      doi: null,
      pmcid: "PMC1",
      license: "CC BY 4.0",
      shareable: true,
      is_preprint: false,
      notices: ["retraction"],
      reports: 3,
      state: "read",
    },
  ],
  properties: [
    {
      key: "ic50",
      label: "IC50",
      value_type: "quantity",
      unit: "umol/L",
      dimension: "L-3 N",
      categories: [],
      xrefs: [],
      description: null,
      reports: 2,
    },
  ],
  entities: [
    { id: 1, kind: "chemical", label: "doxorubicin", xrefs: [], names: [], reports: 2 },
    { id: 2, kind: "cell_line", label: "MCF-7", xrefs: [], names: [], reports: 2 },
  ],
};

const withoutNotices: Index = {
  ...index,
  works: index.works.map((work) => ({ ...work, notices: [] })),
};

function value(id: number, check: Report["check"], display: number | null): Report {
  return {
    id,
    work: 1,
    run: 1,
    local_id: `r${String(id)}`,
    kind: "value",
    body: {
      type: "value",
      id: `r${String(id)}`,
      subject: ["dox"],
      property: "ic50",
      value: { number: "0.24", unit: "umol/L", unit_text: "\u{3bc}M" },
      conditions: [{ role: "cell_line", entity: "mcf7" }],
    },
    entities: { dox: 1, mcf7: 2 },
    property: "ic50",
    direction: null,
    score: null,
    check,
    checks: [],
    evidence: [],
    display:
      display === null
        ? null
        : {
            unit: "umol/L",
            value: display,
            low: null,
            high: null,
            interval_low: null,
            interval_high: null,
          },
  };
}

function effect(id: number, direction: "increase" | "no_change", score: number): Report {
  return {
    ...value(id, "unchecked", null),
    kind: "effect",
    property: null,
    direction,
    score,
    body: {
      type: "effect",
      id: `e${String(id)}`,
      agents: [{ entity: "dox" }],
      target: { entity: "mcf7" },
      direction,
      significance: { p: "0.34" },
    },
  };
}

describe("effectRows", () => {
  it("puts reports of the same cell in separate slots", () => {
    const rows = effectRows(index, [
      effect(5, "no_change", 0),
      effect(6, "increase", 1),
      value(1, "confirmed", 1),
    ]);
    expect(rows).toHaveLength(2);
    const marks = matrixRows(rows);
    expect(marks.map((mark) => [mark.slot, mark.score])).toEqual([
      [0, "no change (0)"],
      [1, "increase (+1)"],
    ]);
    expect(rows[0]?.detail).toBe("p = 0.34");
  });

  it("gives the facets of each effect, and finds an effect by its words", () => {
    const items = effectItems(
      effectRows(index, [effect(5, "no_change", 0), effect(6, "increase", 1)]),
    );
    expect(items[0]?.values).toEqual({
      agent: ["doxorubicin"],
      target: ["MCF-7"],
      direction: ["no_change"],
      work: ["1"],
      check: ["unchecked"],
      notice: ["retraction"],
    });
    const found = filter(items, { ...emptySelection(), text: "dox increase" }, EFFECT_FACETS);
    expect(found.items.map((item) => item.report.id)).toEqual([6]);
    const label = effectLabels(index);
    expect(label("direction", "no_change")).toBe("no change");
    expect(label("work", "1")).toBe("Lafi et al. 2023, PLOS ONE");
    expect(label("agent", "doxorubicin")).toBe("doxorubicin");
  });

  it("gives each mark the notices of its source", () => {
    const rows = effectRows(index, [effect(5, "no_change", 0)]);
    expect(rows[0]?.notices).toEqual(["retraction"]);
    expect(matrixRows(rows)[0]).toMatchObject({
      notices: "Retraction",
      marker: "\u{26a0}\u{fe0e}",
    });
    const plain = matrixRows(effectRows(withoutNotices, [effect(5, "no_change", 0)]));
    expect(plain[0]).toMatchObject({ notices: "none", marker: "" });
  });
});
