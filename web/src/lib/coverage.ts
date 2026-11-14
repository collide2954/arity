import type { Coverage, CoverageCount } from "./types";

function part(count: CoverageCount, where: string): string {
  const skipped = count.skipped > 0 ? ` (${String(count.skipped)} skipped)` : "";
  return `${String(count.quoted)} of ${String(count.numbers)} numbers in ${where}${skipped}`;
}

/** How many numbers of a text the reports quote, as one sentence. */
export function coverageLine(coverage: Coverage): string {
  return `Reports quote ${part(coverage.cells, "table cells")}, and ${part(coverage.text, "the text")}.`;
}

/** The ranges of the numbers of one block that no report quotes. */
export function unquotedOf(
  coverage: Coverage | null,
  block: string,
): { start: number; end: number }[] {
  return (coverage?.unquoted ?? [])
    .filter((unquoted) => unquoted.block === block)
    .map(({ start, end }) => ({ start, end }));
}
