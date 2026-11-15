/**
 * Statistics that arity calculates from the values of a selection of
 * reports. They describe the selection; they are not a consensus value.
 */

export interface Summary {
  readonly count: number;
  readonly min: number;
  readonly q1: number;
  readonly median: number;
  readonly q3: number;
  readonly max: number;
}

/** The quantile of sorted values, with linear interpolation between two values. */
function quantile(sorted: readonly number[], part: number): number {
  const position = (sorted.length - 1) * part;
  const low = Math.floor(position);
  const below = sorted[low] ?? 0;
  const above = sorted[Math.min(low + 1, sorted.length - 1)] ?? below;
  return below + (position - low) * (above - below);
}

export function summary(values: readonly number[]): Summary | null {
  if (values.length === 0) return null;
  const sorted = [...values].sort((a, b) => a - b);
  return {
    count: sorted.length,
    min: sorted[0] ?? 0,
    q1: quantile(sorted, 0.25),
    median: quantile(sorted, 0.5),
    q3: quantile(sorted, 0.75),
    max: sorted[sorted.length - 1] ?? 0,
  };
}

/** A bin of a histogram: the values from its start, up to its end. The last bin also holds its end. */
export interface Bin {
  readonly start: number;
  readonly end: number;
  readonly count: number;
}

/**
 * The bins of the values, each of the same width. On a logarithmic scale the
 * bins have the same width in powers of ten, and values that are not positive
 * are not counted. Without a number of bins, the number grows with the number
 * of values, from 5 to 30.
 */
export function histogram(values: readonly number[], logarithmic: boolean, bins?: number): Bin[] {
  const usable = logarithmic ? values.filter((value) => value > 0) : values;
  if (usable.length === 0) return [];
  const forward = logarithmic ? Math.log10 : (value: number) => value;
  const back = logarithmic ? (value: number) => 10 ** value : (value: number) => value;
  const positions = usable.map(forward);
  const low = Math.min(...positions);
  const high = Math.max(...positions);
  if (low === high) return [{ start: back(low), end: back(high), count: usable.length }];
  const count = bins ?? Math.min(30, Math.max(5, Math.ceil(Math.log2(usable.length)) + 1));
  const width = (high - low) / count;
  const counts: number[] = Array.from({ length: count }, () => 0);
  for (const position of positions) {
    const bin = Math.min(count - 1, Math.floor((position - low) / width));
    counts[bin] = (counts[bin] ?? 0) + 1;
  }
  return counts.map((found, bin) => ({
    start: back(low + bin * width),
    end: back(low + (bin + 1) * width),
    count: found,
  }));
}
