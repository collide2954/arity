import type { EffectRow } from "./charts";
import { conditions, entityName, names, propertyName } from "./describe";
import { directionLabel, scoreLabel, significanceLabel } from "./format";
import type { EffectBody, Index, Report } from "./types";

/** One reported effect, for the table and the matrix. */
export interface EffectTableRow {
  readonly report: Report;
  readonly body: EffectBody;
  readonly agent: string;
  readonly target: string;
  readonly citation: string;
  readonly detail: string;
  readonly context: string;
}

export function effectRows(index: Index, reports: readonly Report[]): EffectTableRow[] {
  const lookup = names(index);
  const works = new Map(index.works.map((work) => [work.id, work]));
  return reports
    .filter((report) => report.body.type === "effect")
    .map((report) => {
      const body = report.body as EffectBody;
      const agent = body.agents.map((item) => entityName(report, item.entity, lookup)).join(" + ");
      const property = body.target.property
        ? ` \u{b7} ${propertyName(body.target.property, lookup)}`
        : "";
      const magnitude = body.magnitude
        ? `${body.magnitude.measure.replaceAll("_", " ")} ${body.magnitude.number}`
        : "";
      const detail = [magnitude, significanceLabel(body.significance) ?? ""]
        .filter((part) => part.length > 0)
        .join(", ");
      return {
        report,
        body,
        agent,
        target: `${entityName(report, body.target.entity, lookup)}${property}`,
        citation: works.get(report.work)?.citation ?? `work ${String(report.work)}`,
        detail,
        context: conditions(report, lookup).join(" \u{b7} "),
      };
    });
}

/**
 * The marks of the matrix. Reports in the same cell take the next slot, so
 * that each report stays visible as its own mark.
 */
export function matrixRows(rows: readonly EffectTableRow[]): EffectRow[] {
  const slots = new Map<string, number>();
  return rows.map((row) => {
    const cell = `${row.agent}\u{0}${row.target}`;
    const slot = slots.get(cell) ?? 0;
    slots.set(cell, slot + 1);
    return {
      report: row.report.id,
      agent: row.agent,
      target: row.target,
      direction: row.body.direction,
      score: `${directionLabel(row.body.direction)} (${scoreLabel(row.report.score)})`,
      slot,
      work: row.citation,
      detail: row.detail || "none",
      check: row.report.check,
    };
  });
}
