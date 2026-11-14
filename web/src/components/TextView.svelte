<script lang="ts">
  import { unquotedOf } from "../lib/coverage";
  import { byBlock, reveal, segments, type Highlight } from "../lib/highlight";
  import type { Block, Coverage, TextDocument } from "../lib/types";

  let {
    document,
    coverage = null,
    highlights,
    selected,
    onselect,
  }: {
    document: TextDocument;
    coverage?: Coverage | null;
    highlights: readonly Highlight[];
    selected: number | null;
    onselect: (report: number) => void;
  } = $props();

  interface Table {
    key: string;
    rows: Block[][];
  }

  type Part = { kind: "block"; block: Block } | { kind: "table"; table: Table };

  const parts = $derived.by(() => {
    const result: Part[] = [];
    for (const block of document.blocks) {
      const cell = block.cell;
      if (!cell) {
        result.push({ kind: "block", block });
        continue;
      }
      const last = result.at(-1);
      let table: Table;
      if (last?.kind === "table" && last.table.key === cell.table) {
        table = last.table;
      } else {
        table = { key: cell.table, rows: [] };
        result.push({ kind: "table", table });
      }
      const row = table.rows.find((cells) => cells[0]?.cell?.row === cell.row);
      if (row) row.push(block);
      else table.rows.push([block]);
    }
    return result;
  });

  const blockHighlights = $derived(byBlock(highlights));

  function rangesOf(block: Block) {
    return (blockHighlights.get(block.key) ?? []).map(({ start, end, report }) => ({
      start,
      end,
      report,
    }));
  }

  let container: HTMLDivElement | undefined = $state();

  $effect(() => {
    if (selected === null || !container) return;
    const main = highlights
      .filter((highlight) => highlight.report === selected)
      .sort((a, b) => a.index - b.index)[0];
    if (!main) return;
    const element = container.querySelector<HTMLElement>(
      `[data-block="${CSS.escape(main.block)}"]`,
    );
    if (element) reveal(container, element);
  });
</script>

{#snippet content(block: Block)}
  {#each segments(block.text, rangesOf(block), unquotedOf(coverage, block.key)) as segment, index (index)}
    {#if segment.unquoted}
      <span class="unquoted" title="No report quotes this number yet.">{segment.text}</span>
    {:else if segment.reports.length > 0}
      <mark
        data-reports={segment.reports.join(" ")}
        class:selected={selected !== null && segment.reports.includes(selected)}
        ><button
          type="button"
          onclick={() => {
            onselect(segment.reports[0] ?? 0);
          }}>{segment.text}</button
        ></mark
      >
    {:else}{segment.text}{/if}
  {/each}
{/snippet}

<div class="text-view" bind:this={container}>
  {#each parts as part, index (index)}
    {#if part.kind === "table"}
      <div class="table-wrap">
        <table>
          <tbody>
            {#each part.table.rows as row, rowIndex (rowIndex)}
              <tr>
                {#each row as cell (cell.key)}
                  <svelte:element
                    this={cell.cell?.header ? "th" : "td"}
                    colspan={cell.cell?.column_span ?? 1}
                    rowspan={cell.cell?.row_span ?? 1}
                    title={cell.key}
                    data-block={cell.key}>{@render content(cell)}</svelte:element
                  >
                {/each}
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {:else if part.block.kind === "title"}
      <h2 data-block={part.block.key}>{@render content(part.block)}</h2>
    {:else if part.block.kind === "heading"}
      <h3 data-block={part.block.key}>{@render content(part.block)}</h3>
    {:else}
      <p title={part.block.key} data-block={part.block.key}>
        {#if part.block.label}<strong>{part.block.label}. </strong>{/if}{@render content(
          part.block,
        )}
      </p>
    {/if}
  {/each}
</div>

<style>
  .text-view {
    max-height: 80vh;
    overflow: auto;
    background: var(--surface);
    border-radius: 8px;
    box-shadow: 0 0 0 1px var(--ring);
    padding: 1rem 1.25rem;
    max-width: 80ch;
  }

  .unquoted {
    text-decoration: underline dotted var(--muted);
    text-underline-offset: 0.2em;
  }

  mark {
    background: var(--mark);
    color: inherit;
    border-radius: 2px;
  }

  mark.selected {
    background: var(--mark-strong);
    outline: 2px solid var(--accent);
  }

  mark button {
    all: unset;
    cursor: pointer;
  }

  mark button:focus-visible {
    outline: 2px solid var(--accent);
  }

  table {
    margin: 0.75rem 0;
    font-size: 0.9rem;
  }
</style>
