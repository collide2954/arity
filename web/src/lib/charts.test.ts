import { describe, expect, it } from "vitest";

import {
  TOKENS,
  config,
  effectListSpec,
  effectSpec,
  effectTilesSpec,
  histogramSpec,
  valueSpec,
  type EffectRow,
  type ValueRow,
} from "./charts";

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
  marker: "",
  detail: "doxorubicin",
  slot: 0,
};

const options = { unit: "\u{b5}mol/L", logarithmic: false, mode: "light", grouped: false } as const;

const effect: EffectRow = {
  report: 2,
  agent: "hydralazine",
  target: "MCF-7 \u{b7} cell viability",
  group: "hydralazine",
  direction: "no_change",
  score: "0",
  slot: 0,
  work: "Lafi et al. 2023, PLOS ONE",
  detail: "p = 0.34",
  check: "confirmed",
  notices: "none",
  marker: "",
};

interface EffectLayers {
  layer: {
    mark: { type: string };
    encoding: {
      color?: { scale: { domain: string[]; range: string[] } };
      shape?: { scale: { domain: string[]; range: string[] } };
      text?: { field: string };
    };
  }[];
}

describe("config", () => {
  it("uses the surface and the ink of each mode", () => {
    expect(config("light").background).toBe(TOKENS.light.surface);
    expect(config("dark").background).toBe(TOKENS.dark.surface);
    expect(config("dark").axis?.labelColor).toBe(TOKENS.dark.muted);
  });
});

describe("valueSpec", () => {
  it("draws one dot, one interval, and one notice sign for each report", () => {
    const spec = valueSpec([value], options);
    expect(spec.data).toEqual({ values: [value] });
    const layers = (spec as { layer: { mark: { type: string } }[] }).layer;
    expect(layers.map((layer) => layer.mark.type)).toEqual(["rule", "point", "text"]);
  });

  it("leaves out the layer of intervals when no report has an interval", () => {
    const spec = valueSpec([{ ...value, low: null, high: null }], options);
    const layers = (spec as { layer: { mark: { type: string } }[] }).layer;
    expect(layers.map((layer) => layer.mark.type)).toEqual(["point", "text"]);
    expect(spec.autosize).toEqual({ type: "fit-x", contains: "padding" });
  });

  it("uses a logarithmic scale on request", () => {
    const spec = JSON.stringify(valueSpec([value], { ...options, logarithmic: true }));
    expect(spec).toContain('"type":"log"');
    expect(spec).not.toContain('"zero":false');
  });

  it("fills only the dots of confirmed reports, and makes the dots of rejected reports pale", () => {
    const spec = JSON.stringify(valueSpec([value], options));
    expect(spec).toContain("datum.check === 'confirmed'");
    expect(spec).toContain("datum.check === 'rejected'");
  });

  it("gives each slot of a row its own place when a row has more than one dot", () => {
    const second = { ...value, report: 2, slot: 1 };
    expect(JSON.stringify(valueSpec([value, second], { ...options, grouped: true }))).toContain(
      '"yOffset":{"field":"slot"',
    );
    expect(JSON.stringify(valueSpec([value], { ...options, grouped: true }))).not.toContain(
      "yOffset",
    );
  });

  it("puts the axis of values at the top of a tall figure, and limits the width of the labels", () => {
    const many = Array.from({ length: 13 }, (_, position) => ({
      ...value,
      report: position,
      label: `row ${String(position)}`,
    }));
    expect(JSON.stringify(valueSpec(many, { ...options, labelLimit: 150 }))).toContain(
      '"orient":"top"',
    );
    expect(JSON.stringify(valueSpec([value], { ...options, labelLimit: 150 }))).toContain(
      '"labelLimit":150',
    );
    expect(JSON.stringify(valueSpec(many.slice(0, 12), options))).not.toContain(
      '"axis":{"orient":"top"',
    );
  });

  it("draws a box behind the dots of each row that has one, and keeps the order of the rows", () => {
    const box = { label: "b", count: 5, min: 0.1, q1: 0.2, median: 0.24, q3: 0.3, max: 0.4 };
    const rows = [
      { ...value, label: "a" },
      { ...value, report: 2, label: "b" },
    ];
    const spec = valueSpec(rows, { ...options, grouped: true, boxes: [box] });
    const layers = (spec as { layer: { mark: { type: string }; data?: unknown }[] }).layer;
    expect(layers.map((layer) => layer.mark.type)).toEqual([
      "rule",
      "bar",
      "tick",
      "rule",
      "point",
      "text",
    ]);
    expect(layers[1]?.data).toEqual({ values: [box] });
    expect(JSON.stringify(spec)).toContain('"domain":["a","b"]');
  });

  it("lets a person drag a range along the axis of values, and makes a highlighted dot larger", () => {
    const spec = valueSpec([value], options);
    expect(JSON.stringify(spec)).toContain(
      '{"name":"range","select":{"type":"interval","encodings":["x"]}}',
    );
    expect(spec.params).toEqual([{ name: "highlight", value: 0 }]);
    expect(JSON.stringify(spec)).toContain("datum.report === highlight");
  });
});

