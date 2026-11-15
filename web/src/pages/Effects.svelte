<script lang="ts">
  import { docsLink } from "../lib/docs";
  import CheckBadge from "../components/CheckBadge.svelte";
  import Chart from "../components/Chart.svelte";
  import ConditionList from "../components/ConditionList.svelte";
  import FilteredTable from "../components/FilteredTable.svelte";
  import NoticeChips from "../components/NoticeChips.svelte";
  import { effectListSpec, effectSpec, effectTilesSpec } from "../lib/charts";
  import { download, toCsv } from "../lib/csv";
  import { loadIndex, loadReports } from "../lib/data";
  import {
    EFFECT_COLUMNS,
    EFFECT_FACETS,
    EFFECT_TITLES,
    EFFECT_VIEWS,
    effectGroupings,
    effectItems,
    effectLabels,
    effectRows,
    groupedMarks,
    matrixRows,
    type EffectItem,
    type EffectView,
  } from "../lib/effects";
  import { directionLabel, scoreLabel } from "../lib/format";
  import { link } from "../lib/route";
  import { replaceRoute, theme } from "../lib/state.svelte";
  import type { Report } from "../lib/types";
  import type { Column } from "../lib/view";

  let { params = "" }: { params?: string } = $props();

  const data = Promise.all([loadIndex(), loadReports()]);

  const VIEW_LABELS: Readonly<Record<EffectView, string>> = {
    tiles: "Tiles",
    list: "List",
    matrix: "Matrix",
  };

  let view = $state<EffectView>("tiles");
  let by = $state("agent");
  let figureWidth = $state(0);
  /** The width of the row labels: a part of the width of the figure, in steps of 20 pixels. */
  const labelLimit = $derived(
    figureWidth > 0 ? Math.min(320, Math.max(120, Math.round((figureWidth * 0.3) / 20) * 20)) : 240,
  );

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
  {:else if column.id === "conditions"}
    <ConditionList parts={row.contextParts} limit={4} />
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
  Each mark is one effect or association that one source reports: an increase (+1), a decrease (−1),
  no change or no effect (0), or a mixed result. arity does not combine the reports, so two sources
  that disagree show as two marks. The tiles give one square for each report, so the length of a row
  shows its number of reports. The list gives each direction its own column. The matrix puts the
  agents against the targets. If the source has a notice, for example a retraction, a sign shows
  above the mark, or a square has a dark outline. The filters change the figure and the table.
  <a href={docsLink("website", "figures")}>How the figures work</a>.
</p>
{#await data}
  <p class="muted">arity is loading the effects.</p>
{:then [index, reports]}
  {@const items = effectItems(effectRows(index, reports))}
  {#if items.length === 0}
    <p class="muted">No reader has submitted effects yet.</p>
  {:else}
    {#snippet figure(rows: readonly EffectItem[])}
      {#if rows.length > 0}
        {@const groupings = effectGroupings(rows)}
        {@const key = groupings.some((grouping) => grouping.key === by) ? by : "agent"}
        <div class="controls" role="group" aria-label="Figure">
          <span class="secondary">Figure</span>
          {#each EFFECT_VIEWS as option (option)}
            <button
              type="button"
              aria-pressed={view === option}
              onclick={() => {
                view = option;
              }}>{VIEW_LABELS[option]}</button
            >
          {/each}
          {#if view !== "matrix"}
            <label class="rows"
              >Rows
              <select
                value={key}
                onchange={(event) => {
                  by = event.currentTarget.value;
                }}
              >
                {#each groupings as grouping (grouping.key)}
                  <option value={grouping.key}>{grouping.label}</option>
                {/each}
              </select></label
            >
          {/if}
        </div>
        <div class="card figure" bind:clientWidth={figureWidth}>
          <Chart
            spec={view === "matrix"
              ? effectSpec(matrixRows(rows), theme.mode, labelLimit)
              : view === "list"
                ? effectListSpec(groupedMarks(rows, key, "list"), theme.mode, labelLimit)
                : effectTilesSpec(groupedMarks(rows, key, "tiles"), theme.mode, labelLimit)}
            label="The effects that each source reports"
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
      above={figure}
      key={(row: EffectItem) => row.report.id}
      name="effects"
      placeholder="Find effects, for example: hydralazine apoptosis"
    />
  {/if}
{:catch error}
  <p>arity cannot load the effects: {error instanceof Error ? error.message : String(error)}</p>
{/await}

<style>
  .figure {
    overflow-x: auto;
  }

  .rows {
    display: inline-flex;
    gap: 0.4rem;
    align-items: center;
    margin-left: 0.5rem;
  }

  select {
    background: var(--surface);
    border: 1px solid var(--axis);
    border-radius: 6px;
    padding: 0.25rem 0.4rem;
    max-width: 16rem;
  }
</style>
