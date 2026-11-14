import { describe, expect, it } from "vitest";

import { TOKENS, config, effectSpec, valueSpec, type EffectRow, type ValueRow } from "./charts";

const value: ValueRow = {
  report: 1,
  label: "Lafi et al. 2023 \u{b7} doxorubicin",
  value: 0.24,
  low: 0.22,
  high: 0.26,
  reported: "0.24 \u{b1} 0.02 \u{3bc}M",
  work: "Lafi et al. 2023, PLOS ONE",
  check: "unchecked",
  origin: "stated",
  notices: "",
};

const effect: EffectRow = {
  report: 2,
  agent: "hydralazine",
  target: "MCF-7 \u{b7} cell viability",
  direction: "no_change",
  score: "0",
  slot: 0,
  work: "Lafi et al. 2023, PLOS ONE",
  detail: "p = 0.34",
  check: "confirmed",
};

describe("config", () => {
  it("uses the surface and the ink of each mode", () => {
    expect(config("light").background).toBe(TOKENS.light.surface);
    expect(config("dark").background).toBe(TOKENS.dark.surface);
    expect(config("dark").axis?.labelColor).toBe(TOKENS.dark.muted);
  });
});

describe("valueSpec", () => {
  it("draws one dot and one interval for each report", () => {
    const spec = valueSpec([value], { unit: "\u{b5}mol/L", logarithmic: false, mode: "light" });
    expect(spec.data).toEqual({ values: [value] });
    const layers = (spec as { layer: { mark: { type: string } }[] }).layer;
    expect(layers.map((layer) => layer.mark.type)).toEqual(["rule", "point"]);
  });

  it("uses a logarithmic scale on request", () => {
    const spec = JSON.stringify(
      valueSpec([value], { unit: "\u{b5}mol/L", logarithmic: true, mode: "light" }),
    );
    expect(spec).toContain('"type":"log"');
    expect(spec).not.toContain('"zero":false');
  });

  it("fills only the dots of confirmed reports", () => {
    const spec = JSON.stringify(
      valueSpec([value], { unit: "\u{b5}mol/L", logarithmic: false, mode: "light" }),
    );
    expect(spec).toContain("datum.check === 'confirmed'");
  });
});

describe("effectSpec", () => {
  it("shows no change with the neutral color and a dot", () => {
    const spec = effectSpec([effect], "light") as unknown as {
      encoding: {
        color: { scale: { domain: string[]; range: string[] } };
        shape: { scale: { domain: string[]; range: string[] } };
      };
    };
    const colors = spec.encoding.color.scale;
    const shapes = spec.encoding.shape.scale;
    const index = colors.domain.indexOf("no_change");
    expect(colors.range[index]).toBe(TOKENS.light.neutral);
    expect(shapes.range[shapes.domain.indexOf("no_change")]).toBe("circle");
    expect(shapes.range[shapes.domain.indexOf("increase")]).toBe("triangle-up");
    expect(shapes.range[shapes.domain.indexOf("decrease")]).toBe("triangle-down");
  });

  it("keeps each report as its own mark", () => {
    const second = { ...effect, report: 3, slot: 1, direction: "decrease" as const };
    const spec = effectSpec([effect, second], "dark");
    expect((spec.data as { values: unknown[] }).values).toHaveLength(2);
  });
});
