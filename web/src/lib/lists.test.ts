import { describe, expect, it } from "vitest";

import { emptySelection, filter } from "./filter";
import {
  ENTITY_FACETS,
  entityLabel,
  entityRows,
  PROPERTY_FACETS,
  propertyLabel,
  propertyRows,
  WORK_FACETS,
  workLabel,
  workRows,
} from "./lists";
import type { Index } from "./types";

const index: Index = {
  format: "arity-site/1",
  software: "arity 0.1.0",
  generated_at: "2026-11-14T10:00:00Z",
  database: "arity.sqlite",
  works: [
    {
      id: 1,
      title: "Doxorubicin and hydralazine",
      citation: "Lafi et al. 2023, PLOS ONE",
      year: 2023,
      journal: "PLOS ONE",
      doi: "10.1371/journal.pone.0291981",
      pmcid: "PMC10538757",
      license: "CC BY 4.0",
      shareable: true,
      is_preprint: false,
      notices: [],
      reports: 47,
      state: "read",
    },
    {
      id: 2,
      title: "New pyrimidines",
      citation: "Keshk et al. 2025, Scientific Reports",
      year: 2025,
      journal: "Scientific Reports",
      doi: null,
      pmcid: "PMC11920053",
      license: null,
      shareable: true,
      is_preprint: false,
      notices: ["retraction"],
      reports: 1357,
      state: "read",
    },
  ],
  properties: [
    {
      key: "ic50",
      label: "half maximal inhibitory concentration",
      value_type: "quantity",
      unit: "umol/L",
      dimension: "L-3 N",
      categories: [],
      xrefs: [],
      description: null,
      reports: 116,
    },
    {
      key: "is_active",
      label: "activity",
      value_type: "boolean",
      unit: null,
      dimension: null,
      categories: [],
      xrefs: [],
      description: "Whether the compound is active",
      reports: 2,
    },
  ],
  entities: [
    {
      id: 7,
      kind: "chemical",
      label: "doxorubicin",
      xrefs: ["CHEBI:28748", "PUBCHEM.COMPOUND:31703"],
      names: ["DOX"],
      reports: 40,
    },
    { id: 8, kind: "cell_line", label: "MCF-7", xrefs: [], names: [], reports: 30 },
  ],
};

describe("workRows", () => {
  const rows = workRows(index);

  it("gives the facets of each work: its notices, its license, and its year", () => {
    expect(rows.map((row) => row.values)).toEqual([
      { notice: ["none"], license: ["CC BY 4.0"], year: ["2023"] },
      { notice: ["retraction"], license: ["not stated"], year: ["2025"] },
    ]);
    expect(workLabel("notice", "none")).toBe("No notice");
    expect(workLabel("notice", "retraction")).toBe("Retraction");
    expect(workLabel("year", "2025")).toBe("2025");
  });

  it("finds a work by its title, citation, journal, and identifiers", () => {
    const found = (text: string) =>
      filter(rows, { ...emptySelection(), text }, WORK_FACETS).items.map((row) => row.work.id);
    expect(found("keshk pyrimidines")).toEqual([2]);
    expect(found("pmc10538757")).toEqual([1]);
  });
});

describe("propertyRows", () => {
  const rows = propertyRows(index);

  it("gives the type and the dimension of each property, and finds it by its unit", () => {
    expect(rows.map((row) => row.values)).toEqual([
      { type: ["quantity"], dimension: ["L-3 N"] },
      { type: ["boolean"], dimension: [] },
    ]);
    expect(propertyLabel("type", "boolean")).toBe("yes or no");
    const found = filter(rows, { ...emptySelection(), text: "\u{b5}mol" }, PROPERTY_FACETS);
    expect(found.items.map((row) => row.property.key)).toEqual(["ic50"]);
  });
});

describe("entityRows", () => {
  const rows = entityRows(index);

  it("gives the kind and the registries of each entity", () => {
    expect(rows.map((row) => row.values)).toEqual([
      { kind: ["chemical"], registry: ["CHEBI", "PUBCHEM.COMPOUND"] },
      { kind: ["cell_line"], registry: ["none"] },
    ]);
    expect(entityLabel("kind", "cell_line")).toBe("cell line");
    expect(entityLabel("registry", "none")).toBe("No identifier");
    const found = filter(rows, { ...emptySelection(), text: "dox" }, ENTITY_FACETS);
    expect(found.items.map((row) => row.entity.id)).toEqual([7]);
  });
});
