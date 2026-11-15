<script lang="ts">
  import Chart from "../components/Chart.svelte";
  import ColumnChooser from "../components/ColumnChooser.svelte";
  import CopyLink from "../components/CopyLink.svelte";
  import DataTable from "../components/DataTable.svelte";
  import FilterPanel from "../components/FilterPanel.svelte";
  import RangeFields from "../components/RangeFields.svelte";
  import ReportCell from "../components/ReportCell.svelte";
  import SelectionChips from "../components/SelectionChips.svelte";
  import { valueSpec } from "../lib/charts";
  import { download } from "../lib/csv";
  import { loadIndex, loadReports } from "../lib/data";
  import {
    filter,
    readSelection,
    selectionQuery,
    writeSelection,
    type Selection,
  } from "../lib/filter";
  import { unitLabel } from "../lib/format";
  import {
    FACET_TITLES,
    facetLabels,
    REPORT_COLUMNS,
    reportRows,
    reportsCsv,
    reportsJson,
    type ReportFacet,
    type ReportRow,
  } from "../lib/reports";
  import { link } from "../lib/route";
  import { replaceRoute, theme } from "../lib/state.svelte";
  import type { Index, PropertySummary } from "../lib/types";
  import {
    FIGURE_ROWS,
    figureRows,
    readFigure,
    rowsOfFigure,
    writeFigure,
    type FigureRows,
    type FigureView,
  } from "../lib/values";
  import {
    arrange,
    ordered,
    readView,
    shownColumns,
    writeView,
    type Column,
    type TableView,
  } from "../lib/view";

  let { key, params = "" }: { key: string; params?: string } = $props();

  const FACETS = [
    "entity",
    "condition",
    "work",
    "check",
    "notice",
    "origin",
  ] as const satisfies readonly ReportFacet[];
  type Facet = (typeof FACETS)[number];
  const IDS = REPORT_COLUMNS.map((column) => column.id);
  const DEFAULT_COLUMNS = [
    "subject",
    "reported",
    "converted",
    "conditions",
    "source",
    "check",
    "notices",
  ];
  const ROW_LABELS: Readonly<Record<FigureRows, string>> = {
    report: "Report",
    subject: "Subject",
    work: "Work",
  };

  interface Loaded {
    readonly index: Index;
    readonly property: PropertySummary | undefined;
    readonly rows: readonly ReportRow[];
    readonly label: (facet: Facet, value: string) => string;
  }

  let loaded = $state.raw<Loaded | null>(null);
  let failure = $state<string | null>(null);
  let highlight = $state<number | null>(null);
  let chartWidth = $state(0);

  // svelte-ignore state_referenced_locally
  const property = key;

  Promise.all([loadIndex(), loadReports()])
    .then(([index, reports]) => {
      const own = reports.filter(
        (report) => report.kind === "value" && report.property === property,
      );
      loaded = {
        index,
        property: index.properties.find((candidate) => candidate.key === property),
        rows: reportRows(index, own),
        label: facetLabels(index),
      };
    })
    .catch((error: unknown) => {
      failure = error instanceof Error ? error.message : String(error);
    });

  const query = $derived(new URLSearchParams(params));
  const selection = $derived(readSelection(query, FACETS));
  const view = $derived(readView(query, IDS));
  const figure = $derived(readFigure(query));
  const filtered = $derived(loaded ? filter(loaded.rows, selection, FACETS) : null);
  const page = $derived(filtered ? arrange(filtered.items, REPORT_COLUMNS, view) : null);
  const columns = $derived(shownColumns(REPORT_COLUMNS, view, DEFAULT_COLUMNS));
  const rows = $derived(
    rowsOfFigure(figure, {
      reports: loaded?.rows.length ?? 0,
      subjects: new Set(loaded?.rows.map((row) => row.subject)).size,
    }),
  );
  const linear = $derived(filtered ? figureRows(filtered.items, { rows, logarithmic: false }) : []);
  const positive = $derived(linear.every((row) => row.value > 0 && (row.low ?? 1) > 0));
  const logarithmic = $derived(figure.logarithmic && positive);
  const plotted = $derived(
    logarithmic && filtered ? figureRows(filtered.items, { rows, logarithmic }) : linear,
  );
  /** The width of the row labels: a part of the width of the figure, in steps of 20 pixels. */
  const labelLimit = $derived(
    chartWidth > 0 ? Math.min(320, Math.max(120, Math.round((chartWidth * 0.3) / 20) * 20)) : 0,
  );

  function show(
    next: Selection<Facet>,
    nextView: TableView = { ...view, page: 1 },
    nextFigure: FigureView = figure,
  ): void {
    const out = new URLSearchParams();
    writeSelection(out, next, FACETS);
    writeFigure(out, nextFigure);
    writeView(out, nextView);
    replaceRoute({ page: "property", key: property, params: out.toString() });
  }

  function explorer(chosen: Selection<Facet>): string {
    return link({
      page: "reports",
      params: selectionQuery(chosen, FACETS, [
        ["property", property],
        ["kind", "value"],
      ]),
    });
  }

  function open(report: number): void {
    const found = loaded?.rows.find((row) => row.report.id === report);
    if (found) window.location.hash = link({ page: "work", id: found.report.work, report });
  }
</script>

