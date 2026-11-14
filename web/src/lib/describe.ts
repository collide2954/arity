import { reportedValue, unitLabel } from "./format";
import type { Condition, EntitySummary, Index, PropertySummary, Report } from "./types";

/** Lookups that turn the identifiers of a report into names. */
export interface Names {
  readonly entities: ReadonlyMap<number, EntitySummary>;
  readonly properties: ReadonlyMap<string, PropertySummary>;
}

export function names(index: Index): Names {
  return {
    entities: new Map(index.entities.map((entity) => [entity.id, entity])),
    properties: new Map(index.properties.map((property) => [property.key, property])),
  };
}

export function entityName(report: Report, local: string, lookup: Names): string {
  const id = report.entities[local];
  return (id === undefined ? undefined : lookup.entities.get(id)?.label) ?? local;
}

export function propertyName(key: string | undefined, lookup: Names): string {
  if (key === undefined) return "";
  return lookup.properties.get(key)?.label ?? key;
}

const ROLES: Record<string, string> = {
  cell_line: "cell line",
  cell_type: "cell type",
  anatomical_site: "site",
  co_treatment: "with",
};

export function conditionText(report: Report, condition: Condition, lookup: Names): string {
  if (condition.property !== undefined && condition.value !== undefined) {
    return `${propertyName(condition.property, lookup)} ${reportedValue(condition.value)}`;
  }
  const role = condition.role === undefined ? "" : (ROLES[condition.role] ?? condition.role);
  const what =
    condition.entity === undefined
      ? (condition.text ?? "")
      : entityName(report, condition.entity, lookup);
  return role ? `${role}: ${what}` : what;
}

export function conditions(report: Report, lookup: Names): string[] {
  return (report.body.conditions ?? []).map((condition) =>
    conditionText(report, condition, lookup),
  );
}

/** A short statement of the report, for lists and tooltips. */
export function headline(report: Report, lookup: Names): string {
  const body = report.body;
  if (body.type === "value") {
    const subject = body.subject.map((local) => entityName(report, local, lookup)).join(" + ");
    return `${propertyName(body.property, lookup)} of ${subject}: ${reportedValue(body.value)}`;
  }
  const agents = body.agents.map((agent) => entityName(report, agent.entity, lookup)).join(" + ");
  const target = entityName(report, body.target.entity, lookup);
  const property = body.target.property ? ` ${propertyName(body.target.property, lookup)}` : "";
  const verb = body.relation === "association" ? "associated with" : "effect on";
  const words: Record<typeof body.direction, string> = {
    increase: "increase",
    decrease: "decrease",
    no_change: "no change (0)",
    mixed: "mixed",
  };
  return `${agents}, ${verb} ${target}${property}: ${words[body.direction]}`;
}

export function displayUnit(report: Report): string {
  return report.display ? unitLabel(report.display.unit) : "";
}
