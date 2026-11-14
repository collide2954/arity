import type { ValueRow } from "./charts";
import { conditions, entityName, names } from "./describe";
import { reportedValue } from "./format";
import type { Index, Report, ValueBody } from "./types";

/** One reported value of a property, for the table and the figure. */
export interface ValueTableRow {
  readonly report: Report;
  readonly body: ValueBody;
  readonly label: string;
  readonly subject: string;
  readonly conditions: readonly string[];
  readonly citation: string;
  readonly notices: readonly string[];
}

/** The value reports of a property, each with a unique label for the figure. */
export function valueRows(
  index: Index,
  reports: readonly Report[],
  property: string,
  includeRejected: boolean,
): ValueTableRow[] {
  const lookup = names(index);
  const works = new Map(index.works.map((work) => [work.id, work]));
  const counts = new Map<string, number>();
  return reports
    .filter((report) => report.property === property && report.body.type === "value")
    .filter((report) => includeRejected || report.check !== "rejected")
    .map((report) => {
      const body = report.body as ValueBody;
      const subject = body.subject.map((local) => entityName(report, local, lookup)).join(" + ");
      const work = works.get(report.work);
      const context = conditions(report, lookup);
      const citation = work?.citation ?? `work ${String(report.work)}`;
      const base = `${[subject, ...context].join(" \u{b7} ")} \u{2014} ${citation}`;
      const count = (counts.get(base) ?? 0) + 1;
      counts.set(base, count);
      return {
        report,
        body,
        subject,
        conditions: context,
        citation,
        notices: work?.notices ?? [],
        label: count === 1 ? base : `${base} (${String(count)})`,
      };
    });
}

/** The rows that the figure can plot: those with a value in the display unit. */
export function figureRows(rows: readonly ValueTableRow[]): ValueRow[] {
  return rows.flatMap((row) => {
    const display = row.report.display;
    if (display?.value === null || display?.value === undefined) return [];
    return [
      {
        report: row.report.id,
        label: row.label,
        value: display.value,
        low: display.interval_low,
        high: display.interval_high,
        reported: reportedValue(row.body.value),
        work: row.citation,
        check: row.report.check,
        origin: row.body.value.read_from_figure ? "read from figure" : "stated",
        notices: row.notices.join(", ") || "none",
      },
    ];
  });
}
