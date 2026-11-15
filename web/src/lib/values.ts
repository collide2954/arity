import type { ValueRow } from "./charts";
import { markerSign, noticeMarker, noticeNames } from "./notices";
import type { ReportRow } from "./reports";
import { compareValues } from "./sort";

/** What one row of the figure of a property shows: one report, or all reports of a subject or of a work. */
export type FigureRows = "report" | "subject" | "work";

export const FIGURE_ROWS: readonly FigureRows[] = ["report", "subject", "work"];

export interface FigureOptions {
  readonly rows: FigureRows;
  readonly logarithmic: boolean;
}

/** The settings of the figure that the address keeps. No rows means the default rows. */
export interface FigureView {
  readonly rows: FigureRows | null;
  readonly logarithmic: boolean;
}

/** The largest number of rows that the figure shows by default. */
const FEW_REPORTS = 60;
const FEW_SUBJECTS = 40;

function isFigureRows(text: string | null): text is FigureRows {
  return (FIGURE_ROWS as readonly (string | null)[]).includes(text);
}

/** Reads the settings of the figure from the parameters `rows` and `scale` of an address. */
export function readFigure(params: URLSearchParams): FigureView {
  const rows = params.get("rows");
  return { rows: isFigureRows(rows) ? rows : null, logarithmic: params.get("scale") === "log" };
}

export function writeFigure(params: URLSearchParams, figure: FigureView): void {
  if (figure.rows !== null) params.set("rows", figure.rows);
  if (figure.logarithmic) params.set("scale", "log");
}

/**
 * The rows of the figure: those of the settings, or by default one row for
 * each report when there are few reports, one row for each subject when there
 * are few subjects, and else one row for each work.
 */
export function rowsOfFigure(
  figure: FigureView,
  counts: { readonly reports: number; readonly subjects: number },
): FigureRows {
  if (figure.rows !== null) return figure.rows;
  if (counts.reports <= FEW_REPORTS) return "report";
  return counts.subjects <= FEW_SUBJECTS ? "subject" : "work";
}

/** Two dots of one row are near when they are closer than this part of the width of the figure. */
const NEAR = 0.012;

function withNotices(label: string, notices: readonly string[]): string {
  return notices.length > 0 ? `${noticeMarker(notices)} \u{b7} ${label}` : label;
}

/**
 * The dots of the figure of a property: one dot for each report with a value
 * in the unit of the property. Each report keeps its own dot. In a row of a
 * subject or of a work, a dot that is near another dot goes into the next slot.
 */
export function figureRows(rows: readonly ReportRow[], options: FigureOptions): ValueRow[] {
  const counts = new Map<string, number>();
  const dots = rows.flatMap((row): ValueRow[] => {
    const display = row.report.display;
    if (display?.value === null || display?.value === undefined) return [];
    const detail = [row.subject, ...row.conditions].join(" \u{b7} ");
    let label: string;
    if (options.rows === "subject") {
      label = row.subject;
    } else if (options.rows === "work") {
      label = withNotices(row.citation, row.notices);
    } else {
      const base = `${detail} \u{2014} ${row.citation}`;
      const count = (counts.get(base) ?? 0) + 1;
      counts.set(base, count);
      label = withNotices(count === 1 ? base : `${base} (${String(count)})`, row.notices);
    }
    return [
      {
        report: row.report.id,
        label,
        value: display.value,
        low: display.interval_low,
        high: display.interval_high,
        reported: row.reported,
        work: row.citation,
        check: row.report.check,
        origin: row.values.origin?.includes("image") ? "read from an image" : "stated",
        notices: noticeNames(row.notices) || "none",
        marker: markerSign(row.notices),
        detail,
        slot: 0,
      },
    ];
  });
  if (options.rows === "report") return dots;
  return withSlots(
    dots.sort((a, b) => compareValues(a.label, b.label)),
    options.logarithmic ? Math.log10 : (value: number) => value,
  );
}

function withSlots(dots: readonly ValueRow[], position: (value: number) => number): ValueRow[] {
  const places = dots.flatMap((dot) =>
    [dot.value, dot.low, dot.high].flatMap((value) => (value === null ? [] : [position(value)])),
  );
  const span = Math.max(...places) - Math.min(...places);
  const gap = span > 0 ? span * NEAR : 1;
  const groups = new Map<string, ValueRow[]>();
  for (const dot of dots) {
    const group = groups.get(dot.label);
    if (group) group.push(dot);
    else groups.set(dot.label, [dot]);
  }
  const slots = new Map<ValueRow, number>();
  for (const members of groups.values()) {
    const ends: number[] = [];
    for (const dot of [...members].sort((a, b) => a.value - b.value)) {
      const at = position(dot.value);
      let slot = ends.findIndex((end) => at - end > gap);
      if (slot < 0) slot = ends.length;
      ends[slot] = at;
      slots.set(dot, slot);
    }
  }
  return dots.map((dot) => ({ ...dot, slot: slots.get(dot) ?? 0 }));
}
