<script lang="ts" generics="F extends string">
  import { emptySelection, isEmpty, toggle, type Selection } from "../lib/filter";

  let {
    selection,
    facets,
    titles,
    label,
    rangeLabel = "Value",
    onchange,
  }: {
    selection: Selection<F>;
    facets: readonly F[];
    titles: Readonly<Record<F, string>>;
    label: (facet: F, value: string) => string;
    rangeLabel?: string;
    onchange: (selection: Selection<F>) => void;
  } = $props();

  function range(min: number | null, max: number | null): string {
    if (min !== null && max !== null) return `${String(min)} to ${String(max)}`;
    return min !== null ? `${String(min)} or more` : `${String(max)} or less`;
  }
</script>

{#if !isEmpty(selection)}
  <div class="chips" role="group" aria-label="Filters">
    {#if selection.text.trim() !== ""}
      <button
        type="button"
        class="chip"
        onclick={() => {
          onchange({ ...selection, text: "" });
        }}>Words: "{selection.text.trim()}" <span aria-label="Remove">×</span></button
      >
    {/if}
    {#each facets as facet (facet)}
      {#each selection.facets[facet] ?? [] as value (value)}
        <button
          type="button"
          class="chip"
          onclick={() => {
            onchange(toggle(selection, facet, value));
          }}>{titles[facet]}: {label(facet, value)} <span aria-label="Remove">×</span></button
        >
      {/each}
    {/each}
    {#if selection.min !== null || selection.max !== null}
      <button
        type="button"
        class="chip"
        onclick={() => {
          onchange({ ...selection, min: null, max: null });
        }}
        >{rangeLabel}: {range(selection.min, selection.max)}
        <span aria-label="Remove">×</span></button
      >
    {/if}
    <button
      type="button"
      class="clear"
      onclick={() => {
        onchange(emptySelection());
      }}>Remove all filters</button
    >
  </div>
{/if}

<style>
  .chips {
    margin: var(--space-3) 0 0;
    align-items: center;
  }

  button.chip {
    cursor: pointer;
    white-space: normal;
    text-align: left;
    min-height: 1.6rem;
    background: var(--accent-soft);
    border-color: transparent;
    color: var(--accent-ink);
    font-weight: 550;
  }

  button.chip:hover:not(:disabled) {
    background: var(--accent-soft);
    border-color: var(--accent);
  }

  .clear {
    min-height: 0;
    border: 0;
    background: none;
    padding: 0 var(--space-2);
    color: var(--secondary);
    font-size: var(--text-sm);
  }

  .clear:hover:not(:disabled) {
    background: none;
    color: var(--ink);
    text-decoration: underline;
  }
</style>
