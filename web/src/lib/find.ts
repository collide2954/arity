import { matchesAll, searchForm, terms } from "./search";
import type { EntitySummary, Index, PropertySummary, WorkSummary } from "./types";

/** The works, properties, and entities that have each word of a search. */
export interface Found {
  readonly works: WorkSummary[];
  readonly properties: PropertySummary[];
  readonly entities: EntitySummary[];
}

/** Finds in the catalogue. A search with fewer than two characters finds nothing. */
export function findInCatalogue(index: Index, search: string): Found | null {
  const words = terms(search);
  if (words.join("").length < 2) return null;
  const has = (...parts: readonly (string | null | undefined)[]) =>
    matchesAll(searchForm(parts.filter((part) => part).join(" ")), words);
  return {
    works: index.works.filter((work) =>
      has(work.title, work.citation, work.journal, work.doi, work.pmcid, String(work.year ?? "")),
    ),
    properties: index.properties.filter((property) =>
      has(property.label, property.key, property.unit, property.description),
    ),
    entities: index.entities.filter((entity) =>
      has(entity.label, entity.kind.replaceAll("_", " "), ...entity.names, ...entity.xrefs),
    ),
  };
}
