/** The targets of the quick jump: the pages, properties, entities, works, and reports. */

import { headline, names } from "./describe";
import type { Route } from "./route";
import { matchesAll, searchForm, terms } from "./search";
import type { Index, Report } from "./types";

export type TargetKind = "page" | "property" | "entity" | "work" | "report";

export interface Target {
  readonly kind: TargetKind;
  readonly label: string;
  readonly detail: string;
  readonly route: Route;
  /** The text that the search finds the target by, in search form. */
  readonly text: string;
}

const PAGES: readonly { readonly label: string; readonly route: Route }[] = [
  { label: "Works", route: { page: "works" } },
  { label: "Reports", route: { page: "reports" } },
  { label: "Properties", route: { page: "properties" } },
  { label: "Entities", route: { page: "entities" } },
  { label: "Effects", route: { page: "effects" } },
  { label: "Compare", route: { page: "compare" } },
  { label: "Your checks", route: { page: "checks" } },
  { label: "About", route: { page: "about" } },
];

const ORDER: Readonly<Record<TargetKind, number>> = {
  page: 0,
  property: 1,
  entity: 2,
  work: 3,
  report: 4,
};

function textOf(...parts: readonly (string | null | undefined)[]): string {
  return searchForm(parts.filter((part) => part).join(" "));
}

/** The pages, properties, entities, and works of the catalogue. */
export function catalogueTargets(index: Index): Target[] {
  return [
    ...PAGES.map(({ label, route }) => ({
      kind: "page" as const,
      label,
      detail: "page",
      route,
      text: textOf(label),
    })),
    ...index.properties.map((property) => ({
      kind: "property" as const,
      label: property.label,
      detail: `property \u{b7} ${String(property.reports)} reports`,
      route: { page: "property" as const, key: property.key },
      text: textOf(property.label, property.key, property.unit),
    })),
    ...index.entities.map((entity) => ({
      kind: "entity" as const,
      label: entity.label,
      detail: `${entity.kind.replaceAll("_", " ")} \u{b7} ${String(entity.reports)} reports`,
      route: { page: "entity" as const, id: entity.id },
      text: textOf(
        entity.label,
        entity.kind.replaceAll("_", " "),
        ...entity.names,
        ...entity.xrefs,
      ),
    })),
    ...index.works.map((work) => ({
      kind: "work" as const,
      label: work.title,
      detail: work.citation,
      route: { page: "work" as const, id: work.id, report: null },
      text: textOf(work.title, work.citation, work.pmcid, work.doi),
    })),
  ];
}

/** The reports, each with its statement and its source. */
export function reportTargets(index: Index, reports: readonly Report[]): Target[] {
  const lookup = names(index);
  const works = new Map(index.works.map((work) => [work.id, work]));
  return reports.map((report) => {
    const work = works.get(report.work);
    const label = headline(report, lookup);
    return {
      kind: "report" as const,
      label,
      detail: work?.citation ?? `work ${String(report.work)}`,
      route: { page: "report" as const, id: report.id },
      text: textOf(
        label,
        report.property,
        report.local_id,
        work?.citation,
        work?.pmcid,
        `#${String(report.id)}`,
      ),
    };
  });
}

/**
 * The targets that have each word of the search, at most `limit`: first those
 * whose name starts with the search, then pages, properties, entities, works,
 * and reports.
 */
export function findTargets(targets: readonly Target[], search: string, limit: number): Target[] {
  const words = terms(search);
  if (words.length === 0) return [];
  const start = searchForm(search);
  return targets
    .filter((target) => matchesAll(target.text, words))
    .map((target, position) => ({
      target,
      position,
      rank: (searchForm(target.label).startsWith(start) ? 0 : 10) + ORDER[target.kind],
    }))
    .sort((a, b) => a.rank - b.rank || a.position - b.position)
    .slice(0, limit)
    .map(({ target }) => target);
}
