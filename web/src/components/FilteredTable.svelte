<script lang="ts" generics="F extends string, T extends Item<F>">
  import type { Snippet } from "svelte";

  import { filter, readSelection, writeSelection, type Item, type Selection } from "../lib/filter";
  import { arrange, ordered, readView, writeView, type Column, type TableView } from "../lib/view";
  import CopyLink from "./CopyLink.svelte";
  import DataTable from "./DataTable.svelte";
  import FilterPanel from "./FilterPanel.svelte";
  import SelectionChips from "./SelectionChips.svelte";

  let {
    rows,
    facets,
    titles,
    label,
    columns,
    params,
    onparams,
    cell,
    key,
    name,
    placeholder,
    actions,
    above,
  }: {
    rows: readonly T[];
    facets: readonly F[];
    titles: Readonly<Record<F, string>>;
    label: (facet: F, value: string) => string;
    columns: readonly Column<T>[];
    /** The query of the address, which keeps the selection and the view. */
    params: string;
    onparams: (params: string) => void;
    cell: Snippet<[T, Column<T>]>;
    key: (row: T) => string | number;
    /** The name of the rows, in the plural. */
    name: string;
    placeholder: string;
    /** Controls that act on the selected rows, in the order of the table. */
    actions?: Snippet<[readonly T[]]>;
    /** Content above the table that shows the selected rows. */
    above?: Snippet<[readonly T[]]>;
  } = $props();

  const query = $derived(new URLSearchParams(params));
  const selection = $derived(readSelection(query, facets));
  const view = $derived(
    readView(
      query,
      columns.map((column) => column.id),
    ),
  );
  const filtered = $derived(filter(rows, selection, facets));
  const page = $derived(arrange(filtered.items, columns, view));

  function show(next: Selection<F>, nextView: TableView = { ...view, page: 1 }): void {
    const out = new URLSearchParams();
    writeSelection(out, next, facets);
    writeView(out, nextView);
    onparams(out.toString());
  }
</script>

<div class="explorer">
  <FilterPanel
    {facets}
    {titles}
    counts={filtered.counts}
    {selection}
    {label}
    onchange={(next: Selection<F>) => {
      show(next);
    }}
  />
  <section aria-label={name}>
    <input
      type="search"
      class="find"
      aria-label="Find {name}"
      {placeholder}
      value={selection.text}
      oninput={(event) => {
        show({ ...selection, text: event.currentTarget.value });
      }}
    />
    <SelectionChips
      {selection}
      {facets}
      {titles}
      {label}
      onchange={(next: Selection<F>) => {
        show(next);
      }}
    />
    <div class="controls">
      <strong>{filtered.items.length}</strong>
      <span class="secondary">of {rows.length} {name}</span>
      <span class="actions">
        {@render actions?.(ordered(filtered.items, columns, view))}
        <CopyLink />
      </span>
    </div>
    {@render above?.(filtered.items)}
    {#if filtered.items.length === 0}
      <p class="card muted">
        Nothing agrees with all filters. Remove a filter, or change the words.
      </p>
    {:else}
      <DataTable
        {page}
        {columns}
        {view}
        label={name}
        {key}
        {cell}
        onview={(next: TableView) => {
          show(selection, next);
        }}
      />
    {/if}
  </section>
</div>

<style>
  .explorer {
    display: grid;
    grid-template-columns: minmax(13rem, 16rem) minmax(0, 1fr);
    gap: 1.5rem;
    align-items: start;
  }

  .find {
    max-width: none;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-left: auto;
  }

  @media (max-width: 960px) {
    .explorer {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
