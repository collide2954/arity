import { describe, expect, it } from "vitest";

import { reportRows } from "./reports";
import type { Index, Report } from "./types";
import { figureRows, readFigure, rowsOfFigure, writeFigure, type FigureView } from "./values";

const index: Index = {
  format: "arity-site/1",
  software: "arity 0.1.0",
  generated_at: "2026-11-14T10:00:00Z",
  database: "arity.sqlite",
  works: [1, 2].map((id) => ({
    id,
    title: `Work ${String(id)}`,
    citation: id === 1 ? "Lafi et al. 2023, PLOS ONE" : "Keshk et al. 2025, Sci Rep",
    year: 2023,
    journal: null,
    doi: null,
    pmcid: null,
    license: null,
    shareable: true,
    is_preprint: false,
    notices: id === 2 ? ["retraction"] : [],
    reports: 1,
    state: "read",
  })),
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
    {
      id: 1,
      kind: "chemical",
      label: "doxorubicin",
      xrefs: [],
      names: [],
      current: true,
      reports: 2,
    },
    {
      id: 2,
      kind: "chemical",
      label: "hydralazine",
      xrefs: [],
      names: [],
      current: true,
      reports: 1,
    },
    { id: 3, kind: "cell_line", label: "MCF-7", xrefs: [], names: [], current: true, reports: 2 },
  ],
  former_entities: [],
};

function value(id: number, work: number, subject: "dox" | "hyd", display: number | null): Report {
  return {
    id,
    work,
    run: work,
    local_id: `r${String(id)}`,
    kind: "value",
    body: {
      type: "value",
      id: `r${String(id)}`,
      subject: [subject],
      property: "ic50",
      value: { number: String(display ?? 1), unit: "umol/L", unit_text: "\u{3bc}M" },
      conditions: [{ role: "cell_line", entity: "mcf7" }],
    },
    entities: { [subject]: subject === "dox" ? 1 : 2, mcf7: 3 },
    property: "ic50",
    direction: null,
    score: null,
    check: "unchecked",
    checks: [],
    replaced_by: null,
    warnings: [],
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

const reports = [
  value(1, 1, "dox", 0.24),
  value(2, 1, "dox", 0.24),
  value(3, 2, "dox", 9),
  value(4, 2, "hyd", 165),
  value(5, 1, "hyd", null),
];

const rows = reportRows(index, reports);

describe("readFigure and writeFigure", () => {
  function written(figure: FigureView): string {
    const params = new URLSearchParams();
    writeFigure(params, figure);
    return params.toString();
  }

  it("keep the rows and the scale of the figure in the address", () => {
    const figure: FigureView = { rows: "work", logarithmic: true };
    expect(written(figure)).toBe("rows=work&scale=log");
    expect(readFigure(new URLSearchParams("rows=work&scale=log"))).toEqual(figure);
    expect(written({ rows: null, logarithmic: false })).toBe("");
  });

  it("ignore values that are not valid", () => {
    expect(readFigure(new URLSearchParams("rows=cell&scale=square"))).toEqual({
      rows: null,
      logarithmic: false,
    });
  });
});

describe("rowsOfFigure", () => {
  it("shows one row for each report, subject, or work, so that the figure has few rows", () => {
    const automatic: FigureView = { rows: null, logarithmic: false };
    expect(rowsOfFigure(automatic, { reports: 60, subjects: 60 })).toBe("report");
    expect(rowsOfFigure(automatic, { reports: 61, subjects: 40 })).toBe("subject");
    expect(rowsOfFigure(automatic, { reports: 61, subjects: 41 })).toBe("work");
    expect(rowsOfFigure({ ...automatic, rows: "report" }, { reports: 500, subjects: 90 })).toBe(
      "report",
    );
  });
});

describe("figureRows", () => {
  it("gives each report a unique label, with the notices of its source first", () => {
    const plotted = figureRows(rows, { rows: "report", logarithmic: false });
    expect(plotted.map((row) => row.label)).toEqual([
      "doxorubicin \u{b7} cell line: MCF-7 \u{2014} Lafi et al. 2023, PLOS ONE",
      "doxorubicin \u{b7} cell line: MCF-7 \u{2014} Lafi et al. 2023, PLOS ONE (2)",
      "\u{26a0}\u{fe0e} Retraction \u{b7} doxorubicin \u{b7} cell line: MCF-7 \u{2014} Keshk et al. 2025, Sci Rep",
      "\u{26a0}\u{fe0e} Retraction \u{b7} hydralazine \u{b7} cell line: MCF-7 \u{2014} Keshk et al. 2025, Sci Rep",
    ]);
    expect(plotted.map((row) => row.slot)).toEqual([0, 0, 0, 0]);
    expect(plotted[2]).toMatchObject({
      report: 3,
      value: 9,
      origin: "stated",
      notices: "Retraction",
      marker: "\u{26a0}\u{fe0e}",
      detail: "doxorubicin \u{b7} cell line: MCF-7",
    });
  });

  it("puts the reports of a subject in one row, and gives near values separate slots", () => {
    const plotted = figureRows(rows, { rows: "subject", logarithmic: false });
    expect(plotted.map((row) => [row.report, row.label, row.slot])).toEqual([
      [1, "doxorubicin", 0],
      [2, "doxorubicin", 1],
      [3, "doxorubicin", 0],
      [4, "hydralazine", 0],
    ]);
  });

  it("puts the reports of a work in one row, with the notices of the work first", () => {
    const plotted = figureRows(rows, { rows: "work", logarithmic: true });
    expect(plotted.map((row) => [row.report, row.label, row.slot])).toEqual([
      [3, "\u{26a0}\u{fe0e} Retraction \u{b7} Keshk et al. 2025, Sci Rep", 0],
      [4, "\u{26a0}\u{fe0e} Retraction \u{b7} Keshk et al. 2025, Sci Rep", 0],
      [1, "Lafi et al. 2023, PLOS ONE", 0],
      [2, "Lafi et al. 2023, PLOS ONE", 1],
    ]);
  });
});
