import { headline, type Names } from "./describe";
import type { Report } from "./types";

/** One quote of one report, to mark in the source. */
export interface Highlight {
  readonly report: number;
  /** The position of the quote in the evidence of its report. 0 is the main quote. */
  readonly index: number;
  readonly block: string;
  readonly quote: string;
  readonly start: number;
  readonly end: number;
  readonly label: string;
}

export function highlightsOf(reports: readonly Report[], lookup: Names): Highlight[] {
  return reports.flatMap((report) => {
    const label = headline(report, lookup);
    return report.evidence.map((evidence, index) => ({
      report: report.id,
      index,
      block: evidence.block,
      quote: evidence.quote,
      start: evidence.start,
      end: evidence.end,
      label,
    }));
  });
}

/**
 * Scrolls a container so that an element inside it is in the middle of the
 * view. Only the container moves, not the page.
 */
export function reveal(container: HTMLElement, element: HTMLElement): void {
  const offset =
    element.getBoundingClientRect().top -
    container.getBoundingClientRect().top +
    container.scrollTop;
  container.scrollTo({ top: offset - container.clientHeight / 2, behavior: "smooth" });
}

/** The mark of the main quote of a report, or its first mark. */
export function mainMark(container: HTMLElement, report: number): HTMLElement | null {
  const marks = [...container.querySelectorAll<HTMLElement>(`[data-report="${String(report)}"]`)];
  marks.sort(
    (a, b) => Number(a.dataset.evidence ?? Infinity) - Number(b.dataset.evidence ?? Infinity),
  );
  return marks[0] ?? null;
}

/** The highlights of each block. */
export function byBlock(highlights: readonly Highlight[]): Map<string, Highlight[]> {
  const blocks = new Map<string, Highlight[]>();
  for (const highlight of highlights) {
    const list = blocks.get(highlight.block);
    if (list) list.push(highlight);
    else blocks.set(highlight.block, [highlight]);
  }
  return blocks;
}

export interface Segment {
  readonly text: string;
  /** The reports whose quotes cover this segment. */
  readonly reports: readonly number[];
  /** True for a number that no report quotes and no reader skipped. */
  readonly unquoted?: true;
}

/**
 * Splits the text of a block at the edges of each highlight. The ranges count
 * Unicode code points, as the Rust checks record them.
 */
export function segments(
  text: string,
  ranges: readonly { start: number; end: number; report: number }[],
  unquoted: readonly { start: number; end: number }[] = [],
): Segment[] {
  const points = Array.from(text);
  const edges = new Set<number>([0, points.length]);
  for (const range of [...ranges, ...unquoted]) {
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
    const segment = { text: points.slice(start, end).join(""), reports: [...new Set(reports)] };
    const isUnquoted =
      reports.length === 0 && unquoted.some((range) => range.start <= start && range.end >= end);
    result.push(isUnquoted ? { ...segment, unquoted: true } : segment);
  }
  return result;
}
