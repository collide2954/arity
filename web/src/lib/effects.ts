import { DIRECTIONS, type EffectRow } from "./charts";
import { NOT_STATED } from "./compare";
import type { Item } from "./filter";
import {
  conditionParts,
  conditions,
  entityName,
  names,
  propertyName,
  type ConditionPart,
} from "./describe";
import { directionLabel, scoreLabel, significanceLabel } from "./format";
import { markerSign, noticeLabel, noticeNames } from "./notices";
import { facetLabels } from "./reports";
import { searchForm } from "./search";
import { compareValues } from "./sort";
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
  /** The conditions, each as a name and a value. */
  readonly contextParts: readonly ConditionPart[];
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
        contextParts: conditionParts(report, lookup),
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
      group: `${row.agent} \u{2192} ${row.target}`,
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

/** The forms of the figure of effects. */
export type EffectView = "tiles" | "list" | "matrix";

export const EFFECT_VIEWS: readonly EffectView[] = ["tiles", "list", "matrix"];

/** A way to put effects into the rows of the figure. */
export interface Grouping {
  readonly key: string;
  readonly label: string;
}

/** The groupings of the effects: agent, target, both, work, and each condition, the most frequent first. */
export function effectGroupings(rows: readonly EffectItem[]): Grouping[] {
  const counts = new Map<string, number>();
  for (const row of rows) {
    for (const name of new Set(row.contextParts.map((part) => part.name))) {
      if (name !== "") counts.set(name, (counts.get(name) ?? 0) + 1);
    }
  }
  const names = [...counts]
    .sort((a, b) => b[1] - a[1] || compareValues(a[0], b[0]))
    .map(([name]) => name);
  return [
    { key: "agent", label: "Agent" },
    { key: "target", label: "Target" },
    { key: "pair", label: "Agent and target" },
    { key: "work", label: "Work" },
    ...names.map((name) => ({ key: `condition:${name}`, label: name })),
  ];
}

/** The row of an effect for a grouping. An effect without the condition goes into "not stated". */
export function groupOf(row: EffectItem, key: string): string {
  switch (key) {
    case "agent":
      return row.agent;
    case "target":
      return row.target;
    case "pair":
      return `${row.agent} \u{2192} ${row.target}`;
    case "work":
      return row.citation;
    default: {
      const name = key.replace(/^condition:/, "");
      const values = row.contextParts
        .filter((part) => part.name === name)
        .map((part) => part.value);
      return values.length > 0 ? values.join(" + ") : NOT_STATED;
    }
  }
}

/**
 * The marks of the tiles or the list, with the row of each effect. In tiles,
 * the reports of a row go in the order of the directions. In a list, the
 * reports of one row and one direction take the next slot.
 */
export function groupedMarks(
  rows: readonly EffectItem[],
  key: string,
  view: Exclude<EffectView, "matrix">,
): EffectRow[] {
  const marks = matrixRows(rows).map((mark, position) => {
    const row = rows[position];
    return { ...mark, group: row ? groupOf(row, key) : mark.group };
  });
  const order = (direction: string) => (DIRECTIONS as readonly string[]).indexOf(direction);
  const slots = new Map<EffectRow, number>();
  if (view === "tiles") {
    const groups = new Map<string, EffectRow[]>();
    for (const mark of marks) {
      const members = groups.get(mark.group);
      if (members) members.push(mark);
      else groups.set(mark.group, [mark]);
    }
    for (const members of groups.values()) {
      [...members]
        .sort((a, b) => order(a.direction) - order(b.direction))
        .forEach((mark, slot) => slots.set(mark, slot));
    }
  } else {
    const next = new Map<string, number>();
    for (const mark of marks) {
      const cell = `${mark.group}\u{0}${mark.direction}`;
      const slot = next.get(cell) ?? 0;
      next.set(cell, slot + 1);
      slots.set(mark, slot);
    }
  }
  return marks.map((mark) => ({ ...mark, slot: slots.get(mark) ?? 0 }));
}
