import { describe, expect, it } from "vitest";

import { catalogueTargets, findTargets, reportTargets } from "./jump";
import type { Index, Report } from "./types";

const index: Index = {
  format: "arity-site/1",
  software: "arity 0.1.0",
  generated_at: "2026-11-14T10:00:00Z",
  database: "arity.sqlite",
  works: [
    {
      id: 1,
      title: "Doxorubicin with hydralazine against MCF-7 cells",
      citation: "Lafi et al. 2023, PLOS ONE",
      year: 2023,
      journal: "PLOS ONE",
      doi: null,
      pmcid: "PMC10538757",
      license: null,
      shareable: true,
      is_preprint: false,
      notices: [],
      reports: 1,
      state: "read",
    },
  ],
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
  ],
  entities: [
    {
      id: 7,
      kind: "chemical",
      label: "doxorubicin",
      xrefs: [],
      names: ["DOX"],
      current: true,
      reports: 1,
    },
  ],
  former_entities: [],
};

const report: Report = {
  id: 40,
  work: 1,
  run: 1,
  local_id: "dox_ic50",
  kind: "value",
  body: {
    type: "value",
    id: "dox_ic50",
    subject: ["dox"],
    property: "ic50",
    value: { number: "0.24", unit: "umol/L", unit_text: "\u{3bc}M" },
  },
  entities: { dox: 7 },
  property: "ic50",
  direction: null,
  score: null,
  check: "unchecked",
  checks: [],
  replaced_by: null,
  evidence: [],
  display: null,
};

const targets = [...catalogueTargets(index), ...reportTargets(index, [report])];
const labels = (search: string, limit = 10) =>
  findTargets(targets, search, limit).map((target) => `${target.kind}: ${target.label}`);

describe("findTargets", () => {
  it("finds pages, properties, entities, works, and reports by each word", () => {
    expect(labels("ic50")).toEqual([
      "property: half maximal inhibitory concentration",
      "report: half maximal inhibitory concentration of doxorubicin: 0.24 \u{3bc}M",
    ]);
    expect(labels("effects")).toEqual(["page: Effects"]);
    expect(labels("pmc10538757")).toEqual([
      "work: Doxorubicin with hydralazine against MCF-7 cells",
      "report: half maximal inhibitory concentration of doxorubicin: 0.24 \u{3bc}M",
    ]);
  });

  it("puts the targets whose name starts with the search first", () => {
    expect(labels("doxorubicin")).toEqual([
      "entity: doxorubicin",
      "work: Doxorubicin with hydralazine against MCF-7 cells",
      "report: half maximal inhibitory concentration of doxorubicin: 0.24 \u{3bc}M",
    ]);
  });

  it("finds a report by its number, and gives no more targets than the limit", () => {
    expect(labels("#40")).toEqual([
      "report: half maximal inhibitory concentration of doxorubicin: 0.24 \u{3bc}M",
    ]);
    expect(labels("doxorubicin", 1)).toEqual(["entity: doxorubicin"]);
    expect(labels(" ")).toEqual([]);
  });
});
