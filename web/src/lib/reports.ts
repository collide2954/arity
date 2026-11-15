import { DIRECTIONS } from "./charts";
import { conditions, entityName, names, propertyName, type Names } from "./describe";
import type { Item } from "./filter";
import { directionLabel, reportedValue, scoreLabel, unitLabel } from "./format";
import { noticeLabel } from "./notices";
import { searchForm } from "./search";
import type { EffectBody, Index, Report, ValueBody, WorkSummary } from "./types";
import type { Column } from "./view";

export const REPORT_FACETS = [
  "property",
  "entity",
  "condition",
  "work",
  "kind",
  "check",
  "notice",
  "origin",
  "direction",
] as const;

export type ReportFacet = (typeof REPORT_FACETS)[number];

export const FACET_TITLES: Readonly<Record<ReportFacet, string>> = {
  property: "Property",
  entity: "Entity",
  condition: "Condition",
  work: "Work",
  kind: "Kind of report",
  check: "Check",
  notice: "Notice",
  origin: "Origin of the value",
  direction: "Direction of the effect",
};

/** A report with the texts that the tables show and the values that the filters test. */
export interface ReportRow extends Item<ReportFacet> {
  readonly report: Report;
  /** The property of a value, or the kind and the target property of an effect. */
  readonly about: string;
  /** The subject of a value, or the agents and the target of an effect. */
  readonly subject: string;
  /** The value as the source states it, or the direction of an effect. */
  readonly reported: string;
  /** The value in the unit of the property, as arity calculates it. */
  readonly converted: number | null;
  readonly unit: string;
  readonly conditions: readonly string[];
  readonly method: string;
  readonly citation: string;
  /** The PMCID of the work, or its DOI. */
  readonly identifier: string;
  readonly year: number | null;
  readonly notices: readonly string[];
  /** The first quote of the report. */
  readonly quote: string;
}

interface Statement {
  readonly about: string;
  readonly subject: string;
  readonly reported: string;
}

function valueStatement(report: Report, body: ValueBody, lookup: Names): Statement {
  return {
    about: propertyName(body.property, lookup),
    subject: body.subject.map((local) => entityName(report, local, lookup)).join(" + "),
    reported: reportedValue(body.value),
  };
}

function effectStatement(report: Report, body: EffectBody, lookup: Names): Statement {
  const association = body.relation === "association";
  const relation = association ? "association" : "effect";
  const agents = body.agents.map((agent) => entityName(report, agent.entity, lookup)).join(" + ");
  return {
    about: body.target.property
      ? `${relation} ${association ? "with" : "on"} ${propertyName(body.target.property, lookup)}`
      : relation,
    subject: `${agents} \u{2192} ${entityName(report, body.target.entity, lookup)}`,
    reported: `${directionLabel(body.direction)} (${scoreLabel(report.score)})`,
  };
}

/** Gives the search form of a text, and keeps it for the next request of the same text. */
type SearchForms = (text: string) => string;

function searchForms(): SearchForms {
  const known = new Map<string, string>();
  return (text) => {
    let form = known.get(text);
    if (form === undefined) {
      form = searchForm(text);
      known.set(text, form);
    }
    return form;
  };
}

