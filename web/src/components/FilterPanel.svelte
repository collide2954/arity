<script lang="ts" generics="F extends string">
  import { MediaQuery } from "svelte/reactivity";

  import { options, toggle, type Selection } from "../lib/filter";
  import FacetGroup from "./FacetGroup.svelte";

  let {
    facets,
    titles,
    counts,
    selection,
    label,
    onchange,
  }: {
    facets: readonly F[];
    titles: Readonly<Record<F, string>>;
    counts: ReadonlyMap<F, ReadonlyMap<string, number>>;
    selection: Selection<F>;
    label: (facet: F, value: string) => string;
    onchange: (selection: Selection<F>) => void;
  } = $props();

  const wide = new MediaQuery("min-width: 960px");
</script>

{#snippet groups()}
  {#each facets as facet (facet)}
    <FacetGroup
      title={titles[facet]}
      options={options(facet, counts.get(facet) ?? new Map<string, number>(), selection, (value) =>
        label(facet, value),
      )}
      ontoggle={(value: string) => {
        onchange(toggle(selection, facet, value));
      }}
    />
  {/each}
{/snippet}

{#if wide.current}
  <aside class="filters" aria-label="Filters">{@render groups()}</aside>
{:else}
  <details class="filters">
    <summary>Filters</summary>
    {@render groups()}
  </details>
{/if}

<style>
  aside {
    position: sticky;
    top: calc(var(--header) + var(--space-3));
    max-height: calc(100vh - var(--header) - var(--space-5));
    overflow: auto;
    padding-right: var(--space-2);
  }

  details {
    border: 1px solid var(--line);
    border-radius: var(--radius);
    background: var(--surface);
    padding: var(--space-2) var(--space-4);
  }

  summary {
    font-weight: 600;
    cursor: pointer;
    padding: var(--space-1) 0;
  }
</style>
