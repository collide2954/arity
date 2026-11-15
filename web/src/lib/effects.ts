import type { EffectRow } from "./charts";
import type { Item } from "./filter";
import { conditions, entityName, names, propertyName } from "./describe";
import { directionLabel, scoreLabel, significanceLabel } from "./format";
import { markerSign, noticeLabel, noticeNames } from "./notices";
import { facetLabels } from "./reports";
import { searchForm } from "./search";
import type { EffectBody, Index, Report } from "./types";
import type { Column } from "./view";

/** One reported effect, for the table and the matrix. */
export interface EffectTableRow {
  readonly report: Report;
  readonly body: EffectBody;
  readonly agent: string;
  readonly target: string;
  readonly citation: string;
  readonly notices: readonly string[];
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
      const work = works.get(report.work);
      return {
        report,
        body,
        agent,
        target: `${entityName(report, body.target.entity, lookup)}${property}`,
        citation: work?.citation ?? `work ${String(report.work)}`,
        notices: work?.notices ?? [],
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
      notices: noticeNames(row.notices) || "none",
      marker: markerSign(row.notices),
    };
  });
}

export const EFFECT_FACETS = ["agent", "target", "direction", "work", "check", "notice"] as const;
export type EffectFacet = (typeof EFFECT_FACETS)[number];

export const EFFECT_TITLES: Readonly<Record<EffectFacet, string>> = {
  agent: "Agent",
  target: "Target",
  direction: "Direction",
  work: "Work",
  check: "Check",
  notice: "Notice",
};

/** An effect with the text and the values that the filters test. */
export interface EffectItem extends EffectTableRow, Item<EffectFacet> {}

export function effectItems(rows: readonly EffectTableRow[]): EffectItem[] {
  return rows.map((row) => ({
    ...row,
    text: searchForm(
      [
        row.agent,
        row.target,
        directionLabel(row.body.direction),
        row.detail,
        row.context,
        row.citation,
        row.body.comparator ?? "",
        row.body.note ?? "",
        ...row.report.evidence.map((evidence) => evidence.quote),
      ].join(" "),
    ),
    number: null,
    values: {
      agent: [row.agent],
      target: [row.target],
      direction: [row.body.direction],
      work: [String(row.report.work)],
      check: [row.report.check],
      notice: row.notices.length > 0 ? row.notices : ["none"],
    },
  }));
}

/** Names the values of the facets of effects. */
export function effectLabels(index: Index): (facet: EffectFacet, value: string) => string {
  const label = facetLabels(index);
  return (facet, value) => (facet === "agent" || facet === "target" ? value : label(facet, value));
}

export const EFFECT_COLUMNS: readonly Column<EffectItem>[] = [
  { id: "agent", label: "Agent", value: (row) => row.agent },
  { id: "target", label: "Target", value: (row) => row.target },
  { id: "direction", label: "Direction", value: (row) => row.report.score, numeric: true },
  { id: "details", label: "Details", value: (row) => row.detail },
  { id: "conditions", label: "Conditions", value: (row) => row.context },
  { id: "source", label: "Source", value: (row) => row.citation },
  { id: "check", label: "Check", value: (row) => row.report.check },
  { id: "notices", label: "Notices", value: (row) => row.notices.map(noticeLabel).join("; ") },
];
