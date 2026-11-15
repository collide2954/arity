/** The rows of the lists of works, properties, and entities, with their facets and columns. */

import type { Item } from "./filter";
import { unitLabel } from "./format";
import { noticeLabel } from "./notices";
import { searchForm } from "./search";
import type { EntitySummary, Index, PropertySummary, WorkSummary } from "./types";
import type { Column } from "./view";

function textOf(...parts: readonly (string | number | null | undefined)[]): string {
  return searchForm(parts.filter((part) => part !== null && part !== undefined).join(" "));
}

export const WORK_FACETS = ["notice", "license", "year"] as const;
export type WorkFacet = (typeof WORK_FACETS)[number];

export const WORK_TITLES: Readonly<Record<WorkFacet, string>> = {
  notice: "Notice",
  license: "License",
  year: "Year",
};

export interface WorkRow extends Item<WorkFacet> {
  readonly work: WorkSummary;
}

export function workRows(index: Index): WorkRow[] {
  return index.works.map((work) => ({
    work,
    text: textOf(work.title, work.citation, work.journal, work.doi, work.pmcid, work.year),
    number: null,
    values: {
      notice: work.notices.length > 0 ? work.notices : ["none"],
      license: [work.license ?? "not stated"],
      year: work.year === null ? [] : [String(work.year)],
    },
  }));
}

export function workLabel(facet: WorkFacet, value: string): string {
  if (facet === "notice") return value === "none" ? "No notice" : noticeLabel(value);
  return value;
}

export const WORK_COLUMNS: readonly Column<WorkRow>[] = [
  { id: "title", label: "Work", value: (row) => row.work.title },
  { id: "year", label: "Year", value: (row) => row.work.year, numeric: true },
  { id: "license", label: "License", value: (row) => row.work.license ?? "not stated" },
  {
    id: "notices",
    label: "Notices",
    value: (row) => row.work.notices.map(noticeLabel).join("; "),
  },
  { id: "reports", label: "Reports", value: (row) => row.work.reports, numeric: true },
];

export const PROPERTY_FACETS = ["type", "dimension"] as const;
export type PropertyFacet = (typeof PROPERTY_FACETS)[number];

export const PROPERTY_TITLES: Readonly<Record<PropertyFacet, string>> = {
  type: "Type",
  dimension: "Dimension",
};

const TYPES: Readonly<Record<string, string>> = {
  quantity: "quantity",
  boolean: "yes or no",
  category: "category",
};

export interface PropertyRow extends Item<PropertyFacet> {
  readonly property: PropertySummary;
  /** The display unit, as people read it. */
  readonly unit: string;
}

export function propertyRows(index: Index): PropertyRow[] {
  return index.properties.map((property) => {
    const unit = property.unit ? unitLabel(property.unit) || "1" : "";
    return {
      property,
      unit,
      text: textOf(
        property.label,
        property.key,
        property.unit,
        unit,
        property.dimension,
        property.description,
      ),
      number: null,
      values: {
        type: [property.value_type],
        dimension: property.dimension === null ? [] : [property.dimension],
      },
    };
  });
}

export function propertyLabel(facet: PropertyFacet, value: string): string {
  return facet === "type" ? (TYPES[value] ?? value) : value;
}

export const PROPERTY_COLUMNS: readonly Column<PropertyRow>[] = [
  { id: "label", label: "Property", value: (row) => row.property.label },
  { id: "key", label: "Key", value: (row) => row.property.key },
  { id: "type", label: "Type", value: (row) => propertyLabel("type", row.property.value_type) },
  { id: "unit", label: "Display unit", value: (row) => row.unit },
  { id: "dimension", label: "Dimension", value: (row) => row.property.dimension },
  { id: "reports", label: "Reports", value: (row) => row.property.reports, numeric: true },
];

export const ENTITY_FACETS = ["kind", "registry"] as const;
export type EntityFacet = (typeof ENTITY_FACETS)[number];

export const ENTITY_TITLES: Readonly<Record<EntityFacet, string>> = {
  kind: "Kind",
  registry: "Registry",
};

export interface EntityRow extends Item<EntityFacet> {
  readonly entity: EntitySummary;
}

export function entityRows(index: Index): EntityRow[] {
  return index.entities.map((entity) => {
    const registries = [...new Set(entity.xrefs.map((xref) => xref.split(":")[0] ?? xref))];
    return {
      entity,
      text: textOf(
        entity.label,
        entity.kind.replaceAll("_", " "),
        ...entity.names,
        ...entity.xrefs,
      ),
      number: null,
      values: {
        kind: [entity.kind],
        registry: registries.length > 0 ? registries : ["none"],
      },
    };
  });
}

export function entityLabel(facet: EntityFacet, value: string): string {
  if (facet === "kind") return value.replaceAll("_", " ");
  return value === "none" ? "No identifier" : value;
}

export const ENTITY_COLUMNS: readonly Column<EntityRow>[] = [
  { id: "label", label: "Entity", value: (row) => row.entity.label },
  { id: "kind", label: "Kind", value: (row) => entityLabel("kind", row.entity.kind) },
  { id: "identifiers", label: "Identifiers", value: (row) => row.entity.xrefs.join("; ") },
  { id: "names", label: "Names in sources", value: (row) => row.entity.names.join("; ") },
  { id: "reports", label: "Reports", value: (row) => row.entity.reports, numeric: true },
];
