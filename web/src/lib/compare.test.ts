import { describe, expect, it } from "vitest";

import {
  dimensions,
  dimensionValue,
  matrixCsv,
  matrixOf,
  NOT_STATED,
  readCompare,
  writeCompare,
  type Compare,
} from "./compare";
import { names } from "./describe";
import { reportRows } from "./reports";
import type { Condition, Index, Report } from "./types";

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
      journal: null,
      doi: null,
      pmcid: null,
      license: null,
      shareable: true,
      is_preprint: false,
      notices: [],
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
      reports: 3,
    },
    {
      key: "exposure_time",
      label: "exposure time",
      value_type: "quantity",
      unit: "h",
      dimension: "T",
      categories: [],
      xrefs: [],
      description: null,
      reports: 0,
    },
  ],
  entities: [
    { id: 1, kind: "chemical", label: "doxorubicin", xrefs: [], names: [], reports: 3 },
    { id: 2, kind: "cell_line", label: "MCF-7", xrefs: [], names: [], reports: 2 },
    { id: 3, kind: "cell_line", label: "HeLa", xrefs: [], names: [], reports: 1 },
  ],
};

function value(id: number, number: string, conditions: Condition[]): Report {
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
      value: { number, unit: "umol/L", unit_text: "\u{3bc}M" },
      conditions,
      method: { label: "MTT assay" },
    },
    entities: { dox: 1, mcf7: 2, hela: 3 },
    property: "ic50",
    direction: null,
    score: null,
    check: "unchecked",
    checks: [],
    evidence: [],
    display: null,
  };
}

const hours = (number: string): Condition => ({
  property: "exposure_time",
  value: { number, unit: "h", unit_text: "h" },
});

const rows = reportRows(index, [
  value(1, "0.24", [{ role: "cell_line", entity: "mcf7" }, hours("48")]),
  value(2, "0.31", [{ role: "cell_line", entity: "mcf7" }, hours("48")]),
  value(3, "1.2", [{ role: "cell_line", entity: "hela" }]),
]);
const lookup = names(index);

describe("dimensions", () => {
  it("gives the subject, the work, the method, and each role and property of the conditions", () => {
    expect(dimensions(rows, lookup)).toEqual([
      { key: "subject", label: "Subject" },
      { key: "work", label: "Work" },
      { key: "method", label: "Method" },
      { key: "role:cell_line", label: "cell line" },
      { key: "condition:exposure_time", label: "exposure time" },
    ]);
  });
});

describe("dimensionValue", () => {
  it("gives the value of a report for a dimension, or not stated", () => {
    const [first, , third] = rows;
    expect(first && dimensionValue(first, "role:cell_line", lookup)).toBe("MCF-7");
    expect(first && dimensionValue(first, "condition:exposure_time", lookup)).toBe("48 h");
    expect(third && dimensionValue(third, "condition:exposure_time", lookup)).toBe(NOT_STATED);
    expect(third && dimensionValue(third, "work", lookup)).toBe("Lafi et al. 2023, PLOS ONE");
  });
});

describe("matrixOf", () => {
  it("puts each report in the cell of its row and its column, without combining reports", () => {
    const matrix = matrixOf(rows, "role:cell_line", "condition:exposure_time", lookup);
    expect(matrix.rows).toEqual(["HeLa", "MCF-7"]);
    expect(matrix.columns).toEqual(["48 h", NOT_STATED]);
    expect(
      matrix.cells
        .get("MCF-7")
        ?.get("48 h")
        ?.map((row) => row.report.id),
    ).toEqual([1, 2]);
    expect(
      matrix.cells
        .get("HeLa")
        ?.get(NOT_STATED)
        ?.map((row) => row.report.id),
    ).toEqual([3]);
    expect(matrix.cells.get("HeLa")?.get("48 h")).toBeUndefined();
  });

  it("writes one line of CSV for each report", () => {
    const matrix = matrixOf(rows, "role:cell_line", "condition:exposure_time", lookup);
    expect(matrixCsv(matrix, "cell line", "exposure time").split("\r\n").slice(0, 3)).toEqual([
      "cell line,exposure time,report,as_reported,converted,unit,source,check",
      'HeLa,not stated,3,1.2 \u{3bc}M,,,"Lafi et al. 2023, PLOS ONE",unchecked',
      'MCF-7,48 h,1,0.24 \u{3bc}M,,,"Lafi et al. 2023, PLOS ONE",unchecked',
    ]);
  });
});

describe("readCompare and writeCompare", () => {
  it("keep the property, the dimensions, and the kind of value in the address", () => {
    const compare: Compare = {
      property: "ic50",
      rows: "role:cell_line",
      columns: "work",
      converted: true,
    };
    const params = new URLSearchParams();
    writeCompare(params, compare);
    expect(params.toString()).toBe(
      "property=ic50&by=role%3Acell_line&across=work&values=converted",
    );
    expect(readCompare(params)).toEqual(compare);
    expect(readCompare(new URLSearchParams(""))).toEqual({
      property: null,
      rows: "subject",
      columns: "work",
      converted: false,
    });
  });
});
