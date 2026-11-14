import type { EffectBody, ValueInput } from "./types";

const SUPERSCRIPTS: Record<string, string> = {
  "0": "\u{2070}",
  "1": "\u{b9}",
  "2": "\u{b2}",
  "3": "\u{b3}",
  "4": "\u{2074}",
  "5": "\u{2075}",
  "6": "\u{2076}",
  "7": "\u{2077}",
  "8": "\u{2078}",
  "9": "\u{2079}",
  "-": "\u{207b}",
  "+": "",
};

function superscript(digits: string): string {
  return Array.from(digits, (character) => SUPERSCRIPTS[character] ?? character).join("");
}

const ATOMS: Record<string, string> = {
  Cel: "\u{b0}C",
  "[degF]": "\u{b0}F",
  Ao: "\u{c5}",
  Ohm: "\u{3a9}",
  "[iU]": "IU",
  "[IU]": "IU",
  "[arb'U]": "arb. unit",
  "[CFU]": "CFU",
  "[PFU]": "PFU",
  "[ppm]": "ppm",
  "[ppb]": "ppb",
  "[ppth]": "\u{2030}",
  "[g]": "\u{d7}g",
  "[psi]": "psi",
  "m[Hg]": "mHg",
  deg: "\u{b0}",
};

function symbol(token: string): string {
  const annotation = /^\{(.*)\}$/.exec(token);
  if (annotation) return annotation[1] ?? "";
  const power = /^10\*([+-]?\d+)$/.exec(token);
  if (power) return `10${superscript(power[1] ?? "")}`;
  const match = /^(.*?)([+-]?\d+)?$/.exec(token);
  const base = match?.[1] ?? token;
  const exponent = match?.[2];
  let label = ATOMS[base] ?? base;
  if (ATOMS[base] === undefined && /^u[A-Za-z[]/.test(base)) {
    const rest = base.slice(1);
    label = `\u{b5}${ATOMS[rest] ?? rest}`;
  } else if (ATOMS[base] === undefined) {
    for (const [atom, text] of Object.entries(ATOMS)) {
      if (base.endsWith(atom) && base.length > atom.length && base.length - atom.length <= 2) {
        label = base.slice(0, base.length - atom.length) + text;
        break;
      }
    }
  }
  return exponent === undefined ? label : label + superscript(exponent);
}

/** Writes a UCUM code in the form that people read, for example `umol/L` as `µmol/L`. */
export function unitLabel(code: string): string {
  if (code === "1") return "";
  const parts = code.split(/([./()])/);
  return parts
    .map((part) => {
      if (part === ".") return "\u{b7}";
      if (part === "/" || part === "(" || part === ")" || part === "") return part;
      return symbol(part);
    })
    .join("");
}

/** Writes a calculated number with four significant digits. */
export function formatNumber(value: number): string {
  if (!Number.isFinite(value)) return String(value);
  if (value === 0) return "0";
  const magnitude = Math.abs(value);
  if (magnitude >= 1e-3 && magnitude < 1e6) return String(Number(value.toPrecision(4)));
  const [mantissa = "", exponent = "0"] = value.toExponential(3).split("e");
  return `${String(Number(mantissa))} \u{d7} 10${superscript(String(Number(exponent)))}`;
}

const BOUNDS: Record<string, string> = {
  lt: "< ",
  le: "\u{2264} ",
  gt: "> ",
  ge: "\u{2265} ",
  approx: "~",
};

/** Writes a value with the numbers and the unit exactly as the source states them. */
export function reportedValue(value: ValueInput): string {
  if (value.boolean !== undefined) return value.boolean ? "yes" : "no";
  if (value.category !== undefined) return value.category;
  const bound = value.bound === undefined ? "" : (BOUNDS[value.bound] ?? "");
  const main =
    value.number ??
    (value.low !== undefined && value.high !== undefined
      ? `${value.low}\u{2013}${value.high}`
      : "");
  const uncertainty = value.uncertainty;
  let spread = "";
  if (uncertainty?.number !== undefined) spread = ` \u{b1} ${uncertainty.number}`;
  else if (uncertainty?.low !== undefined && uncertainty.high !== undefined)
    spread = ` (${uncertainty.low}\u{2013}${uncertainty.high})`;
  const unit = value.unit_text ?? (value.unit === undefined ? "" : unitLabel(value.unit));
  return `${bound}${main}${spread}${unit ? ` ${unit}` : ""}`;
}

export function uncertaintyLabel(value: ValueInput): string | null {
  const uncertainty = value.uncertainty;
  if (!uncertainty) return null;
  const names: Record<string, string> = {
    sd: "SD",
    sem: "SEM",
    ci: uncertainty.level === undefined ? "CI" : `${uncertainty.level}% CI`,
    range: "range",
    iqr: "IQR",
    other: "uncertainty",
  };
  return names[uncertainty.kind] ?? uncertainty.kind;
}

const DIRECTIONS: Record<EffectBody["direction"], string> = {
  increase: "increase",
  decrease: "decrease",
  no_change: "no change",
  mixed: "mixed",
};

export function directionLabel(direction: EffectBody["direction"]): string {
  return DIRECTIONS[direction];
}

/** The value that a matrix shows for a direction: +1, -1, 0, or ± for mixed. */
export function scoreLabel(score: number | null): string {
  if (score === null) return "\u{b1}";
  if (score > 0) return "+1";
  if (score < 0) return "\u{2212}1";
  return "0";
}

export function significanceLabel(significance: EffectBody["significance"]): string | null {
  if (!significance) return null;
  const comparisons: Record<string, string> = {
    eq: "=",
    lt: "<",
    le: "\u{2264}",
    gt: ">",
    ge: "\u{2265}",
  };
  const parts: string[] = [];
  if (significance.p !== undefined)
    parts.push(`p ${comparisons[significance.comparison ?? "eq"] ?? "="} ${significance.p}`);
  if (significance.statement === "not_significant") parts.push("not significant");
  if (significance.statement === "significant") parts.push("significant");
  return parts.join(", ");
}

/** The label and the explanation of a value that the reader read from an image. */
export function imageReading(value: ValueInput): { label: string; explanation: string } | null {
  const reading = value.read_from_figure;
  if (!reading) return null;
  return reading.figure.startsWith("t")
    ? {
        label: "Read from table image",
        explanation:
          "The reader read this value from the image of a table. The text of the source does not state the number.",
      }
    : {
        label: "Read from figure",
        explanation:
          "The reader read this value from a plot. The source does not state the number.",
      };
}
