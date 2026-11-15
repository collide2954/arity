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
    cursor: pointer;
    border: 1px solid var(--axis);
    border-radius: 6px;
    padding: 0.3rem 0.75rem;
    background: var(--surface);
    list-style: none;
  }

  summary::-webkit-details-marker {
    display: none;
  }

  .panel {
    position: absolute;
    z-index: 5;
    margin-top: 0.25rem;
    display: grid;
    gap: 0.25rem;
    min-width: 14rem;
  }

  label {
    display: flex;
    gap: 0.4rem;
    align-items: baseline;
    white-space: nowrap;
  }

  button {
    margin-top: 0.35rem;
  }
</style>
