/**
 * Vega-Lite specifications for the figures. Each figure shows each report
 * as its own mark: arity does not combine, average, or weigh reports.
 */

import type { TopLevelSpec } from "vega-lite";

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

export interface ValueOptions {
  unit: string;
  logarithmic: boolean;
  mode: Mode;
  /** True when a row of the figure can have more than one dot: the reports of a subject or of a work. */
  grouped: boolean;
  /** The largest width of a row label, in pixels. */
  labelLimit?: number;
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
          },
        },
      ]
    : [];
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
      },
      ...offset,
    },
    layer: [
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
        },
      },
      {
        mark: { type: "text", dy: -12, fontSize: 12, color: tokens.ink },
        encoding: {
          x: { field: "value", type: "quantitative", scale },
          text: { field: "marker" },
        },
      },
    ],
  };
}

/** One reported effect, as a mark of the effect matrix. */
export interface EffectRow {
  report: number;
  agent: string;
  target: string;
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

/**
 * A grid of agents and targets. Each report is its own mark in its cell: up
 * for an increase, down for a decrease, a dot for no change (0), and a
 * diamond for a mixed result. A mark whose source has notices has the notice
 * sign above it.
 */
export function effectSpec(rows: readonly EffectRow[], mode: Mode): TopLevelSpec {
  const tokens = TOKENS[mode];
  const agents = new Set(rows.map((row) => row.agent)).size;
  return {
    $schema: "https://vega.github.io/schema/vega-lite/v6.json",
    width: "container",
    height: Math.max(120, agents * 44),
    data: { values: rows.map((row) => ({ ...row })) },
    config: config(mode),
    encoding: {
      x: {
        field: "target",
        type: "nominal",
        axis: { title: "Target", labelAngle: 0, labelLimit: 180 },
      },
      y: { field: "agent", type: "nominal", axis: { title: "Agent", labelLimit: 240 } },
      xOffset: { field: "slot", type: "nominal" },
    },
    layer: [
      {
        mark: { type: "point", size: 140, strokeWidth: 2, filled: true, cursor: "pointer" },
        encoding: {
          color: {
            field: "direction",
            type: "nominal",
            scale: {
              domain: [...DIRECTIONS],
              range: [tokens.increase, tokens.neutral, tokens.decrease, tokens.neutral],
            },
            legend: {
              title: "Direction",
              labelExpr:
                "{'increase': 'Increase (+1)', 'no_change': 'No change (0)', 'decrease': 'Decrease (\u{2212}1)', 'mixed': 'Mixed'}[datum.label]",
            },
          },
          shape: {
            field: "direction",
            type: "nominal",
            scale: {
              domain: [...DIRECTIONS],
              range: ["triangle-up", "circle", "triangle-down", "diamond"],
            },
            legend: null,
          },
          stroke: { value: tokens.surface },
          opacity: { condition: { test: "datum.check === 'rejected'", value: 0.35 }, value: 1 },
          tooltip: [
            { field: "agent", title: "Agent" },
            { field: "target", title: "Target" },
            { field: "score", title: "Direction" },
            { field: "detail", title: "Details" },
            { field: "work", title: "Source" },
            { field: "check", title: "Check" },
            { field: "notices", title: "Notices" },
          ],
        },
      },
      {
        mark: { type: "text", dy: -16, fontSize: 13, color: tokens.ink, cursor: "pointer" },
        encoding: { text: { field: "marker" } },
      },
    ],
  };
}
