<script lang="ts">
  import CheckBadge from "../components/CheckBadge.svelte";
  import Chart from "../components/Chart.svelte";
  import FilteredTable from "../components/FilteredTable.svelte";
  import NoticeChips from "../components/NoticeChips.svelte";
  import { effectSpec } from "../lib/charts";
  import { download, toCsv } from "../lib/csv";
  import { loadIndex, loadReports } from "../lib/data";
  import {
    EFFECT_COLUMNS,
    EFFECT_FACETS,
    EFFECT_TITLES,
    effectItems,
    effectLabels,
    effectRows,
    matrixRows,
    type EffectItem,
  } from "../lib/effects";
  import { directionLabel, scoreLabel } from "../lib/format";
  import { link } from "../lib/route";
  import { replaceRoute, theme } from "../lib/state.svelte";
  import type { Report } from "../lib/types";
  import type { Column } from "../lib/view";

  let { params = "" }: { params?: string } = $props();

  const data = Promise.all([loadIndex(), loadReports()]);

  function save(rows: readonly EffectItem[]): void {
    download(
      "arity-effects.csv",
      toCsv(
        [
          "report",
          "source",
          "agent",
          "target",
          "direction",
          "score",
          "details",
          "conditions",
          "check",
          "notices",
        ],
        rows.map((row) => [
          row.report.id,
          row.citation,
          row.agent,
          row.target,
          row.body.direction,
          row.report.score,
          row.detail,
          row.context,
          row.report.check,
          row.notices.join("; "),
        ]),
      ),
    );
  }

  function open(report: number, reports: readonly Report[]): void {
    const found = reports.find((candidate) => candidate.id === report);
    if (found) window.location.hash = link({ page: "work", id: found.work, report });
  }
</script>

{#snippet cell(row: EffectItem, column: Column<EffectItem>)}
  {#if column.id === "direction"}
    <strong>{scoreLabel(row.report.score)}</strong>
    {directionLabel(row.body.direction)}
  {:else if column.id === "source"}
    <a href={link({ page: "work", id: row.report.work, report: row.report.id })}>{row.citation}</a>
  {:else if column.id === "check"}
    <CheckBadge state={row.report.check} />
  {:else if column.id === "notices"}
    <NoticeChips kinds={row.notices} />
  {:else}
    {column.value(row) ?? ""}
  {/if}
{/snippet}

{#snippet actions(rows: readonly EffectItem[])}
  <button
    type="button"
    onclick={() => {
      save(rows);
    }}>Download CSV</button
  >
{/snippet}

<h1>Effects</h1>
<p class="lede">
  Each mark is one effect or association that one source reports. Up is an increase (+1), down is a
  decrease (−1), and a dot is a statement of no change or no effect (0). Two sources that disagree
  show as two marks in the same cell: arity does not combine them. If the source has a notice, for
  example a retraction, a sign shows above the mark. The filters change the matrix and the table.
</p>

{#await data}
  <p class="muted">arity is loading the effects.</p>
{:then [index, reports]}
  {@const items = effectItems(effectRows(index, reports))}
  {#if items.length === 0}
    <p class="muted">No reader has submitted effects yet.</p>
  {:else}
    {#snippet matrix(rows: readonly EffectItem[])}
      {#if rows.length > 0}
        <div class="card">
          <Chart
            spec={effectSpec(matrixRows(rows), theme.mode)}
            label="The effects that each source reports, by agent and target"
            onselect={(report: number) => {
              open(report, reports);
            }}
          />
        </div>
        <h2>Table</h2>
      {/if}
    {/snippet}
    <FilteredTable
      rows={items}
      facets={EFFECT_FACETS}
      titles={EFFECT_TITLES}
      label={effectLabels(index)}
      columns={EFFECT_COLUMNS}
      {params}
      onparams={(next: string) => {
        replaceRoute({ page: "effects", params: next });
      }}
      {cell}
      {actions}
      above={matrix}
      key={(row: EffectItem) => row.report.id}
      name="effects"
      placeholder="Find effects, for example: hydralazine apoptosis"
    />
  {/if}
{:catch error}
  <p>arity cannot load the effects: {error instanceof Error ? error.message : String(error)}</p>
{/await}
