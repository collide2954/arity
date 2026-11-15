/**
 * Vega-Lite specifications for the figures. Each figure shows each report
 * as its own mark: arity does not combine, average, or weigh reports.
 */

import type { TopLevelSpec } from "vega-lite";

import type { Bin, Summary } from "./stats";

export type Mode = "light" | "dark";

/** The colors of the figures, from the validated reference palette. */
export const TOKENS = {
  light: {
    surface: "#fcfcfb",
    ink: "#0b0b0b",
    secondary: "#52514e",
    muted: "#898781",
    grid: "#e1e0d9",
    axis: "#c3c2b7",
    series: "#2a78d6",
    decrease: "#2a78d6",
    neutral: "#898781",
    increase: "#e34948",
  },
  dark: {
    surface: "#1a1a19",
    ink: "#f0efec",
    secondary: "#c3c2b7",
    muted: "#898781",
    grid: "#2c2c2a",
    axis: "#383835",
    series: "#3987e5",
    decrease: "#3987e5",
    neutral: "#5f5e5a",
    increase: "#e66767",
  },
} as const;

const FONT = 'system-ui, -apple-system, "Segoe UI", sans-serif';

export function config(mode: Mode): NonNullable<TopLevelSpec["config"]> {
  const tokens = TOKENS[mode];
  return {
    background: tokens.surface,
    font: FONT,
    view: { stroke: null },
    axis: {
      domainColor: tokens.axis,
      gridColor: tokens.grid,
      tickColor: tokens.axis,
      labelColor: tokens.muted,
      titleColor: tokens.secondary,
      labelFont: FONT,
      titleFont: FONT,
      titleFontWeight: "normal",
      labelFontSize: 12,
      titleFontSize: 12,
    },
    legend: {
      labelColor: tokens.secondary,
      titleColor: tokens.secondary,
      labelFont: FONT,
      titleFont: FONT,
      titleFontWeight: "normal",
    },
    title: { color: tokens.ink, font: FONT, fontWeight: "normal" },
  };
}

/** One reported value, as a row of the value figure. */
export interface ValueRow {
  report: number;
  label: string;
  value: number;
  low: number | null;
  high: number | null;
  reported: string;
  work: string;
  check: "unchecked" | "confirmed" | "rejected";
  origin: "stated" | "read from an image";
  notices: string;
  /** The notice sign of the source, or empty text. */
  marker: string;
  /** The subject and the conditions of the report. */
  detail: string;
  /** The place of the dot in its row, when a row has more than one dot. */
  slot: number;
}

/** The box of one row of the value figure: the quartiles, the median, and the extremes of its values. */
export interface Box extends Summary {
  readonly label: string;
}

export interface ValueOptions {
  unit: string;
  logarithmic: boolean;
  mode: Mode;
  /** True when a row of the figure can have more than one dot: the reports of a subject or of a work. */
  grouped: boolean;
  /** The largest width of a row label, in pixels. */
  labelLimit?: number;
  /** The box of each row that has one. The figure draws it behind the dots. */
  boxes?: readonly Box[];
}

/**
 * The layers of the boxes: a line from the smallest to the largest value, a
 * box from the first to the third quartile, and a line at the median.
 */
function boxLayers(boxes: readonly Box[], unit: string, mode: Mode, scale: object) {
  if (boxes.length === 0) return [];
  const tokens = TOKENS[mode];
  const data = { values: boxes.map((box) => ({ ...box })) };
  const format = ".4~g";
  return [
    {
      data,
      mark: { type: "rule" as const, color: tokens.muted, strokeWidth: 1 },
      encoding: {
        x: { field: "min", type: "quantitative" as const, scale },
        x2: { field: "max" },
      },
    },
    {
      data,
      mark: {
        type: "bar" as const,
        color: tokens.series,
        opacity: 0.14,
        stroke: tokens.series,
        strokeOpacity: 0.55,
        cornerRadius: 3,
        height: { band: 0.6 },
      },
      encoding: {
        x: { field: "q1", type: "quantitative" as const, scale },
        x2: { field: "q3" },
        tooltip: [
          { field: "count", title: "Values in the box" },
          { field: "min", title: `Smallest, in ${unit}`, format },
          { field: "q1", title: "First quartile", format },
          { field: "median", title: "Median", format },
          { field: "q3", title: "Third quartile", format },
          { field: "max", title: "Largest", format },
        ],
      },
    },
    {
      data,
      mark: { type: "tick" as const, color: tokens.series, thickness: 2 },
      encoding: { x: { field: "median", type: "quantitative" as const, scale } },
    },
  ];
}

