<script lang="ts">
  import type { Option } from "../lib/filter";
  import { searchForm } from "../lib/search";

  let {
    title,
    options,
    ontoggle,
    limit = 6,
  }: {
    title: string;
    options: readonly Option[];
    ontoggle: (value: string) => void;
    limit?: number;
  } = $props();

  let open = $state(false);
  let find = $state("");

  const chosen = $derived(options.filter((option) => option.selected).length);
  const shown = $derived.by(() => {
    if (!open) return options.slice(0, Math.max(limit, chosen));
    const needle = searchForm(find);
    return needle === ""
      ? options
      : options.filter((option) => option.selected || searchForm(option.label).includes(needle));
  });
</script>

{#if options.length > 0}
  <fieldset class="facet">
    <legend>{title}</legend>
    {#if open && options.length > 12}
      <input type="search" placeholder="Find a value" aria-label="Find a value" bind:value={find} />
    {/if}
    {#each shown as option (option.value)}
      <label class:empty={option.count === 0}>
        <input
          type="checkbox"
          checked={option.selected}
          onchange={() => {
            ontoggle(option.value);
          }}
        />
        <span class="name">{option.label}</span>
        <span class="count">{option.count}</span>
      </label>
    {/each}
    {#if options.length > Math.max(limit, chosen)}
      <button
        type="button"
        class="more"
        aria-expanded={open}
        onclick={() => {
          open = !open;
          find = "";
        }}>{open ? "Show fewer" : `Show all ${String(options.length)}`}</button
      >
    {/if}
  </fieldset>
{/if}

<style>
  .facet {
    border: 0;
    margin: 0 0 1rem;
    padding: 0;
    min-width: 0;
  }

  legend {
    font-weight: 600;
    font-size: 0.85rem;
    color: var(--secondary);
    padding: 0;
    margin-bottom: 0.25rem;
  }

  label {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    gap: 0.4rem;
    align-items: baseline;
    padding: 0.1rem 0;
    font-size: 0.9rem;
    cursor: pointer;
  }

  .name {
    overflow-wrap: anywhere;
  }

  .count {
    color: var(--muted);
    font-variant-numeric: tabular-nums;
    font-size: 0.8rem;
  }

  .empty .name {
    color: var(--muted);
  }

  input[type="search"] {
    margin-bottom: 0.35rem;
    font-size: 0.85rem;
  }

  .more {
    border: 0;
    background: none;
    padding: 0.15rem 0;
    color: var(--accent-ink);
    font-size: 0.85rem;
  }
</style>
