/**
 * Check mode: the marks that a person makes on reports, and the check file
 * that `arity check --file` reads.
 */

import type { Report, WorkSummary } from "./types";

export const CHECKS_FORMAT = "arity-checks/1";

export type Verdict = "confirmed" | "rejected";

/** The check of one report by the person of the draft. */
export interface Mark {
  /** The id of the report on this website. */
  readonly report: number;
  /** The PMCID or the DOI of the work of the report. */
  readonly work: string;
  readonly run: number;
  /** The identifier of the report in its submission. */
  readonly local: string;
  readonly state: Verdict;
  readonly note: string;
  /** When the person made the check, in RFC 3339 form. */
  readonly at: string;
}

export interface Draft {
  readonly person: { readonly name: string; readonly orcid: string };
  readonly marks: readonly Mark[];
}

export function emptyDraft(): Draft {
  return { person: { name: "", orcid: "" }, marks: [] };
}

export function markOf(
  report: Report,
  work: WorkSummary,
  state: Verdict,
  note: string,
  at: string,
): Mark {
  return {
    report: report.id,
    work: work.pmcid ?? work.doi ?? String(work.id),
    run: report.run,
    local: report.local_id,
    state,
    note: note.trim(),
    at,
  };
}

/** The draft with the mark, in place of an earlier mark of the same report. */
export function withMark(draft: Draft, mark: Mark): Draft {
  return {
    ...draft,
    marks: [...draft.marks.filter((other) => other.report !== mark.report), mark],
  };
}

export function withoutMark(draft: Draft, report: number): Draft {
  return { ...draft, marks: draft.marks.filter((mark) => mark.report !== report) };
}

export function isOrcid(text: string): boolean {
  return /^\d{4}-\d{4}-\d{4}-\d{3}[\dX]$/.test(text.trim());
}

/** The check file of the draft, as JSON text. */
export function checkFile(draft: Draft): string {
  const name = draft.person.name.trim();
  const orcid = draft.person.orcid.trim();
  return `${JSON.stringify(
    {
      format: CHECKS_FORMAT,
      person: isOrcid(orcid) ? { name, orcid } : { name },
      checks: draft.marks.map((mark) => ({
        work: mark.work,
        run: mark.run,
        report: mark.local,
        state: mark.state,
        ...(mark.note === "" ? {} : { note: mark.note }),
        at: mark.at,
      })),
    },
    null,
    2,
  )}\n`;
}

function isMark(value: unknown): value is Mark {
  if (typeof value !== "object" || value === null) return false;
  const mark = value as Record<string, unknown>;
  return (
    typeof mark.report === "number" &&
    typeof mark.work === "string" &&
    typeof mark.run === "number" &&
    typeof mark.local === "string" &&
    (mark.state === "confirmed" || mark.state === "rejected") &&
    typeof mark.note === "string" &&
    typeof mark.at === "string"
  );
}

/** Reads a draft that the browser stored. Text that is not a draft gives an empty draft. */
export function readDraft(text: string | null): Draft {
  if (text === null) return emptyDraft();
  try {
    const value = JSON.parse(text) as { person?: unknown; marks?: unknown };
    const person = value.person as { name?: unknown; orcid?: unknown } | undefined;
    if (
      typeof person?.name !== "string" ||
      typeof person.orcid !== "string" ||
      !Array.isArray(value.marks) ||
      !value.marks.every(isMark)
    ) {
      return emptyDraft();
    }
    return { person: { name: person.name, orcid: person.orcid }, marks: value.marks };
  } catch {
    return emptyDraft();
  }
}