function reportRow(
  report: Report,
  lookup: Names,
  work: WorkSummary | undefined,
  formOf: SearchForms,
): ReportRow {
  const body = report.body;
  const statement =
    body.type === "value"
      ? valueStatement(report, body, lookup)
      : effectStatement(report, body, lookup);
  const context = conditions(report, lookup);
  const ids = [...new Set(Object.values(report.entities))];
  const entities = ids.flatMap((id) => lookup.entities.get(id) ?? []);
  const notices = work?.notices ?? [];
  const citation = work?.citation ?? `work ${String(report.work)}`;
  const quotes = report.evidence.map((evidence) => evidence.quote);
  const converted = report.display?.value ?? null;
  const text = [
    report.property,
    statement.about,
    statement.subject,
    statement.reported,
    ...entities.flatMap((entity) => [entity.label, ...entity.names, ...entity.xrefs]),
    ...context,
    body.method?.label,
    body.note,
    citation,
    work?.title,
    work?.pmcid,
    work?.doi,
    report.local_id,
    ...quotes,
  ]
    .flatMap((part) => (part ? [formOf(part)] : []))
    .join(" ");
  return {
    report,
    ...statement,
    converted,
    unit: report.display ? unitLabel(report.display.unit) : "",
    conditions: context,
    method: body.method?.label ?? "",
    citation,
    identifier: work?.pmcid ?? work?.doi ?? "",
    year: work?.year ?? null,
    notices,
    quote: quotes[0] ?? "",
    text,
    number: converted,
    values: {
      property: report.property === null ? [] : [report.property],
      entity: ids.map(String),
      condition: [...new Set(context)],
      work: [String(report.work)],
      kind: [report.kind],
      check: [report.check],
      notice: notices.length > 0 ? notices : ["none"],
      origin: body.type === "value" ? [body.value.read_from_figure ? "image" : "stated"] : [],
      direction: body.type === "effect" ? [body.direction] : [],
    },
  };
}

/** The rows of the reports, in the order of the reports. */
export function reportRows(index: Index, reports: readonly Report[]): ReportRow[] {
  const lookup = names(index);
  const works = new Map(index.works.map((work) => [work.id, work]));
  const formOf = searchForms();
  return reports.map((report) => reportRow(report, lookup, works.get(report.work), formOf));
}

const KINDS: Readonly<Record<string, string>> = { value: "Value", effect: "Effect" };

const CHECKS: Readonly<Record<string, string>> = {
  confirmed: "Confirmed",
  unchecked: "Not checked",
  rejected: "Rejected",
};

const ORIGINS: Readonly<Record<string, string>> = {
  stated: "Stated in the text",
  image: "Read from an image",
};

function isDirection(value: string): value is EffectBody["direction"] {
  return (DIRECTIONS as readonly string[]).includes(value);
}

/** Names the values of the facets of reports. */
export function facetLabels(index: Index): (facet: ReportFacet, value: string) => string {
  const lookup = names(index);
  const works = new Map(index.works.map((work) => [String(work.id), work]));
  return (facet, value) => {
    switch (facet) {
      case "property":
        return propertyName(value, lookup);
      case "entity": {
        const entity = lookup.entities.get(Number(value));
        return entity ? `${entity.label} (${entity.kind.replaceAll("_", " ")})` : `entity ${value}`;
      }
      case "condition":
        return value;
      case "work":
        return works.get(value)?.citation ?? `work ${value}`;
      case "kind":
        return KINDS[value] ?? value;
      case "check":
        return CHECKS[value] ?? value;
      case "notice":
        return value === "none" ? "No notice" : noticeLabel(value);
      case "origin":
        return ORIGINS[value] ?? value;
      case "direction":
        return isDirection(value) ? directionLabel(value) : value;
    }
  };
}

export const REPORT_COLUMNS: readonly Column<ReportRow>[] = [
  { id: "about", label: "Property or effect", value: (row) => row.about },
  { id: "subject", label: "Subject", value: (row) => row.subject },
  { id: "reported", label: "As reported", value: (row) => row.reported },
  { id: "converted", label: "Converted", value: (row) => row.converted, numeric: true },
  { id: "unit", label: "Unit", value: (row) => row.unit },
  { id: "conditions", label: "Conditions", value: (row) => row.conditions.join("; ") },
  { id: "method", label: "Method", value: (row) => row.method },
  { id: "source", label: "Source", value: (row) => row.citation },
  { id: "identifier", label: "Work", value: (row) => row.identifier },
  { id: "year", label: "Year", value: (row) => row.year, numeric: true },
  { id: "check", label: "Check", value: (row) => row.report.check },
  { id: "notices", label: "Notices", value: (row) => row.notices.map(noticeLabel).join("; ") },
  { id: "quote", label: "Quote", value: (row) => row.quote },
  { id: "id", label: "Report", value: (row) => row.report.id, numeric: true },
];

export const DEFAULT_REPORT_COLUMNS: readonly string[] = [
  "about",
  "subject",
  "reported",
  "converted",
  "conditions",
  "source",
  "check",
  "notices",
];
