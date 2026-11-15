<script lang="ts">
  import PageHeader from "../components/PageHeader.svelte";
  import FilteredTable from "../components/FilteredTable.svelte";
  import { loadIndex } from "../lib/data";
  import {
    PROPERTY_COLUMNS,
    PROPERTY_FACETS,
    PROPERTY_TITLES,
    propertyLabel,
    propertyRows,
    type PropertyRow,
  } from "../lib/lists";
  import { link } from "../lib/route";
  import { replaceRoute } from "../lib/state.svelte";
  import type { Column } from "../lib/view";

  let { params = "" }: { params?: string } = $props();
</script>

{#snippet cell(row: PropertyRow, column: Column<PropertyRow>)}
  {#if column.id === "label"}
    <a href={link({ page: "property", key: row.property.key })}>{row.property.label}</a>
  {:else if column.id === "key" || column.id === "dimension"}
    <span class="mono">{column.value(row) ?? ""}</span>
  {:else}
    {column.value(row) ?? ""}
  {/if}
{/snippet}

<PageHeader
  title="Properties"
  summary="The quantities and qualities that reports give values for. Select one to compare its values."
/>

{#await loadIndex()}
  <p class="muted">arity is loading the properties.</p>
{:then index}
  <FilteredTable
    rows={propertyRows(index)}
    facets={PROPERTY_FACETS}
    titles={PROPERTY_TITLES}
    label={propertyLabel}
    columns={PROPERTY_COLUMNS}
    {params}
    onparams={(next: string) => {
      replaceRoute({ page: "properties", params: next });
    }}
    {cell}
    key={(row: PropertyRow) => row.property.key}
    name="properties"
    placeholder="Find properties by name, key, or unit"
  />
{:catch error}
  <p>arity cannot load the properties: {error instanceof Error ? error.message : String(error)}</p>
{/await}