/** A figure with more rows than this has the axis of values at the top. */
const TALL = 12;

/**
 * A dot for each reported value, with its uncertainty interval as a line and
 * the notice sign of its source above it. A person can drag a range along the
 * axis of values; the parameter `highlight` makes the dot of one report larger.
 */
export function valueSpec(rows: readonly ValueRow[], options: ValueOptions): TopLevelSpec {
  const tokens = TOKENS[options.mode];
  const scale = options.logarithmic ? { type: "log" as const } : { zero: false };
  const slots = Math.max(1, ...rows.map((row) => row.slot + 1));
  const tall = new Set(rows.map((row) => row.label)).size > TALL;
  const offset =
    options.grouped && slots > 1
      ? { yOffset: { field: "slot", type: "nominal" as const, legend: null } }
      : {};
  const intervals = rows.some((row) => row.low !== null && row.high !== null)
    ? [
        {
          mark: {
            type: "rule" as const,
            strokeWidth: 2,
            strokeCap: "round" as const,
            color: tokens.series,
          },
          encoding: {
            x: { field: "low", type: "quantitative" as const, scale },
            x2: { field: "high" },
            ...offset,
          },
        },
      ]
    : [];
  // The layers of the boxes have their own data, so a grouped figure names
  // the order of its rows: else the rows with a box would come first.
  const order = options.grouped
    ? { scale: { domain: [...new Set(rows.map((row) => row.label))] } }
    : {};
  return {
    $schema: "https://vega.github.io/schema/vega-lite/v6.json",
    width: "container",
    height: {
      step: options.grouped ? Math.min(64, Math.max(28, 10 + 9 * slots)) : 28,
      for: "position",
    },
    autosize: { type: "fit-x", contains: "padding" },
    data: { values: rows.map((row) => ({ ...row })) },
    config: config(options.mode),
    params: [{ name: "highlight", value: 0 }],
    encoding: {
      y: {
        field: "label",
        type: "nominal",
        sort: options.grouped ? null : { field: "value", order: "ascending" },
        axis: { title: null, labelLimit: options.labelLimit ?? 320 },
        ...order,
      },
    },
    layer: [
      ...boxLayers(options.boxes ?? [], options.unit, options.mode, scale),
      ...intervals,
      {
        mark: {
          type: "point",
          strokeWidth: 2,
          stroke: tokens.series,
          cursor: "pointer",
        },
        params: [
          { name: "hover", select: { type: "point", on: "pointerover", clear: "pointerout" } },
          { name: "range", select: { type: "interval", encodings: ["x"] } },
        ],
        encoding: {
          x: {
            field: "value",
            type: "quantitative",
            scale,
            title: options.unit,
            ...(tall ? { axis: { orient: "top" as const } } : {}),
          },
          fill: {
            condition: { test: "datum.check === 'confirmed'", value: tokens.series },
            value: tokens.surface,
          },
          shape: {
            field: "origin",
            type: "nominal",
            scale: { domain: ["stated", "read from an image"], range: ["circle", "diamond"] },
            legend: { title: "Value", orient: "top" },
          },
          size: { condition: { test: "datum.report === highlight", value: 320 }, value: 110 },
          opacity: {
            condition: [
              { test: "datum.check === 'rejected'", value: 0.35 },
              { param: "hover", empty: false, value: 1 },
            ],
            value: 0.9,
          },
          tooltip: [
            { field: "detail", title: "Report" },
            { field: "work", title: "Source" },
            { field: "reported", title: "As reported" },
            { field: "value", title: `In ${options.unit}`, format: ".4~g" },
            { field: "check", title: "Check" },
            { field: "notices", title: "Notices" },
          ],
          ...offset,
        },
      },
      {
        mark: { type: "text", dy: -12, fontSize: 12, color: tokens.ink },
        encoding: {
          x: { field: "value", type: "quantitative", scale },
          text: { field: "marker" },
          ...offset,
        },
      },
    ],
  };
}

