import type { Coverage, CoverageCount } from "./types";

function part(count: CoverageCount, where: string): string {
  const skipped = count.skipped > 0 ? ` (${String(count.skipped)} skipped)` : "";
  return `${String(count.quoted)} of ${String(count.numbers)} numbers in ${where}${skipped}`;
}

/** How many numbers of a text the reports quote, as one sentence. A part with no numbers is left out. */
export function coverageLine(coverage: Coverage): string {
  const parts = [
    coverage.cells.numbers > 0 ? part(coverage.cells, "table cells") : null,
    coverage.text.numbers > 0 ? part(coverage.text, "the text") : null,
  ].filter((found) => found !== null);
  if (parts.length === 0) return "The text has no numbers for reports to quote.";
  return `Reports quote ${parts.join(", and ")}.`;
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