describe("histogramSpec", () => {
  it("draws one bar for each bin", () => {
    const bins = [
      { start: 0, end: 2, count: 2 },
      { start: 2, end: 4, count: 1 },
    ];
    const spec = histogramSpec(bins, { unit: "\u{b5}mol/L", logarithmic: false, mode: "light" });
    expect(spec.data).toEqual({ values: bins });
    expect(JSON.stringify(spec)).toContain('"type":"bar"');
    expect(JSON.stringify(spec)).toContain('"bin":{"binned":true}');
    expect(JSON.stringify(histogramSpec(bins, { ...options, logarithmic: true }))).toContain(
      '"type":"log"',
    );
  });
});

describe("effectSpec", () => {
  it("shows no change with the neutral color and a dot", () => {
    const [marks] = (effectSpec([effect], "light") as unknown as EffectLayers).layer;
    const colors = marks?.encoding.color?.scale;
    const shapes = marks?.encoding.shape?.scale;
    expect(colors?.range[colors.domain.indexOf("no_change")]).toBe(TOKENS.light.neutral);
    expect(shapes?.range[shapes.domain.indexOf("no_change")]).toBe("circle");
    expect(shapes?.range[shapes.domain.indexOf("increase")]).toBe("triangle-up");
    expect(shapes?.range[shapes.domain.indexOf("decrease")]).toBe("triangle-down");
  });

  it("puts the notice sign of the source above each mark", () => {
    const [, signs] = (effectSpec([effect], "light") as unknown as EffectLayers).layer;
    expect(signs?.mark.type).toBe("text");
    expect(signs?.encoding.text).toEqual({ field: "marker" });
  });

  it("keeps each report as its own mark", () => {
    const second = { ...effect, report: 3, slot: 1, direction: "decrease" as const };
    const spec = effectSpec([effect, second], "dark");
    expect((spec.data as { values: unknown[] }).values).toHaveLength(2);
  });

  it("turns the labels of the targets and gives each target its own width", () => {
    const spec = JSON.stringify(effectSpec([effect], "light", 200));
    expect(spec).toContain('"labelAngle":-40');
    expect(spec).toContain('"width":{"step":');
    expect(spec).toContain('"labelLimit":200');
  });
});

describe("effectListSpec", () => {
  it("gives each row one column for each direction, and each report its own mark", () => {
    const spec = effectListSpec([effect], "light", 200) as unknown as {
      encoding: {
        x: { field: string; scale: { domain: string[] } };
        y: { field: string };
        xOffset: { field: string };
      };
    };
    expect(spec.encoding.x.field).toBe("direction");
    expect(spec.encoding.x.scale.domain).toEqual(["decrease", "no_change", "increase", "mixed"]);
    expect(spec.encoding.y.field).toBe("group");
    expect(spec.encoding.xOffset.field).toBe("slot");
  });
});

describe("effectTilesSpec", () => {
  it("gives each report one square in its row, with the color of its direction", () => {
    const spec = effectTilesSpec([effect], "light", 200) as unknown as {
      encoding: { x: { field: string }; y: { field: string } };
      layer: { mark: { type: string; shape?: string } }[];
    };
    expect(spec.encoding.x.field).toBe("slot");
    expect(spec.encoding.y.field).toBe("group");
    expect(spec.layer[0]?.mark).toMatchObject({ type: "point", shape: "square" });
  });
});
