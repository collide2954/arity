/**
 * The compare matrix: the reports of one property in rows and columns that a
 * person chooses. Each cell lists its reports; arity does not combine them.
 */

import { toCsv } from "./csv";
import { entityName, propertyName, type Names } from "./describe";
import { reportedValue } from "./format";
import type { ReportRow } from "./reports";
import { compareValues } from "./sort";

/** The value of a report that has no value for a dimension. */
export const NOT_STATED = "not stated";

/** A way to divide reports: by subject, work, method, or a role or a property of their conditions. */
export interface Dimension {
  readonly key: string;
  readonly label: string;
}

const ROLE_LABELS: Readonly<Record<string, string>> = { co_treatment: "co-treatment" };

function roleLabel(role: string): string {
  return ROLE_LABELS[role] ?? role.replaceAll("_", " ");
}

/** The dimensions that the reports have, in a fixed order. */
export function dimensions(rows: readonly ReportRow[], lookup: Names): Dimension[] {
  const roles = new Set<string>();
  const properties = new Set<string>();
  for (const row of rows) {
    for (const condition of row.report.body.conditions ?? []) {
      if (condition.property !== undefined && condition.value !== undefined) {
        properties.add(condition.property);
      } else if (condition.role !== undefined) {
        roles.add(condition.role);
      }
    }
  }
  return [
    { key: "subject", label: "Subject" },
    { key: "work", label: "Work" },
    ...(rows.some((row) => row.method !== "") ? [{ key: "method", label: "Method" }] : []),
    ...[...roles].map((role) => ({ key: `role:${role}`, label: roleLabel(role) })),
    ...[...properties].map((key) => ({
      key: `condition:${key}`,
      label: propertyName(key, lookup),
    })),
  ];
}

/** The value of a report for a dimension, or "not stated". */
export function dimensionValue(row: ReportRow, dimension: string, lookup: Names): string {
  if (dimension === "subject") return row.subject;
  if (dimension === "work") return row.citation;
  if (dimension === "method") return row.method || NOT_STATED;
  const [kind, key] = dimension.split(/:(.*)/s);
  const conditions = row.report.body.conditions ?? [];
  const values =
    kind === "role"
      ? conditions
          .filter((condition) => condition.role === key && condition.property === undefined)
          .map((condition) =>
            condition.entity === undefined
              ? (condition.text ?? "")
              : entityName(row.report, condition.entity, lookup),
          )
      : conditions.flatMap((condition) =>
          condition.property === key && condition.value !== undefined
            ? [reportedValue(condition.value)]
            : [],
        );
  const stated = values.filter((value) => value !== "");
  return stated.length > 0 ? stated.join(" + ") : NOT_STATED;
}

export interface Matrix {
  readonly rows: readonly string[];
  readonly columns: readonly string[];
  /** For each row, for each column, the reports of the cell. */
  readonly cells: ReadonlyMap<string, ReadonlyMap<string, readonly ReportRow[]>>;
}

function sortedLabels(labels: Iterable<string>): string[] {
  return [...new Set(labels)].sort((a, b) => {
    if (a === NOT_STATED || b === NOT_STATED)
      return Number(a === NOT_STATED) - Number(b === NOT_STATED);
    return compareValues(a, b);
  });
}

export function matrixOf(
  rows: readonly ReportRow[],
  byRow: string,
  byColumn: string,
  lookup: Names,
): Matrix {
  const cells = new Map<string, Map<string, ReportRow[]>>();
  const columns: string[] = [];
  for (const row of rows) {
    const across = dimensionValue(row, byRow, lookup);
    const down = dimensionValue(row, byColumn, lookup);
    columns.push(down);
    const line = cells.get(across) ?? new Map<string, ReportRow[]>();
    cells.set(across, line);
    const cell = line.get(down);
    if (cell) cell.push(row);
    else line.set(down, [row]);
  }
  return { rows: sortedLabels(cells.keys()), columns: sortedLabels(columns), cells };
}

/** The matrix as CSV text, with one line for each report. */
export function matrixCsv(matrix: Matrix, rowTitle: string, columnTitle: string): string {
  const lines = matrix.rows.flatMap((label) =>
    matrix.columns.flatMap((column) =>
      (matrix.cells.get(label)?.get(column) ?? []).map((row) => [
        label,
        column,
        row.report.id,
        row.reported,
        row.converted,
        row.unit,
        row.citation,
        row.report.check,
      ]),
    ),
  );
  return toCsv(
    [rowTitle, columnTitle, "report", "as_reported", "converted", "unit", "source", "check"],
    lines,
  );
}

/** The settings of the matrix that the address keeps. */
export interface Compare {
  readonly property: string | null;
  readonly rows: string;
  readonly columns: string;
  /** True to show the converted values, false to show the values as reported. */
  readonly converted: boolean;
}

export function readCompare(params: URLSearchParams): Compare {
  return {
    property: params.get("property"),
    rows: params.get("by") ?? "subject",
    columns: params.get("across") ?? "work",
    converted: params.get("values") === "converted",
  };
}

export function writeCompare(params: URLSearchParams, compare: Compare): void {
  if (compare.property !== null) params.set("property", compare.property);
  params.set("by", compare.rows);
  params.set("across", compare.columns);
  if (compare.converted) params.set("values", "converted");
}
