import { describe, expect, it } from "vitest";

import { filter, emptySelection } from "./filter";
import { facetLabels, REPORT_FACETS, reportRows, reportsCsv, reportsJson } from "./reports";
import type { Index, Report } from "./types";

const index: Index = {
  format: "arity-site/1",
  software: "arity 0.1.0",
  generated_at: "2026-11-14T10:00:00Z",
  database: "arity.sqlite",
  works: [
    {
      id: 1,
      title: "Doxorubicin in breast cancer cells",
      citation: "Lafi et al. 2023, PLOS ONE",
      year: 2023,
      journal: "PLOS ONE",
      doi: "10.1371/journal.pone.0291981",
      pmcid: "PMC10538757",
      license: "CC BY 4.0",
      shareable: true,
      is_preprint: false,
      notices: [],
      reports: 1,
      state: "read",
    },
    {
      id: 2,
      title: "A retracted work",
      citation: "Keshk et al. 2025, Scientific Reports",
      year: 2025,
      journal: "Scientific Reports",
      doi: "10.1038/s41598-025-92066-w",
      pmcid: null,
      license: "CC BY 4.0",
      shareable: true,
      is_preprint: false,
      notices: ["retraction"],
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
      reports: 2,
    },
  ],
  entities: [
    {
      id: 7,
      kind: "chemical",
      label: "doxorubicin",
      xrefs: ["CHEBI:28748"],
      names: ["DOX"],
      reports: 2,
    },
    { id: 8, kind: "cell_line", label: "MCF-7", xrefs: [], names: [], reports: 2 },
  ],
};

const value: Report = {
  id: 10,
  work: 1,
  run: 1,
  local_id: "dox_ic50",
  kind: "value",
  body: {
    type: "value",
    id: "dox_ic50",
    subject: ["dox"],
    property: "ic50",
    value: {
      number: "0.24",
      unit: "umol/L",
      unit_text: "\u{3bc}M",
      read_from_figure: { figure: "f2", method: "pixel reading", resolution: "0.01" },
    },
    conditions: [{ role: "cell_line", entity: "mcf7" }],
    method: { label: "MTT assay" },
  },
  entities: { dox: 7, mcf7: 8 },
  property: "ic50",
  direction: null,
  score: null,
  check: "unchecked",
  checks: [],
  evidence: [{ block: "s3/p2", quote: "The IC50 of DOX was 0.24 \u{3bc}M", start: 0, end: 31 }],
  display: {
    unit: "umol/L",
    value: 0.24,
    low: null,
    high: null,
    interval_low: null,
    interval_high: null,
  },
};

const effect: Report = {
  ...value,
  id: 11,
  work: 2,
  local_id: "dox_vs_control",
  kind: "effect",
  body: {
    type: "effect",
    id: "dox_vs_control",
    agents: [{ entity: "dox" }],
    target: { entity: "mcf7", property: "ic50" },
    direction: "decrease",
  },
  direction: "decrease",
  score: -1,
  check: "confirmed",
  evidence: [],
  display: null,
};

describe("reportRows", () => {
  const [valueRow, effectRow] = reportRows(index, [value, effect]);

  it("gives the texts of a value for the tables", () => {
    expect(valueRow).toMatchObject({
      about: "half maximal inhibitory concentration",
      subject: "doxorubicin",
      reported: "0.24 \u{3bc}M",
      converted: 0.24,
      unit: "\u{b5}mol/L",
      conditions: ["cell line: MCF-7"],
      method: "MTT assay",
      citation: "Lafi et al. 2023, PLOS ONE",
      identifier: "PMC10538757",
      year: 2023,
      notices: [],
      quote: "The IC50 of DOX was 0.24 \u{3bc}M",
      number: 0.24,
    });
  });

  it("gives the texts of an effect for the tables", () => {
    expect(effectRow).toMatchObject({
      about: "effect on half maximal inhibitory concentration",
      subject: "doxorubicin \u{2192} MCF-7",
      reported: "decrease (\u{2212}1)",
      converted: null,
      identifier: "10.1038/s41598-025-92066-w",
      notices: ["retraction"],
      quote: "",
      number: null,
    });
  });

  it("gives the values that the facets test", () => {
    expect(valueRow?.values).toEqual({
      property: ["ic50"],
      entity: ["7", "8"],
      condition: ["cell line: MCF-7"],
      work: ["1"],
      kind: ["value"],
      check: ["unchecked"],
      notice: ["none"],
      origin: ["image"],
      direction: [],
    });
    expect(effectRow?.values).toMatchObject({
      notice: ["retraction"],
      origin: [],
      direction: ["decrease"],
    });
  });

  it("finds a report by its quote, by a name of an entity, and by an identifier", () => {
    const rows = reportRows(index, [value, effect]);
    const found = (text: string) =>
      filter(rows, { ...emptySelection(), text }, REPORT_FACETS).items.map((row) => row.report.id);
    expect(found("dox was 0.24")).toEqual([10]);
    expect(found("chebi:28748 mcf-7")).toEqual([10, 11]);
    expect(found("PMC10538757")).toEqual([10]);
  });
});

describe("reportsCsv and reportsJson", () => {
  it("write each row with all columns, and each report as arity exports it", () => {
    const rows = reportRows(index, [value]);
    const [header, line] = reportsCsv(rows).split("\r\n");
    expect(header?.split(",").slice(0, 4)).toEqual([
      "Property or effect",
      "Subject",
      "As reported",
      "Converted",
    ]);
    expect(line?.split(",").slice(0, 4)).toEqual([
      "half maximal inhibitory concentration",
      "doxorubicin",
      "0.24 \u{3bc}M",
      "0.24",
    ]);
    expect(JSON.parse(reportsJson(rows))).toEqual([value]);
  });
});

describe("facetLabels", () => {
  it("names each value of each facet", () => {
    const label = facetLabels(index);
    expect(label("property", "ic50")).toBe("half maximal inhibitory concentration");
    expect(label("entity", "8")).toBe("MCF-7 (cell line)");
    expect(label("entity", "99")).toBe("entity 99");
    expect(label("work", "2")).toBe("Keshk et al. 2025, Scientific Reports");
    expect(label("kind", "effect")).toBe("Effect");
    expect(label("check", "unchecked")).toBe("Not checked");
    expect(label("notice", "none")).toBe("No notice");
    expect(label("notice", "retraction")).toBe("Retraction");
    expect(label("origin", "image")).toBe("Read from an image");
    expect(label("direction", "no_change")).toBe("no change");
    expect(label("condition", "cell line: MCF-7")).toBe("cell line: MCF-7");
  });
});