export interface HistogramOptions {
  unit: string;
  logarithmic: boolean;
  mode: Mode;
}

/** A bar for each bin of the values: how many of the selected values are in the bin. */
export function histogramSpec(bins: readonly Bin[], options: HistogramOptions): TopLevelSpec {
  const tokens = TOKENS[options.mode];
  const scale = options.logarithmic ? { type: "log" as const } : { zero: false };
  return {
    $schema: "https://vega.github.io/schema/vega-lite/v6.json",
    width: "container",
    height: 120,
    autosize: { type: "fit-x", contains: "padding" },
    data: { values: bins.map((bin) => ({ ...bin })) },
    config: config(options.mode),
    mark: { type: "bar", color: tokens.series, opacity: 0.8 },
    encoding: {
      x: {
        field: "start",
        type: "quantitative",
        bin: { binned: true },
        scale,
        title: options.unit,
        axis: { labelOverlap: true, tickCount: 8 },
      },
      x2: { field: "end" },
      y: { field: "count", type: "quantitative", title: "Values" },
      tooltip: [
        { field: "start", title: "From", format: ".4~g" },
        { field: "end", title: "To", format: ".4~g" },
        { field: "count", title: "Values" },
      ],
    },
  };
}

/** One reported effect, as a mark of the effect matrix. */
export interface EffectRow {
  report: number;
  agent: string;
  target: string;
  /** The row of the mark in the tiles and the list. */
  group: string;
  direction: "increase" | "decrease" | "no_change" | "mixed";
  score: string;
  slot: number;
  work: string;
  detail: string;
  check: "unchecked" | "confirmed" | "rejected";
  notices: string;
  marker: string;
}

export const DIRECTIONS = ["increase", "no_change", "decrease", "mixed"] as const;

/** The names of the directions in a legend or on an axis. */
const DIRECTION_LABELS =
  "{'increase': 'Increase (+1)', 'no_change': 'No change (0)', 'decrease': 'Decrease (\u{2212}1)', 'mixed': 'Mixed'}[datum.label]";

/** The order of the columns of the list, from a decrease to an increase. */
const LIST_ORDER = ["decrease", "no_change", "increase", "mixed"] as const;

function directionColor(mode: Mode) {
  const tokens = TOKENS[mode];
  return {
    field: "direction",
    type: "nominal" as const,
    scale: {
      domain: [...DIRECTIONS],
      range: [tokens.increase, tokens.neutral, tokens.decrease, tokens.neutral],
    },
    legend: { title: "Direction", orient: "top" as const, labelExpr: DIRECTION_LABELS },
  };
}

const DIRECTION_SHAPES = {
  field: "direction",
  type: "nominal" as const,
  scale: { domain: [...DIRECTIONS], range: ["triangle-up", "circle", "triangle-down", "diamond"] },
  legend: null,
};

const EFFECT_TOOLTIP = [
  { field: "agent", title: "Agent" },
  { field: "target", title: "Target" },
  { field: "score", title: "Direction" },
  { field: "detail", title: "Details" },
  { field: "work", title: "Source" },
  { field: "check", title: "Check" },
  { field: "notices", title: "Notices" },
];

const DIM_REJECTED = {
  condition: { test: "datum.check === 'rejected'", value: 0.35 },
  value: 1,
};

