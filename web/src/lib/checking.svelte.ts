import { readDraft, withMark, withoutMark, type Draft, type Mark } from "./checkfile";

const DRAFT = "arity-checks";
const MODE = "arity-check-mode";

function stored(key: string): string | null {
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}

function keep(key: string, value: string | null): void {
  try {
    if (value === null) localStorage.removeItem(key);
    else localStorage.setItem(key, value);
  } catch {
    return;
  }
}

/** Check mode, and the draft of the checks. The browser keeps both. */
export const checking: { on: boolean; draft: Draft } = $state({
  on: stored(MODE) === "on",
  draft: readDraft(stored(DRAFT)),
});

function save(draft: Draft): void {
  checking.draft = draft;
  keep(DRAFT, JSON.stringify(draft));
}

export function setCheckMode(on: boolean): void {
  checking.on = on;
  keep(MODE, on ? "on" : null);
}

export function addMark(mark: Mark): void {
  save(withMark(checking.draft, mark));
}

export function removeMark(report: number): void {
  save(withoutMark(checking.draft, report));
}

export function removeMarks(): void {
  save({ ...checking.draft, marks: [] });
}

export function setPerson(name: string, orcid: string): void {
  save({ ...checking.draft, person: { name, orcid } });
}
