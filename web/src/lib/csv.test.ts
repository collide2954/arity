import { describe, expect, it } from "vitest";

import { toCsv } from "./csv";

describe("toCsv", () => {
  it("writes a header and rows", () => {
    expect(toCsv(["a", "b"], [[1, "x"]])).toBe("a,b\r\n1,x\r\n");
  });

  it("quotes fields with commas, quotes, or line breaks", () => {
    expect(toCsv(["value"], [["7.30 \u{b1} 0.84, n = 3"], ['say "no"'], ["two\nlines"]])).toBe(
      'value\r\n"7.30 \u{b1} 0.84, n = 3"\r\n"say ""no"""\r\n"two\nlines"\r\n',
    );
  });

  it("writes empty fields for missing values", () => {
    expect(toCsv(["a", "b", "c"], [[null, undefined, false]])).toBe("a,b,c\r\n,,false\r\n");
  });
});
