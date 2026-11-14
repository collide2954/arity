import { headline, type Names } from "./describe";
import type { Report } from "./types";

/** One quote of one report, to mark in the source. */
export interface Highlight {
  readonly report: number;
  readonly block: string;
  readonly quote: string;
  readonly start: number;
  readonly end: number;
  readonly label: string;
}

export function highlightsOf(reports: readonly Report[], lookup: Names): Highlight[] {
  return reports.flatMap((report) => {
    const label = headline(report, lookup);
    return report.evidence.map((evidence) => ({
      report: report.id,
      block: evidence.block,
      quote: evidence.quote,
      start: evidence.start,
      end: evidence.end,
      label,
    }));
  });
}

export interface Segment {
  readonly text: string;
  /** The reports whose quotes cover this segment. */
  readonly reports: readonly number[];
}

/**
 * Splits the text of a block at the edges of each highlight. The ranges count
 * Unicode code points, as the Rust checks record them.
 */
export function segments(
  text: string,
  ranges: readonly { start: number; end: number; report: number }[],
): Segment[] {
  const points = Array.from(text);
  const edges = new Set<number>([0, points.length]);
  for (const range of ranges) {
    edges.add(Math.max(0, Math.min(points.length, range.start)));
    edges.add(Math.max(0, Math.min(points.length, range.end)));
  }
  const sorted = [...edges].sort((a, b) => a - b);
  const result: Segment[] = [];
  for (let index = 0; index + 1 < sorted.length; index += 1) {
    const start = sorted[index] ?? 0;
    const end = sorted[index + 1] ?? 0;
    if (start === end) continue;
    const reports = ranges
      .filter((range) => range.start <= start && range.end >= end)
      .map((range) => range.report);
    result.push({ text: points.slice(start, end).join(""), reports: [...new Set(reports)] });
  }
  return result;
}
