<script lang="ts">
  import FilteredTable from "../components/FilteredTable.svelte";
  import NoticeChips from "../components/NoticeChips.svelte";
  import { download, toCsv } from "../lib/csv";
  import { loadIndex } from "../lib/data";
  import {
    WORK_COLUMNS,
    WORK_FACETS,
    WORK_TITLES,
    workLabel,
    workRows,
    type WorkRow,
  } from "../lib/lists";
  import { link } from "../lib/route";
  import { replaceRoute } from "../lib/state.svelte";
  import type { Column } from "../lib/view";

  let { params = "" }: { params?: string } = $props();

  function save(rows: readonly WorkRow[]): void {
    download(
      "arity-works.csv",
      toCsv(
        [
          "id",
          "title",
          "citation",
          "year",
          "doi",
          "pmcid",
          "license",
          "preprint",
          "notices",
          "reports",
        ],
        rows.map(({ work }) => [
          work.id,
          work.title,
          work.citation,
          work.year,
          work.doi,
          work.pmcid,
          work.license,
          work.is_preprint,
          work.notices.join("; "),
          work.reports,
        ]),
      ),
    );
  }
</script>

{#snippet cell(row: WorkRow, column: Column<WorkRow>)}
  {#if column.id === "title"}
    <a href={link({ page: "work", id: row.work.id, report: null })}>{row.work.title}</a>
    <div class="muted">
      {row.work.citation}
      {#if row.work.pmcid}· {row.work.pmcid}{/if}
      {#if row.work.is_preprint}· <span class="chip">Preprint</span>{/if}
    </div>
  {:else if column.id === "notices"}
    <NoticeChips kinds={row.work.notices} />
  {:else}
    {column.value(row) ?? ""}
  {/if}
{/snippet}

{#snippet actions(rows: readonly WorkRow[])}
  <button
    type="button"
    onclick={() => {
      save(rows);
    }}>Download CSV</button
  >
{/snippet}

<h1>Works</h1>
<p class="lede">
  The open-access works in the catalogue, in the order that arity found them. Sort the works by a
  column, or filter them by notice, license, and year.
</p>

{#await loadIndex()}
  <p class="muted">arity is loading the works.</p>
{:then index}
  <FilteredTable
    rows={workRows(index)}
    facets={WORK_FACETS}
    titles={WORK_TITLES}
    label={workLabel}
    columns={WORK_COLUMNS}
    {params}
    onparams={(next: string) => {
      replaceRoute({ page: "works", params: next });
    }}
    {cell}
    {actions}
    key={(row: WorkRow) => row.work.id}
    name="works"
    placeholder="Find works by title, journal, year, or identifier"
  />
{:catch error}
  <p>arity cannot load the works: {error instanceof Error ? error.message : String(error)}</p>
{/await}