{#snippet cell(row: ReportRow, column: Column<ReportRow>)}
  <ReportCell {row} {column} />
{/snippet}

{#if failure}
  <p>arity cannot load the values: {failure}</p>
{:else if !loaded || !filtered || !page}
  <p class="muted">arity is loading the values.</p>
{:else if !loaded.property}
  <h1>The property does not exist</h1>
  <p>arity has no property <span class="mono">{property}</span>.</p>
{:else}
  {@const data = loaded}
  {@const summary = loaded.property}
  {@const unit = summary.unit ? unitLabel(summary.unit) || "1" : ""}
  <h1>{summary.label}</h1>
  <p class="secondary">
    Key <span class="mono">{summary.key}</span>
    {#if summary.unit}· values in {unit} · dimension
      <span class="mono">{summary.dimension}</span>{/if}
  </p>
  <p class="lede">
    Each dot is one report of one source. arity does not average or weigh the reports. A filled dot
    is a report that a person confirmed, a hollow dot is a report that no person checked yet, and a
    pale dot is a report that a person rejected. A diamond is a value that the reader read from an
    image. A sign above a dot tells that its source has a notice, for example a retraction. Drag
    along the axis of values to select a range.
  </p>

  <div class="explorer">
    <FilterPanel
      facets={FACETS}
      titles={FACET_TITLES}
      counts={filtered.counts}
      {selection}
      label={data.label}
      onchange={(next: Selection<Facet>) => {
        show(next);
      }}
    />

    <section aria-label="Values">
      <input
        type="search"
        class="find"
        aria-label="Find values of {summary.label}"
        placeholder="Find values by words, for example: MCF-7 48 h"
        value={selection.text}
        oninput={(event) => {
          show({ ...selection, text: event.currentTarget.value });
        }}
      />
      <SelectionChips
        {selection}
        facets={FACETS}
        titles={FACET_TITLES}
        label={data.label}
        rangeLabel="Value"
        onchange={(next: Selection<Facet>) => {
          show(next);
        }}
      />
      {#if summary.unit}
        <RangeFields
          legend="Value, in {unit}"
          min={selection.min}
          max={selection.max}
          onchange={(min: number | null, max: number | null) => {
            show({ ...selection, min, max });
          }}
        />
      {/if}
      <div class="controls">
        <strong>{filtered.items.length}</strong>
        <span class="secondary">of {data.rows.length} reports</span>
        <span class="actions">
          <a href={explorer(selection)}>Open in the report explorer</a>
          <a
            href={link({
              page: "compare",
              params: selectionQuery(selection, FACETS, [["property", property]]),
            })}>Compare in a matrix</a
          >
          <CopyLink />
        </span>
      </div>

      {#if plotted.length > 0}
        <div class="controls" role="group" aria-label="Figure">
          <span class="secondary">One row for each</span>
          {#each FIGURE_ROWS as option (option)}
            <button
              type="button"
              aria-pressed={rows === option}
              onclick={() => {
                show(selection, view, { ...figure, rows: option });
              }}>{ROW_LABELS[option]}</button
            >
          {/each}
          <button
            type="button"
            aria-pressed={logarithmic}
            disabled={!positive}
            onclick={() => {
              show(selection, view, { ...figure, logarithmic: !figure.logarithmic });
            }}>Logarithmic scale</button
          >
        </div>
        <div class="card" bind:clientWidth={chartWidth}>
          {#if labelLimit > 0}
            <Chart
              spec={valueSpec(plotted, {
                unit: unit || "value",
                logarithmic,
                mode: theme.mode,
                grouped: rows !== "report",
                labelLimit,
              })}
              label="The values of {summary.label} in each report"
              {highlight}
              onselect={open}
              onrange={([min, max]: readonly [number, number]) => {
                show({ ...selection, min, max });
              }}
            />
          {/if}
        </div>
      {/if}

      <h2>Table</h2>
      <div class="controls">
        <span class="actions">
          <ColumnChooser
            columns={REPORT_COLUMNS}
            shown={columns.map((column) => column.id)}
            onchange={(next: string[] | null) => {
              show(selection, { ...view, columns: next });
            }}
          />
          <button
            type="button"
            onclick={() => {
              download(
                `arity-${property}.csv`,
                reportsCsv(ordered(filtered.items, REPORT_COLUMNS, view)),
              );
            }}>Download CSV</button
          >
          <button
            type="button"
            onclick={() => {
              download(
                `arity-${property}.json`,
                reportsJson(ordered(filtered.items, REPORT_COLUMNS, view)),
                "application/json",
              );
            }}>Download JSON</button
          >
        </span>
      </div>
      {#if filtered.items.length === 0}
        <p class="card muted">
          No report agrees with all filters. Remove a filter, or change the words.
        </p>
      {:else}
        <DataTable
          {page}
          {columns}
          {view}
          label="Values of {summary.label}"
          key={(row: ReportRow) => row.report.id}
          {cell}
          onview={(next: TableView) => {
            show(selection, next);
          }}
          onhover={(row: ReportRow | null) => {
            highlight = row?.report.id ?? null;
          }}
        />
      {/if}
    </section>
  </div>
{/if}

<style>
  .explorer {
    display: grid;
    grid-template-columns: minmax(13rem, 16rem) minmax(0, 1fr);
    gap: 1.5rem;
    align-items: start;
  }

  .find {
    max-width: none;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    align-items: center;
    margin-left: auto;
  }

  @media (max-width: 960px) {
    .explorer {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
