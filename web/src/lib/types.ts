/** The shapes of the files that `arity export` writes. */

export interface Notice {
  kind: string;
  reported_type: string;
  source: string;
  notice_doi?: string;
  notice_pmid?: string;
  notice_citation?: string;
  date?: string;
  source_record?: string;
  retrieved_at: string;
}

export interface Author {
  name: string;
  orcid?: string;
}

export interface WorkMetadata {
  title: string;
  doi?: string;
  pmid?: string;
  pmcid?: string;
  article_type?: string;
  is_preprint?: boolean;
  journal?: string;
  publisher?: string;
  published?: string;
  year?: number;
  authors?: Author[];
  license?: string;
  license_statement?: string;
  copyright?: string;
  keywords?: string[];
}

export interface WorkSummary {
  id: number;
  title: string;
  citation: string;
  year: number | null;
  journal: string | null;
  doi: string | null;
  pmcid: string | null;
  license: string | null;
  shareable: boolean;
  is_preprint: boolean;
  notices: string[];
  reports: number;
  state: string | null;
}

export interface PropertySummary {
  key: string;
  label: string;
  value_type: "quantity" | "boolean" | "category";
  unit: string | null;
  dimension: string | null;
  categories: string[];
  xrefs: string[];
  description: string | null;
  reports: number;
}

export interface EntitySummary {
  id: number;
  kind: string;
  label: string;
  xrefs: string[];
  names: string[];
  reports: number;
}

export interface Index {
  format: string;
  software: string;
  generated_at: string;
  database: string;
  works: WorkSummary[];
  properties: PropertySummary[];
  entities: EntitySummary[];
}

export interface Evidence {
  block: string;
  quote: string;
  start: number;
  end: number;
}

export interface Check {
  state: CheckState;
  person: string;
  orcid: string | null;
  note: string | null;
  at: string;
}

export type CheckState = "unchecked" | "confirmed" | "rejected";

export interface Display {
  unit: string;
  value: number | null;
  low: number | null;
  high: number | null;
  interval_low: number | null;
  interval_high: number | null;
}

export interface Uncertainty {
  kind: string;
  number?: string;
  low?: string;
  high?: string;
  level?: string;
}

export interface ValueInput {
  number?: string;
  low?: string;
  high?: string;
  unit?: string;
  unit_text?: string;
  bound?: string;
  uncertainty?: Uncertainty;
  statistic?: string;
  n?: number;
  boolean?: boolean;
  category?: string;
  read_from_figure?: { figure: string; method: string; resolution: string };
}

export interface Condition {
  role?: string;
  entity?: string;
  text?: string;
  property?: string;
  value?: ValueInput;
}

export interface ValueBody {
  type: "value";
  id: string;
  subject: string[];
  property: string;
  value: ValueInput;
  conditions?: Condition[];
  method?: { label: string; xrefs?: string[] };
  note?: string;
}

export interface EffectBody {
  type: "effect";
  id: string;
  agents: { entity: string; dose?: ValueInput }[];
  target: { entity: string; property?: string };
  direction: "increase" | "decrease" | "no_change" | "mixed";
  relation?: "effect" | "association";
  magnitude?: { measure: string; number: string; unit?: string; unit_text?: string };
  comparator?: string;
  significance?: { statement?: string; comparison?: string; p?: string; test?: string };
  n?: number;
  conditions?: Condition[];
  method?: { label: string; xrefs?: string[] };
  note?: string;
}

export interface Report {
  id: number;
  work: number;
  run: number;
  local_id: string;
  kind: "value" | "effect";
  body: ValueBody | EffectBody;
  entities: Record<string, number>;
  property: string | null;
  direction: string | null;
  score: number | null;
  check: CheckState;
  checks: Check[];
  evidence: Evidence[];
  display: Display | null;
}

export interface Cell {
  table: string;
  row: number;
  column: number;
  header?: boolean;
  row_span?: number;
  column_span?: number;
}

export interface Block {
  key: string;
  kind: string;
  section?: string[];
  label?: string;
  cell?: Cell;
  text: string;
}

export interface Attachment {
  key: string;
  name: string;
  digest: string;
  media_type?: string;
  parser: string;
}

export interface TextDocument {
  format: string;
  source: string;
  parser: string;
  attachments?: Attachment[];
  blocks: Block[];
}

export interface Agent {
  kind: string;
  name: string;
  version: string | null;
  orcid: string | null;
}

export interface Issue {
  severity: "error" | "warning";
  path: string;
  code: string;
  message: string;
}

export interface Run {
  id: number;
  work: number;
  reader: Agent;
  responsible: Agent;
  software: string;
  submitted_at: string;
  submission_sha256: string;
  text_sha256: string;
  parser: string;
  warnings: Issue[];
}

export interface Retrieval {
  role: string;
  supplement_key: string | null;
  url: string;
  media_type: string | null;
  sha256: string;
  bytes: number;
  software: string;
  retrieved_at: string;
}

export interface WorkView {
  id: number;
  metadata: WorkMetadata;
  state: string | null;
  notices: Notice[];
  retrievals: Retrieval[];
  text_sha256: string | null;
  reports: number;
}

export interface Attribution {
  citation: string;
  license: string | null;
  license_url: string | null;
  statement: string | null;
  copyright: string | null;
  doi_url: string | null;
}

export interface CoverageCount {
  numbers: number;
  quoted: number;
  skipped: number;
}

/** A number of the text that no report quotes and no reader skipped. */
export interface Unquoted {
  block: string;
  number: string;
  /** The range in the text of the block, in Unicode code points. */
  start: number;
  end: number;
}

export interface Coverage {
  /** The numbers in the cells of tables and data files, other than headers. */
  cells: CoverageCount;
  /** The numbers in the other blocks that have a unit or a sign. */
  text: CoverageCount;
  unquoted: Unquoted[];
}

export interface WorkPage {
  work: WorkView;
  attribution: Attribution;
  document: TextDocument | null;
  coverage: Coverage | null;
  runs: Run[];
  reports: number[];
  pdf: string | null;
}
