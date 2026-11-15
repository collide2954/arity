<script lang="ts">
  import CheckBadge from "../components/CheckBadge.svelte";
  import ColumnChooser from "../components/ColumnChooser.svelte";
  import DataTable from "../components/DataTable.svelte";
  import FilterPanel from "../components/FilterPanel.svelte";
  import NoticeChips from "../components/NoticeChips.svelte";
  import RangeFields from "../components/RangeFields.svelte";
  import SelectionChips from "../components/SelectionChips.svelte";
  import { download, toCsv } from "../lib/csv";
  import { loadIndex, loadReports } from "../lib/data";
  import { filter, readSelection, writeSelection, type Selection } from "../lib/filter";
  import { formatNumber, imageReading, unitLabel } from "../lib/format";
  import {
    DEFAULT_REPORT_COLUMNS,
    FACET_TITLES,
    facetLabels,
    REPORT_COLUMNS,
    REPORT_FACETS,
    reportRows,
    type ReportFacet,
    type ReportRow,
  } from "../lib/reports";
  import { link } from "../lib/route";
  import { replaceRoute } from "../lib/state.svelte";
  import type { Index } from "../lib/types";
  import {
    arrange,
    ordered,
    readView,
    shownColumns,
    writeView,
    type Column,
    type TableView,
  } from "../lib/view";

  let { params = "" }: { params?: string } = $props();

  const IDS = REPORT_COLUMNS.map((column) => column.id);

  interface Loaded {
    readonly index: Index;
    readonly rows: readonly ReportRow[];
    readonly label: (facet: ReportFacet, value: string) => string;
  }

  let loaded = $state.raw<Loaded | null>(null);
  let failure = $state<string | null>(null);
  let copied = $state(false);

  Promise.all([loadIndex(), loadReports()])
    .then(([index, reports]) => {
      loaded = { index, rows: reportRows(index, reports), label: facetLabels(index) };
    })
    .catch((error: unknown) => {
      failure = error instanceof Error ? error.message : String(error);
    });

  const query = $derived(new URLSearchParams(params));
  const selection = $derived(readSelection(query, REPORT_FACETS));
  const view = $derived(readView(query, IDS));
  const filtered = $derived(loaded ? filter(loaded.rows, selection, REPORT_FACETS) : null);
  const page = $derived(filtered ? arrange(filtered.items, REPORT_COLUMNS, view) : null);
  const columns = $derived(shownColumns(REPORT_COLUMNS, view, DEFAULT_REPORT_COLUMNS));
  const property = $derived.by(() => {
    const keys = selection.facets.property ?? [];
    const key = keys.length === 1 ? keys[0] : undefined;
    return loaded?.index.properties.find((candidate) => candidate.key === key && candidate.unit);
  });

  function show(next: Selection<ReportFacet>, nextView: TableView = { ...view, page: 1 }): void {
    const out = new URLSearchParams();
    writeSelection(out, next, REPORT_FACETS);
    writeView(out, nextView);
    replaceRoute({ page: "reports", params: out.toString() });
  }

  function saveCsv(rows: readonly ReportRow[]): void {
    download(
      "arity-reports.csv",
      toCsv(
        REPORT_COLUMNS.map((column) => column.label),
        rows.map((row) => REPORT_COLUMNS.map((column) => column.value(row))),
      ),
    );
  }

  function saveJson(rows: readonly ReportRow[]): void {
    download(
      "arity-reports.json",
      `${JSON.stringify(
        rows.map((row) => row.report),
        null,
        2,
      )}\n`,
      "application/json",
    );
  }

  function copyLink(): void {
    navigator.clipboard
      .writeText(window.location.href)
      .then(() => {
        copied = true;
        setTimeout(() => {
          copied = false;
        }, 2000);
      })
      .catch(() => {
        copied = false;
      });
  }
</script>

