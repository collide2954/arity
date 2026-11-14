import { readFileSync } from "node:fs";
import { join } from "node:path";

import { describe, expect, it } from "vitest";

import { find, normalize } from "./normalize";

interface Vectors {
  normalize: { input: string; output: string }[];
  find: { text: string; quote: string; ranges: [number, number][] }[];
}

const vectors = JSON.parse(
  readFileSync(join(process.cwd(), "..", "testdata", "text", "normalization.json"), "utf8"),
) as Vectors;

describe("normalize", () => {
  it.each(vectors.normalize)("follows the shared vector for $input", ({ input, output }) => {
    expect(normalize(input).text).toBe(output);
  });

  it("keeps one origin for each code point", () => {
    const result = normalize("a\u{a0} b\u{fb01}");
    expect(Array.from(result.text)).toHaveLength(result.origins.length);
    expect(result.origins).toEqual([0, 1, 3, 4, 4]);
  });
});

describe("find", () => {
  it.each(vectors.find)("follows the shared vector for $quote", ({ text, quote, ranges }) => {
    expect(find(normalize(text), quote)).toEqual(ranges);
  });
});
