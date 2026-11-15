import { describe, expect, it } from "vitest";

import {
  formatNumber,
  imageReading,
  reportedValue,
  scoreLabel,
  groupsLabel,
  significanceLabel,
  uncertaintyLabel,
  unitLabel,
} from "./format";

describe("unitLabel", () => {
  it.each([
    ["umol/L", "\u{b5}mol/L"],
    ["mg/kg/d", "mg/kg/d"],
    ["Cel", "\u{b0}C"],
    ["m2", "m\u{b2}"],
    ["s-1", "s\u{207b}\u{b9}"],
    ["J/(mol.K)", "J/(mol\u{b7}K)"],
    ["10*3/uL", "10\u{b3}/\u{b5}L"],
    ["{cells}/uL", "cells/\u{b5}L"],
    ["[iU]/L", "IU/L"],
    ["kOhm", "k\u{3a9}"],
    ["1", ""],
    ["%", "%"],
  ])("writes %s as %s", (code, label) => {
    expect(unitLabel(code)).toBe(label);
  });
});

describe("formatNumber", () => {
  it.each([
    [0.24, "0.24"],
    [7.3, "7.3"],
    [1234.5678, "1235"],
    [0, "0"],
    [1.2e-9, "1.2 \u{d7} 10\u{207b}\u{2079}"],
    [6.022e23, "6.022 \u{d7} 10\u{b2}\u{b3}"],
  ])("writes %d as %s", (value, text) => {
    expect(formatNumber(value)).toBe(text);
  });
});

describe("reportedValue", () => {
  it("keeps the reported numbers and the reported unit", () => {
    expect(
      reportedValue({
        number: "7.30",
        unit: "umol/L",
        unit_text: "\u{3bc}M",
        uncertainty: { kind: "sd", number: "0.84" },
      }),
    ).toBe("7.30 \u{b1} 0.84 \u{3bc}M");
  });

  it("writes ranges, bounds, and intervals", () => {
    expect(reportedValue({ low: "3", high: "5", unit: "h", unit_text: "h" })).toBe("3\u{2013}5 h");
    expect(reportedValue({ number: "0.1", bound: "lt", unit: "mg/L" })).toBe("< 0.1 mg/L");
    expect(
      reportedValue({
        number: "1.4",
        unit: "1",
        uncertainty: { kind: "ci", low: "1.1", high: "1.9", level: "95" },
      }),
    ).toBe("1.4 (1.1\u{2013}1.9)");
  });

  it("writes boolean and category values", () => {
    expect(reportedValue({ boolean: false })).toBe("no");
    expect(reportedValue({ category: "cubic" })).toBe("cubic");
  });
});

describe("labels", () => {
  it("names the kind of uncertainty", () => {
    expect(uncertaintyLabel({ uncertainty: { kind: "sem", number: "1" } })).toBe("SEM");
    expect(
      uncertaintyLabel({ uncertainty: { kind: "ci", low: "1", high: "2", level: "95" } }),
    ).toBe("95% CI");
    expect(uncertaintyLabel({})).toBeNull();
  });

  it("shows no change as 0", () => {
    expect(scoreLabel(0)).toBe("0");
    expect(scoreLabel(1)).toBe("+1");
    expect(scoreLabel(-1)).toBe("\u{2212}1");
    expect(scoreLabel(null)).toBe("\u{b1}");
  });

  it("writes stated significance", () => {
    expect(significanceLabel({ comparison: "lt", p: "0.05" })).toBe("p < 0.05");
    expect(significanceLabel({ p: "0.34", statement: "not_significant" })).toBe(
      "p = 0.34, not significant",
    );
    expect(significanceLabel(undefined)).toBeNull();
  });

  it("gives the statistic of the test, the q-value, and the correction", () => {
    expect(
      significanceLabel({
        statement: "significant",
        comparison: "eq",
        p: "0.04",
        statistic: { kind: "t", number: "2.31", df: ["12"] },
        correction: "bonferroni",
        q: "0.08",
      }),
    ).toBe("t(12) = 2.31, p = 0.04, q = 0.08, significant, Bonferroni correction");
    expect(
      significanceLabel({ statistic: { kind: "f", number: "4.1", df: ["2", "27"] }, q: "0.01" }),
    ).toBe("F(2, 27) = 4.1, q = 0.01");
    expect(
      significanceLabel({
        statistic: { kind: "chi_squared", number: "7.2", df: ["1"] },
        p: "0.007",
      }),
    ).toBe("χ²(1) = 7.2, p = 0.007");
    expect(significanceLabel({ statistic: { kind: "u", number: "12" }, p: "0.02" })).toBe(
      "U = 12, p = 0.02",
    );
  });
});

describe("groupsLabel", () => {
  it("names each group with its number of samples", () => {
    expect(
      groupsLabel([
        { label: "treated cells", n: 8 },
        { label: "control cells", n: 6 },
      ]),
    ).toBe("treated cells (n = 8), control cells (n = 6)");
    expect(groupsLabel(undefined)).toBeNull();
    expect(groupsLabel([])).toBeNull();
  });
});

describe("imageReading", () => {
  it("tells whether the reader read a value from a plot or from a table image", () => {
    const reading = { method: "transcribed", resolution: "0.01" };
    expect(imageReading({ number: "1" })).toBeNull();
    expect(
      imageReading({ number: "1", read_from_figure: { ...reading, figure: "f2/caption" } })?.label,
    ).toBe("Read from figure");
    expect(
      imageReading({ number: "1", read_from_figure: { ...reading, figure: "t1/caption" } })?.label,
    ).toBe("Read from table image");
  });
});
