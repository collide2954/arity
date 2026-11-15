import { describe, expect, it } from "vitest";

import { relatedReports } from "./related";
import type { Report } from "./types";

function value(id: number, work: number, property: string, subject: number): Report {
  return {
    id,
    work,
    run: work,
    local_id: `r${String(id)}`,
    kind: "value",
    body: {
      type: "value",
      id: `r${String(id)}`,
      subject: ["s"],
      property,
      value: { number: "1" },
    },
    entities: { s: subject },
    property,
    direction: null,
    score: null,
    check: "unchecked",
    checks: [],
    replaced_by: null,
    evidence: [],
    display: null,
  };
}

function effect(id: number, work: number, agent: number, target: number): Report {
  return {
    ...value(id, work, "ic50", agent),
    kind: "effect",
    body: {
      type: "effect",
      id: `e${String(id)}`,
      agents: [{ entity: "a" }],
      target: { entity: "t", property: "ic50" },
      direction: "decrease",
    },
    entities: { a: agent, t: target },
    direction: "decrease",
    score: -1,
  };
}

describe("relatedReports", () => {
  it("finds the values of other works for the same property and subject", () => {
    const reports = [
      value(1, 1, "ic50", 7),
      value(2, 1, "ic50", 7),
      value(3, 2, "ic50", 7),
      value(4, 2, "ic50", 8),
      value(5, 3, "melting_point", 7),
      value(6, 3, "ic50", 7),
    ];
    const [first] = reports;
    expect(first && relatedReports(first, reports).map((report) => report.id)).toEqual([3, 6]);
  });

  it("finds the effects of other works with the same target and an agent in common", () => {
    const reports = [
      effect(1, 1, 7, 9),
      effect(2, 2, 7, 9),
      effect(3, 2, 7, 10),
      effect(4, 3, 8, 9),
      value(5, 2, "ic50", 7),
    ];
    const [first] = reports;
    expect(first && relatedReports(first, reports).map((report) => report.id)).toEqual([2]);
  });
});
