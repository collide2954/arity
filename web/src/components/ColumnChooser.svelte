<script lang="ts" generics="T">
  import type { Column } from "../lib/view";

  let {
    columns,
    shown,
    onchange,
  }: {
    columns: readonly Column<T>[];
    shown: readonly string[];
    /** Gives the new columns, or null for the default columns. */
    onchange: (columns: string[] | null) => void;
  } = $props();

  function toggled(id: string): string[] {
    const next = shown.includes(id) ? shown.filter((other) => other !== id) : [...shown, id];
    return columns.map((column) => column.id).filter((other) => next.includes(other));
  }
</script>

<details class="chooser">
  <summary>Columns</summary>
  <div class="panel card">
    {#each columns as column (column.id)}
      <label>
        <input
          type="checkbox"
          checked={shown.includes(column.id)}
          disabled={shown.length === 1 && shown.includes(column.id)}
          onchange={() => {
            onchange(toggled(column.id));
          }}
        />
        {column.label}
      </label>
    {/each}
    <button
      type="button"
      onclick={() => {
        onchange(null);
      }}>Show the default columns</button
    >
  </div>
</details>

<style>
  .chooser {
    position: relative;
  }

  summary {
    display: inline-flex;
    align-items: center;
    min-height: 2rem;
    padding: 0 0.8rem;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
    background: var(--surface);
    font-size: var(--text-sm);
    font-weight: 550;
    cursor: pointer;
    list-style: none;
  }

  summary:hover {
    background: var(--surface-2);
  }

  summary::-webkit-details-marker {
    display: none;
  }

  .panel {
    position: absolute;
    right: 0;
    z-index: 5;
    margin-top: var(--space-1);
    display: grid;
    gap: 0.15rem;
    min-width: 15rem;
    box-shadow: var(--shadow-2);
  }

  label {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    padding: 0.2rem 0.3rem;
    border-radius: var(--radius-sm);
    font-size: var(--text-sm);
    white-space: nowrap;
    cursor: pointer;
  }

  label:hover {
    background: var(--surface-2);
  }

  button {
    margin-top: var(--space-2);
  }
</style>
