import { describe, expect, it } from "vitest";

import { findInCatalogue } from "./find";
import type { Index } from "./types";

const index: Index = {
  format: "arity-site/1",
  software: "arity 0.1.0",
  generated_at: "2026-11-14T10:00:00Z",
  database: "arity.sqlite",
  works: [
    {
      id: 1,
      title: "Synergistic combination of doxorubicin with hydralazine",
      citation: "Lafi et al. 2023, PLOS ONE",
      year: 2023,
      journal: "PLOS ONE",
      doi: "10.1371/journal.pone.0291981",
      pmcid: "PMC10538757",
      license: "CC BY 4.0",
      shareable: true,
      is_preprint: false,
      notices: [],
      reports: 1,
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
      reports: 1,
    },
  ],
  entities: [
    {
      id: 7,
      kind: "chemical",
      label: "doxorubicin",
      xrefs: ["CHEBI:28748"],
      names: ["DOX", "Adriamycin"],
      reports: 1,
    },
    { id: 8, kind: "cell_line", label: "MCF-7", xrefs: [], names: [], reports: 1 },
  ],
};

const ids = (search: string) => {
  const found = findInCatalogue(index, search);
  return found
    ? {
        works: found.works.map((work) => work.id),
        properties: found.properties.map((property) => property.key),
        entities: found.entities.map((entity) => entity.id),
      }
    : null;
};

describe("findInCatalogue", () => {
  it("finds works, properties, and entities that have each word", () => {
    expect(ids("doxorubicin")).toEqual({ works: [1], properties: [], entities: [7] });
    expect(ids("plos 2023")).toEqual({ works: [1], properties: [], entities: [] });
    expect(ids("IC50")).toEqual({ works: [], properties: ["ic50"], entities: [] });
  });

  it("finds by identifiers, other names, and kinds", () => {
    expect(ids("pmc10538757")?.works).toEqual([1]);
    expect(ids("CHEBI:28748")?.entities).toEqual([7]);
    expect(ids("adriamycin")?.entities).toEqual([7]);
    expect(ids("cell line")?.entities).toEqual([8]);
  });

  it("finds nothing until the search has two characters", () => {
    expect(findInCatalogue(index, " d ")).toBeNull();
    expect(findInCatalogue(index, '""')).toBeNull();
  });
});
