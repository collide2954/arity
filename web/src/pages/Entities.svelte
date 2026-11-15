<script lang="ts">
  import PageHeader from "../components/PageHeader.svelte";
  import FilteredTable from "../components/FilteredTable.svelte";
  import { loadIndex } from "../lib/data";
  import {
    ENTITY_COLUMNS,
    ENTITY_FACETS,
    ENTITY_TITLES,
    entityLabel,
    entityRows,
    type EntityRow,
  } from "../lib/lists";
  import { link } from "../lib/route";
  import { replaceRoute } from "../lib/state.svelte";
  import type { Column } from "../lib/view";

  let { params = "" }: { params?: string } = $props();
</script>

{#snippet cell(row: EntityRow, column: Column<EntityRow>)}
  {#if column.id === "label"}
    <a href={link({ page: "entity", id: row.entity.id })}>{row.entity.label}</a>
  {:else if column.id === "identifiers"}
    <div class="chips">
      {#each row.entity.xrefs as xref (xref)}
        <a class="chip mono" href="https://bioregistry.io/{xref}">{xref}</a>
      {/each}
    </div>
  {:else if column.id === "names"}
    {row.entity.names.join(", ")}
  {:else}
    {column.value(row) ?? ""}
  {/if}
{/snippet}

<PageHeader
  title="Entities"
  summary="The compounds, cells, organisms, materials, and other things that reports are about."
>
  <p>
    arity links an entity across works through its identifiers in public registries. An entity
    without an identifier stays in the work that names it.
  </p>
</PageHeader>

{#await loadIndex()}
  <p class="muted">arity is loading the entities.</p>
{:then index}
  <FilteredTable
    rows={entityRows(index)}
    facets={ENTITY_FACETS}
    titles={ENTITY_TITLES}
    label={entityLabel}
    columns={ENTITY_COLUMNS}
    {params}
    onparams={(next: string) => {
      replaceRoute({ page: "entities", params: next });
    }}
    {cell}
    key={(row: EntityRow) => row.entity.id}
    name="entities"
    placeholder="Find entities by name or identifier"
  />
{:catch error}
  <p>arity cannot load the entities: {error instanceof Error ? error.message : String(error)}</p>
{/await}
