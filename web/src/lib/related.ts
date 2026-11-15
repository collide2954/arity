import type { Report } from "./types";

function ids(report: Report, locals: readonly string[]): Set<number> {
  return new Set(locals.flatMap((local) => report.entities[local] ?? []));
}

function shares(first: ReadonlySet<number>, second: ReadonlySet<number>): boolean {
  return [...first].some((id) => second.has(id));
}

/**
 * The reports of other works that state the same thing about the same
 * entities: for a value, the same property of a subject in common; for an
 * effect, the same target and an agent in common.
 */
export function relatedReports(report: Report, reports: readonly Report[]): Report[] {
  const body = report.body;
  if (body.type === "value") {
    const subjects = ids(report, body.subject);
    return reports.filter(
      (other) =>
        other.work !== report.work &&
        other.body.type === "value" &&
        other.property === report.property &&
        shares(subjects, ids(other, other.body.subject)),
    );
  }
  const agents = ids(
    report,
    body.agents.map((agent) => agent.entity),
  );
  const target = report.entities[body.target.entity];
  return reports.filter(
    (other) =>
      other.work !== report.work &&
      other.body.type === "effect" &&
      target !== undefined &&
      other.entities[other.body.target.entity] === target &&
      shares(
        agents,
        ids(
          other,
          other.body.agents.map((agent) => agent.entity),
        ),
      ),
  );
}
