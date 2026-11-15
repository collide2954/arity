import { describe, expect, it } from "vitest";

import { conditionParts, conditions, headline, names } from "./describe";
import type { Index, Report } from "./types";

const index: Index = {
  format: "arity-site/1",
  software: "arity 0.1.0",
  generated_at: "2026-11-14T10:00:00Z",
  database: "arity.sqlite",
  works: [],
  properties: [
    {
      key: "ic50",
      label: "half maximal inhibitory concentration",
      value_type: "quantity",
      unit: "umol/L",
      dimension: "L-3 N",
      categories: [],
      xrefs: [],
      description: null,
      reports: 1,
    },
    {
      key: "cell_viability",
      label: "cell viability",
      value_type: "quantity",
      unit: "%",
      dimension: "1",
      categories: [],
      xrefs: [],
      description: null,
      reports: 1,
    },
  ],
  entities: [
    {
      id: 1,
      kind: "chemical",
      label: "doxorubicin",
      xrefs: [],
      names: [],
      current: true,
      reports: 1,
    },
    { id: 2, kind: "cell_line", label: "MCF-7", xrefs: [], names: [], current: true, reports: 1 },
    {
      id: 3,
      kind: "chemical",
      label: "hydralazine",
      xrefs: [],
      names: [],
      current: true,
      reports: 1,
    },
  ],
  former_entities: [
    {
      id: 4,
      kind: "organism",
      label: "Sprague-Dawley rat",
      xrefs: ["NCBITaxon:10116"],
      names: ["Sprague-Dawley rat", "Wistar rat"],
      current: false,
      reports: 0,
    },
  ],
};

function report(body: Report["body"]): Report {
  return {
    id: 1,
    work: 1,
    run: 1,
    local_id: body.id,
    kind: body.type,
    body,
    entities: { dox: 1, mcf7: 2, hyd: 3 },
    property: null,
    direction: null,
    score: null,
    check: "unchecked",
    checks: [],
    replaced_by: null,
    warnings: [],
    evidence: [],
    display: null,
  };
}

describe("names", () => {
  it("finds the entities that only replaced runs name", () => {
    expect(names(index).entities.get(4)?.label).toBe("Sprague-Dawley rat");
  });
});

describe("headline", () => {
  it("states a value with the reported numbers", () => {
    const value = report({
      type: "value",
      id: "r1",
      subject: ["dox"],
      property: "ic50",
      value: { number: "7.30", unit: "umol/L", unit_text: "\u{3bc}M" },
    });
    expect(headline(value, names(index))).toBe(
      "half maximal inhibitory concentration of doxorubicin: 7.30 \u{3bc}M",
    );
  });

  it("states no effect as no change (0)", () => {
    const effect = report({
      type: "effect",
      id: "e1",
      agents: [{ entity: "hyd" }],
      target: { entity: "mcf7", property: "cell_viability" },
      direction: "no_change",
    });
    expect(headline(effect, names(index))).toBe(
      "hydralazine, effect on MCF-7 cell viability: no change (0)",
    );
  });
});

describe("conditionParts", () => {
  it("gives the name and the value of each condition", () => {
    const value = report({
      type: "value",
      id: "r1",
      subject: ["dox"],
      property: "ic50",
      value: { number: "7.30", unit: "umol/L" },
      conditions: [
        { role: "cell_line", entity: "mcf7" },
        { property: "cell_viability", value: { number: "48", unit: "h", unit_text: "h" } },
        { role: "medium", text: "RPMI 1640 with 10% FBS" },
        { text: "in the dark" },
      ],
    });
    expect(conditionParts(value, names(index))).toEqual([
      { name: "cell line", value: "MCF-7" },
      { name: "cell viability", value: "48 h" },
      { name: "medium", value: "RPMI 1640 with 10% FBS" },
      { name: "", value: "in the dark" },
    ]);
  });
});

describe("conditions", () => {
  it("names entities and values of the conditions", () => {
    const value = report({
      type: "value",
      id: "r1",
      subject: ["dox"],
      property: "ic50",
      value: { number: "7.30", unit: "umol/L" },
      conditions: [
        { role: "cell_line", entity: "mcf7" },
        { property: "cell_viability", value: { number: "48", unit: "h", unit_text: "h" } },
        { role: "comparator", text: "untreated cells" },
      ],
    });
    expect(conditions(value, names(index))).toEqual([
      "cell line: MCF-7",
      "cell viability 48 h",
      "comparator: untreated cells",
    ]);
  });
});
