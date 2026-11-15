import { describe, expect, it } from "vitest";

import { agentName, runState } from "./runs";
import type { Run } from "./types";

function run(replaces: number | null, replaced_by: number | null): Run {
  const agent = { kind: "person", name: "A Person", version: null, orcid: null };
  return {
    id: 2,
    work: 1,
    reader: agent,
    responsible: agent,
    software: "arity 0.1.0",
    submitted_at: "2026-11-14T10:00:00Z",
    submission_sha256: "a",
    text_sha256: "b",
    parser: "test 1",
    warnings: [],
    note: null,
    replaces,
    replaced_by,
  };
}

describe("runState", () => {
  it("tells whether a run is current, corrects a run, or is replaced", () => {
    expect(runState(run(null, null))).toBe("Current");
    expect(runState(run(1, null))).toBe("Current. It corrects run 1.");
    expect(runState(run(null, 3))).toBe("Replaced by run 3");
    expect(runState(run(1, 3))).toBe("Replaced by run 3");
  });
});

describe("agentName", () => {
  it("gives the name of an agent, with its version when it has one", () => {
    const software = { kind: "software", name: "Claude (Anthropic)", orcid: null };
    expect(agentName({ ...software, version: "claude-opus-5-5" })).toBe(
      "Claude (Anthropic) claude-opus-5-5",
    );
    expect(agentName({ ...software, version: null })).toBe("Claude (Anthropic)");
  });
});
