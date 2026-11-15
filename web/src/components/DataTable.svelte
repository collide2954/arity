<script lang="ts" generics="T extends object">
  import type { Snippet } from "svelte";

  import type { Page } from "../lib/paging";
  import { PAGE_SIZES, sortedBy, type Column, type TableView } from "../lib/view";

  let {
    page,
    columns,
    view,
    onview,
    cell,
    key,
    label,
    onhover,
  }: {
    page: Page<T>;
    columns: readonly Column<T>[];
    view: TableView;
    onview: (view: TableView) => void;
    cell: Snippet<[T, Column<T>]>;
    key: (row: T) => string | number;
    label: string;
    /** Gets the row under the pointer, or null when the pointer leaves the rows. */
    onhover?: (row: T | null) => void;
  } = $props();

  function sign(column: Column<T>): string {
    if (view.sort !== column.id) return "";
    return view.order === "ascending" ? "\u{25b2}" : "\u{25bc}";
  }
</script>

<div class="table-wrap card">
  <table aria-label={label}>
    <thead>
      <tr>
        {#each columns as column (column.id)}
          <th
            class:numeric={column.numeric}
            aria-sort={view.sort === column.id ? view.order : "none"}
            scope="col"
          >
            <button
              type="button"
              title="Sort by {column.label}"
              onclick={() => {
                onview(sortedBy(view, column.id));
              }}>{column.label}<span class="sign" aria-hidden="true">{sign(column)}</span></button
            >
          </th>
        {/each}
      </tr>
    </thead>
    <tbody
      onpointerleave={() => {
        onhover?.(null);
      }}
    >
      {#each page.rows as row (key(row))}
        <tr
          onpointerenter={() => {
            onhover?.(row);
          }}
        >
          {#each columns as column (column.id)}
            <td class:numeric={column.numeric}>{@render cell(row, column)}</td>
          {/each}
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<nav class="pager" aria-label="Pages of {label}">
  <span class="muted">
    {#if page.total === 0}No rows{:else}Rows {page.first} to {page.last} of {page.total}{/if}
  </span>
  {#if page.pages > 1}
    <button
      type="button"
      disabled={page.page <= 1}
      onclick={() => {
        onview({ ...view, page: page.page - 1 });
      }}>Previous</button
    >
    <span>Page {page.page} of {page.pages}</span>
    <button
      type="button"
      disabled={page.page >= page.pages}
      onclick={() => {
        onview({ ...view, page: page.page + 1 });
      }}>Next</button
    >
  {/if}
  <label>
    Rows on a page
    <select
      value={view.size}
      onchange={(event) => {
        onview({ ...view, size: Number(event.currentTarget.value), page: 1 });
      }}
    >
      {#each PAGE_SIZES as size (size)}
        <option value={size}>{size}</option>
      {/each}
    </select>
  </label>
</nav>

<style>
  th button {
    all: unset;
    cursor: pointer;
    display: inline-flex;
    gap: 0.25rem;
    align-items: baseline;
  }

  th button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .sign {
    font-size: 0.7rem;
    color: var(--accent-ink);
  }

  .numeric {
    text-align: right;
  }

  .pager {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    align-items: center;
    margin: 0.75rem 0;
  }

  .pager label {
    display: inline-flex;
    gap: 0.4rem;
    align-items: center;
    margin-left: auto;
  }

  select {
    background: var(--surface);
    border: 1px solid var(--axis);
    border-radius: 6px;
    padding: 0.2rem 0.4rem;
  }
</style>
