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
    margin: 0;
    padding: var(--space-3) 0;
    min-width: 0;
    border-bottom: 1px solid var(--line);
  }

  legend {
    float: left;
    width: 100%;
    padding: 0;
    margin-bottom: var(--space-2);
    font-size: var(--text-xs);
    font-weight: 650;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--muted);
  }

  label {
    clear: both;
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    gap: 0.5rem;
    align-items: baseline;
    padding: 0.22rem 0.35rem;
    margin: 0 -0.35rem;
    border-radius: var(--radius-sm);
    font-size: var(--text-sm);
    line-height: 1.35;
    cursor: pointer;
  }

  label:hover {
    background: var(--surface-2);
  }

  .name {
    overflow-wrap: anywhere;
  }

  .count {
    color: var(--muted);
    font-variant-numeric: tabular-nums;
    font-size: var(--text-xs);
  }

  .empty .name {
    color: var(--muted);
  }

  input[type="search"] {
    clear: both;
    display: block;
    width: 100%;
    margin-bottom: var(--space-2);
    min-height: 2rem;
  }

  .more {
    clear: both;
    min-height: 0;
    border: 0;
    background: none;
    padding: var(--space-1) 0 0;
    color: var(--accent-ink);
    font-size: var(--text-sm);
    font-weight: 550;
  }

  .more:hover:not(:disabled) {
    background: none;
    text-decoration: underline;
  }
</style>
