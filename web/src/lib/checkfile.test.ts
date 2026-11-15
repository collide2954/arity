import { describe, expect, it } from "vitest";

import {
  checkFile,
  emptyDraft,
  isOrcid,
  markOf,
  readDraft,
  withMark,
  withoutMark,
  type Draft,
} from "./checkfile";
import type { Report, WorkSummary } from "./types";

const work: WorkSummary = {
  id: 1,
  title: "A",
  citation: "Lafi et al. 2023, PLOS ONE",
  year: 2023,
  journal: null,
  doi: "10.1371/journal.pone.0291981",
  pmcid: "PMC10538757",
  license: null,
  shareable: true,
  is_preprint: false,
  notices: [],
  reports: 1,
  state: "read",
};

const report = { id: 40, work: 1, run: 14, local_id: "dox_ic50" } as Report;

describe("markOf", () => {
  it("names the report by its work, its run, and its identifier in the submission", () => {
    expect(
      markOf(report, work, "rejected", " The table states 0.42. ", "2026-11-15T10:00:00Z"),
    ).toEqual({
      report: 40,
      work: "PMC10538757",
      run: 14,
      local: "dox_ic50",
      state: "rejected",
      note: "The table states 0.42.",
      at: "2026-11-15T10:00:00Z",
    });
    expect(
      markOf(report, { ...work, pmcid: null }, "confirmed", "", "2026-11-15T10:00:00Z").work,
    ).toBe("10.1371/journal.pone.0291981");
  });
});

describe("withMark and withoutMark", () => {
  it("keep one mark for each report", () => {
    const first = markOf(report, work, "confirmed", "", "2026-11-15T10:00:00Z");
    const second = markOf(report, work, "rejected", "No.", "2026-11-15T11:00:00Z");
    const draft = withMark(withMark(emptyDraft(), first), second);
    expect(draft.marks).toEqual([second]);
    expect(withoutMark(draft, 40).marks).toEqual([]);
  });
});

describe("checkFile", () => {
  it("writes the check file that arity reads, with the person and each check", () => {
    const draft: Draft = {
      person: { name: " A Checker ", orcid: "0000-0002-1825-0097" },
      marks: [
        markOf(report, work, "confirmed", "", "2026-11-15T10:00:00Z"),
        markOf(
          { ...report, id: 41, local_id: "dox_ic50_b" },
          work,
          "rejected",
          "No.",
          "2026-11-15T10:05:00Z",
        ),
      ],
    };
    expect(JSON.parse(checkFile(draft))).toEqual({
      format: "arity-checks/1",
      person: { name: "A Checker", orcid: "0000-0002-1825-0097" },
      checks: [
        {
          work: "PMC10538757",
          run: 14,
          report: "dox_ic50",
          state: "confirmed",
          at: "2026-11-15T10:00:00Z",
        },
        {
          work: "PMC10538757",
          run: 14,
          report: "dox_ic50_b",
          state: "rejected",
          note: "No.",
          at: "2026-11-15T10:05:00Z",
        },
      ],
    });
  });

  it("leaves out an ORCID that is not valid", () => {
    const draft: Draft = { person: { name: "A Checker", orcid: "0000-1" }, marks: [] };
    expect((JSON.parse(checkFile(draft)) as { person: unknown }).person).toEqual({
      name: "A Checker",
    });
    expect(isOrcid("0000-0002-1825-009X")).toBe(true);
    expect(isOrcid("0000-1")).toBe(false);
  });
});

describe("readDraft", () => {
  it("reads a stored draft, and gives an empty draft for text that is not a draft", () => {
    const draft: Draft = {
      person: { name: "A Checker", orcid: "" },
      marks: [markOf(report, work, "confirmed", "", "2026-11-15T10:00:00Z")],
    };
    expect(readDraft(JSON.stringify(draft))).toEqual(draft);
    expect(readDraft(null)).toEqual(emptyDraft());
    expect(readDraft("{")).toEqual(emptyDraft());
    expect(readDraft('{"person": 3, "marks": [{"report": "x"}]}')).toEqual(emptyDraft());
  });
});
