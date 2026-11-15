<script lang="ts">
  import type { ConditionPart } from "../lib/describe";

  let {
    parts,
    limit = Infinity,
  }: {
    parts: readonly ConditionPart[];
    /** The largest number of conditions to list. The list then tells how many it leaves out. */
    limit?: number;
  } = $props();

  const shown = $derived(parts.slice(0, limit));
  const hidden = $derived(parts.length - shown.length);
</script>

{#if parts.length > 0}
  <ul class="conditions">
    {#each shown as part, position (position)}
      <li title={part.name ? `${part.name}: ${part.value}` : part.value}>
        {#if part.name}<span class="name">{part.name}</span>{/if}
        <span class="value">{part.value}</span>
      </li>
    {/each}
    {#if hidden > 0}
      <li class="more">and {hidden} more</li>
    {/if}
  </ul>
{/if}

<style>
  .conditions {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 0.3rem;
    min-width: 18ch;
    max-width: 44ch;
  }

  li {
    display: grid;
    line-height: 1.3;
  }

  .name {
    font-size: 0.75rem;
    color: var(--secondary);
  }

  .value {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .more {
    font-size: 0.8rem;
    color: var(--muted);
  }
</style>
