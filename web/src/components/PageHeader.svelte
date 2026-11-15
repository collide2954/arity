<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    title,
    eyebrow,
    summary,
    actions,
    children,
  }: {
    title: string;
    /** A short word above the title, for example the kind of page. */
    eyebrow?: string;
    /** One sentence about the page. */
    summary?: string;
    /** Controls at the side of the title. */
    actions?: Snippet;
    /** The explanation of the page, which a person can open. */
    children?: Snippet;
  } = $props();
</script>

<header class="page">
  <div class="titles">
    {#if eyebrow}<p class="eyebrow">{eyebrow}</p>{/if}
    <h1>{title}</h1>
    {#if summary}<p class="summary">{summary}</p>{/if}
  </div>
  {#if actions}<div class="actions">{@render actions()}</div>{/if}
</header>
{#if children}
  <details class="explain">
    <summary>How to read this page</summary>
    <div class="body">{@render children()}</div>
  </details>
{/if}

<style>
  .page {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    align-items: flex-end;
    gap: var(--space-3) var(--space-5);
    margin-bottom: var(--space-4);
  }

  .titles {
    min-width: 0;
    max-width: 80ch;
  }

  h1 {
    margin: 0;
    overflow-wrap: anywhere;
  }

  .summary {
    margin: var(--space-2) 0 0;
    color: var(--secondary);
    font-size: var(--text-lg);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    align-items: center;
  }

  .explain {
    margin: 0 0 var(--space-5);
    font-size: var(--text-sm);
  }

  .explain summary {
    display: inline-flex;
    cursor: pointer;
    color: var(--accent-ink);
    font-weight: 550;
  }

  .body {
    max-width: 78ch;
    color: var(--secondary);
    padding: var(--space-2) 0 0;
  }

  .body :global(p) {
    margin: 0 0 var(--space-2);
  }
</style>