{#snippet cell(row: ReportRow, column: Column<ReportRow>)}
  {#if column.id === "about"}
    <span class="about">
      {#if row.report.kind === "value" && row.report.property}
        <a href={link({ page: "property", key: row.report.property })}>{row.about}</a>
      {:else}
        {row.about}
      {/if}
    </span>
  {:else if column.id === "reported"}
    {@const reading = row.report.body.type === "value" ? imageReading(row.report.body.value) : null}
    <a href={link({ page: "report", id: row.report.id })}>{row.reported}</a>
    {#if reading}<span class="chip" title={reading.explanation}>{reading.label}</span>{/if}
  {:else if column.id === "converted"}
    {row.converted === null ? "" : `${formatNumber(row.converted)} ${row.unit}`}
  {:else if column.id === "conditions"}
    {@const text = row.conditions.join(" \u{b7} ")}
    <span class="clamp" title={text}>{text}</span>
  {:else if column.id === "source"}
    <a href={link({ page: "work", id: row.report.work, report: row.report.id })}>{row.citation}</a>
  {:else if column.id === "identifier"}
    <span class="mono">{row.identifier}</span>
  {:else if column.id === "check"}
    <CheckBadge state={row.report.check} />
  {:else if column.id === "notices"}
    <NoticeChips kinds={row.notices} />
  {:else if column.id === "quote"}
    <span class="clamp" title={row.quote}>{row.quote}</span>
  {:else if column.id === "id"}
    <a href={link({ page: "report", id: row.report.id })}>{row.report.id}</a>
  {:else}
    {column.value(row) ?? ""}
  {/if}
{/snippet}

<h1>Reports</h1>
<p class="lede">
  Each row is one report of one source: a value or an effect, with its conditions. Find reports by
  words, filter them, sort the table by a column, and download the rows that you select. arity does
  not combine or weigh the reports. The address of the page keeps your selection, so you can share
  it.
</p>

{#if failure}
  <p>arity cannot load the reports: {failure}</p>
{:else if !loaded || !filtered || !page}
  <p class="muted">arity is loading the reports.</p>
{:else}
  {@const data = loaded}
  <div class="explorer">
    <FilterPanel
      facets={REPORT_FACETS}
      titles={FACET_TITLES}
      counts={filtered.counts}
      {selection}
      label={data.label}
      onchange={(next: Selection<ReportFacet>) => {
        show(next);
      }}
    />

    <section class="results" aria-label="Reports">
      <input
        type="search"
        class="find"
        aria-label="Find reports"
        placeholder="Find reports, for example: doxorubicin MCF-7, or &quot;cell viability&quot;"
        value={selection.text}
        oninput={(event) => {
          show({ ...selection, text: event.currentTarget.value });
        }}
      />
      <SelectionChips
        {selection}
        facets={REPORT_FACETS}
        titles={FACET_TITLES}
        label={data.label}
        rangeLabel="Converted value"
        onchange={(next: Selection<ReportFacet>) => {
          show(next);
        }}
      />
      {#if property?.unit}
        <RangeFields
          legend="Converted value of {property.label}, in {unitLabel(property.unit) || '1'}"
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
              saveCsv(ordered(filtered.items, REPORT_COLUMNS, view));
            }}>Download CSV</button
          >
          <button
            type="button"
            onclick={() => {
              saveJson(ordered(filtered.items, REPORT_COLUMNS, view));
            }}>Download JSON</button
          >
          <button type="button" onclick={copyLink}>{copied ? "Copied" : "Copy link"}</button>
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
          label="Reports"
          key={(row: ReportRow) => row.report.id}
          {cell}
          onview={(next: TableView) => {
            show(selection, next);
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
    gap: 0.5rem;
    margin-left: auto;
  }

  .about {
    display: block;
    min-width: 12ch;
  }

  .clamp {
    display: -webkit-box;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
    min-width: 20ch;
    max-width: 40ch;
  }

  @media (max-width: 960px) {
    .explorer {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
