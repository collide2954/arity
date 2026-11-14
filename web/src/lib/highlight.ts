import type { Match, Quote } from "./anchor";
import { headline, type Names } from "./describe";
import type { Block, Report } from "./types";

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

/** A part of the document that one or more reports quote. */
export interface Quoted {
  readonly quote: Quote;
  /** Each report that quotes it, one time. */
  readonly reports: readonly number[];
  /** The reports for which it is the main quote. */
  readonly main: readonly number[];
}

/** The number of UTF-16 code units of text around a quote to compare with the PDF. */
const AROUND = 100;

/** The UTF-16 offset of a code point index in a text. */
function unitOffset(text: string, point: number): number {
  let offset = 0;
  let count = 0;
  for (const character of text) {
    if (count === point) break;
    offset += character.length;
    count += 1;
  }
  return offset;
}

/**
 * Groups the highlights by the part of the document that they quote, and adds
 * the text around each part. The blocks are in the order of the document.
 */
export function quotedParts(blocks: readonly Block[], highlights: readonly Highlight[]): Quoted[] {
  const offsets = new Map<string, { offset: number; text: string }>();
  let flow = "";
  for (const block of blocks) {
    if (flow !== "") flow += " ";
    offsets.set(block.key, { offset: flow.length, text: block.text });
    flow += block.text;
  }
  const parts = new Map<string, { quote: Quote; reports: Set<number>; main: Set<number> }>();
  for (const highlight of highlights) {
    const key = [highlight.block, highlight.start, highlight.end].join(" ");
    let part = parts.get(key);
    if (!part) {
      part = {
        quote: quoteOf(flow, offsets.get(highlight.block), highlight),
        reports: new Set(),
        main: new Set(),
      };
      parts.set(key, part);
    }
    part.reports.add(highlight.report);
    if (highlight.index === 0) part.main.add(highlight.report);
  }
  return [...parts.values()].map(({ quote, reports, main }) => ({
    quote,
    reports: [...reports],
    main: [...main],
  }));
}

function quoteOf(
  flow: string,
  block: { offset: number; text: string } | undefined,
  highlight: Highlight,
): Quote {
  if (!block) return { text: highlight.quote, before: "", after: "", at: 0 };
  const start = block.offset + unitOffset(block.text, highlight.start);
  const end = block.offset + unitOffset(block.text, highlight.end);
  return {
    text: highlight.quote,
    before: flow.slice(Math.max(0, start - AROUND), start),
    after: flow.slice(end, end + AROUND),
    at: flow === "" ? 0 : start / flow.length,
  };
}

/** A mark on a page of the PDF, with each report that quotes the text under it. */
export interface Mark {
  readonly match: Match;
  readonly reports: readonly number[];
  /** The reports for which it is the main quote. */
  readonly main: readonly number[];
}

export interface Placed {
  /** The marks on each page, by the index of the page. */
  readonly marks: ReadonlyMap<number, readonly Mark[]>;
  /** The page of the main mark of each report that has a mark. */
  readonly pages: ReadonlyMap<number, number>;
}

/**
 * Finds each quoted part in the PDF, and gives one mark for each place. The
 * main mark of a report is the mark of its main quote, or else its first mark.
 */
export function place(parts: readonly Quoted[], find: (quote: Quote) => Match | null): Placed {
  const marks = new Map<string, { match: Match; reports: Set<number>; main: Set<number> }>();
  const pages = new Map<number, number>();
  for (const part of parts) {
    const match = find(part.quote);
    if (!match) continue;
    const { page, start, end } = match;
    const key = [page, start.item, start.offset, end.item, end.offset].join(" ");
    let mark = marks.get(key);
    if (!mark) {
      mark = { match, reports: new Set(), main: new Set() };
      marks.set(key, mark);
    }
    for (const report of part.reports) {
      mark.reports.add(report);
      if (part.main.includes(report) || !pages.has(report)) pages.set(report, page);
    }
    for (const report of part.main) mark.main.add(report);
  }
  const byPage = new Map<number, Mark[]>();
  for (const { match, reports, main } of marks.values()) {
    const mark = { match, reports: [...reports], main: [...main] };
    const list = byPage.get(match.page);
    if (list) list.push(mark);
    else byPage.set(match.page, [mark]);
  }
  return { marks: byPage, pages };
}

/** The label of each report. */
export function labelsOf(highlights: readonly Highlight[]): Map<number, string> {
  return new Map(highlights.map((highlight) => [highlight.report, highlight.label]));
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

/** The mark of the main quote of a report, or else its first mark. */
export function mainMark(container: HTMLElement, report: number): HTMLElement | null {
  const id = String(report);
  return (
    container.querySelector<HTMLElement>(`[data-main~="${id}"]`) ??
    container.querySelector<HTMLElement>(`[data-reports~="${id}"]`)
  );
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