/** A mark for each report, with its shape and color for its direction, and the notice sign above it. */
function effectLayers(mode: Mode) {
  const tokens = TOKENS[mode];
  return [
    {
      mark: {
        type: "point" as const,
        size: 120,
        strokeWidth: 2,
        filled: true,
        cursor: "pointer" as const,
      },
      encoding: {
        color: directionColor(mode),
        shape: DIRECTION_SHAPES,
        stroke: { value: tokens.surface },
        opacity: DIM_REJECTED,
        tooltip: EFFECT_TOOLTIP,
      },
    },
    {
      mark: {
        type: "text" as const,
        dx: 9,
        dy: -7,
        fontSize: 10,
        color: tokens.ink,
        cursor: "pointer" as const,
      },
      encoding: { text: { field: "marker" } },
    },
  ];
}

/**
 * A grid of agents and targets. Each report is its own mark in its cell: up
 * for an increase, down for a decrease, a dot for no change (0), and a
 * diamond for a mixed result. Each target has its own column, so the figure
 * is as wide as its targets need.
 */
export function effectSpec(rows: readonly EffectRow[], mode: Mode, labelLimit = 240): TopLevelSpec {
  return {
    $schema: "https://vega.github.io/schema/vega-lite/v6.json",
    width: { step: 30 },
    height: { step: 36 },
    data: { values: rows.map((row) => ({ ...row })) },
    config: config(mode),
    encoding: {
      x: {
        field: "target",
        type: "nominal",
        axis: {
          title: "Target",
          orient: "top",
          labelAngle: -40,
          labelAlign: "left",
          labelLimit: 160,
        },
      },
      y: { field: "agent", type: "nominal", axis: { title: "Agent", labelLimit } },
      xOffset: { field: "slot", type: "nominal" },
    },
    layer: effectLayers(mode),
  };
}

/** One row for each group, one column for each direction, and one mark for each report. */
export function effectListSpec(
  rows: readonly EffectRow[],
  mode: Mode,
  labelLimit = 240,
): TopLevelSpec {
  return {
    $schema: "https://vega.github.io/schema/vega-lite/v6.json",
    width: "container",
    height: { step: 26, for: "position" },
    autosize: { type: "fit-x", contains: "padding" },
    data: { values: rows.map((row) => ({ ...row })) },
    config: config(mode),
    encoding: {
      x: {
        field: "direction",
        type: "nominal",
        scale: { domain: [...LIST_ORDER] },
        axis: { title: null, orient: "top", labelAngle: 0, labelExpr: DIRECTION_LABELS },
      },
      y: { field: "group", type: "nominal", sort: "ascending", axis: { title: null, labelLimit } },
      xOffset: { field: "slot", type: "nominal" },
    },
    layer: effectLayers(mode),
  };
}

/**
 * One row for each group, with one square for each report in the order of
 * the directions. The length of a row shows its number of reports. A square
 * with a dark outline comes from a source with a notice.
 */
export function effectTilesSpec(
  rows: readonly EffectRow[],
  mode: Mode,
  labelLimit = 240,
): TopLevelSpec {
  const tokens = TOKENS[mode];
  return {
    $schema: "https://vega.github.io/schema/vega-lite/v6.json",
    width: { step: 14 },
    height: { step: 18 },
    data: { values: rows.map((row) => ({ ...row })) },
    config: config(mode),
    encoding: {
      x: { field: "slot", type: "ordinal", axis: null },
      y: {
        field: "group",
        type: "nominal",
        sort: { op: "count", order: "descending" },
        axis: { title: null, labelLimit },
      },
    },
    layer: [
      {
        mark: { type: "point", shape: "square", size: 110, filled: true, cursor: "pointer" },
        encoding: {
          color: directionColor(mode),
          stroke: {
            condition: { test: "datum.marker !== ''", value: tokens.ink },
            value: tokens.surface,
          },
          strokeWidth: { condition: { test: "datum.marker !== ''", value: 1.5 }, value: 1 },
          opacity: DIM_REJECTED,
          tooltip: EFFECT_TOOLTIP,
        },
      },
    ],
  };
}
