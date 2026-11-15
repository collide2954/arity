<script lang="ts">
  import CopyLink from "../components/CopyLink.svelte";
  import FilterPanel from "../components/FilterPanel.svelte";
  import SelectionChips from "../components/SelectionChips.svelte";
  import {
    dimensions,
    matrixCsv,
    matrixOf,
    readCompare,
    writeCompare,
    type Compare,
  } from "../lib/compare";
  import { download } from "../lib/csv";
  import { loadIndex, loadReports } from "../lib/data";
  import { names, type Names } from "../lib/describe";
  import {
    emptySelection,
    filter,
    readSelection,
    writeSelection,
    type Selection,
  } from "../lib/filter";
  import { formatNumber, unitLabel } from "../lib/format";
  import { markerSign, noticeNames } from "../lib/notices";
  import {
    FACET_TITLES,
    facetLabels,
    reportRows,
    type ReportFacet,
    type ReportRow,
  } from "../lib/reports";
  import { link } from "../lib/route";
  import { compareValues } from "../lib/sort";
  import { replaceRoute } from "../lib/state.svelte";
  import type { Index, PropertySummary, Report } from "../lib/types";

  let { params = "" }: { params?: string } = $props();

  const FACETS = [
    "entity",
    "condition",
    "work",
    "check",
    "notice",
    "origin",
  ] as const satisfies readonly ReportFacet[];
  type Facet = (typeof FACETS)[number];

  const CHECK_SIGNS: Readonly<Record<string, string>> = {
    confirmed: "\u{2713}",
    rejected: "\u{2717}",
  };

  interface Loaded {
    readonly index: Index;
    readonly reports: readonly Report[];
    readonly lookup: Names;
    readonly label: (facet: Facet, value: string) => string;
    readonly properties: readonly PropertySummary[];
  }

  let loaded = $state.raw<Loaded | null>(null);
  let failure = $state<string | null>(null);

  Promise.all([loadIndex(), loadReports()])
    .then(([index, reports]) => {
      const valued = new Set(
        reports.filter((report) => report.kind === "value").map((report) => report.property),
      );
      loaded = {
        index,
        reports,
        lookup: names(index),
        label: facetLabels(index),
        properties: index.properties
          .filter((property) => valued.has(property.key))
          .sort((a, b) => compareValues(a.label, b.label)),
      };
    })
    .catch((error: unknown) => {
      failure = error instanceof Error ? error.message : String(error);
    });

  const query = $derived(new URLSearchParams(params));
  const compare = $derived(readCompare(query));
  const selection = $derived(readSelection(query, FACETS));
  const property = $derived(
    loaded?.properties.find((candidate) => candidate.key === compare.property) ?? null,
  );
  const rows = $derived.by((): readonly ReportRow[] => {
    if (!loaded || !property) return [];
    const own = loaded.reports.filter(
      (report) => report.kind === "value" && report.property === property.key,
    );
    return reportRows(loaded.index, own);
  });
  const filtered = $derived(filter(rows, selection, FACETS));
  const available = $derived(loaded ? dimensions(rows, loaded.lookup) : []);
  const byRow = $derived(
    available.some((dimension) => dimension.key === compare.rows) ? compare.rows : "subject",
  );
  const byColumn = $derived(
    available.some((dimension) => dimension.key === compare.columns) ? compare.columns : "work",
  );
  const matrix = $derived(loaded ? matrixOf(filtered.items, byRow, byColumn, loaded.lookup) : null);
  const titleOf = (key: string) =>
    available.find((dimension) => dimension.key === key)?.label ?? key;
  const converted = $derived(compare.converted && property?.unit !== null);

  function show(next: Compare, chosen: Selection<Facet> = selection): void {
    const out = new URLSearchParams();
    writeCompare(out, next);
    writeSelection(out, chosen, FACETS);
    replaceRoute({ page: "compare", params: out.toString() });
  }

  function shown(row: ReportRow): string {
    return converted && row.converted !== null ? formatNumber(row.converted) : row.reported;
  }

  function described(row: ReportRow): string {
    const notices = noticeNames(row.notices);
    return [
      row.citation,
      ...row.conditions,
      `check: ${row.report.check}`,
      notices ? `notices: ${notices}` : "",
    ]
      .filter((part) => part !== "")
      .join(" \u{b7} ");
  }
</script>

<h1>Compare</h1>
<p class="lede">
  Choose a property, and what the rows and the columns of the matrix are. Each cell lists every
  report of its row and its column, with a link to the report. arity does not average or combine the
  reports of a cell. A check sign marks a confirmed report, a cross marks a rejected report, and a
  warning sign marks a report whose source has a notice.
</p>

{#if failure}
  <p>arity cannot load the reports: {failure}</p>
{:else if !loaded}
  <p class="muted">arity is loading the reports.</p>
{:else}
  <div class="controls">
    <label
      >Property
      <select
        value={compare.property ?? ""}
        onchange={(event) => {
          show({ ...compare, property: event.currentTarget.value }, emptySelection());
        }}
      >
        <option value="" disabled>Select a property</option>
        {#each loaded.properties as option (option.key)}
          <option value={option.key}>{option.label} ({option.reports})</option>
        {/each}
      </select></label
    >
  </div>

  {#if property && matrix}
    {@const data = loaded}
    <div class="explorer">
      <FilterPanel
        facets={FACETS}
        titles={FACET_TITLES}
        counts={filtered.counts}
        {selection}
        label={data.label}
        onchange={(next: Selection<Facet>) => {
          show(compare, next);
        }}
      />
      <section aria-label="Matrix">
        <div class="controls">
          <label
            >Rows
            <select
              value={byRow}
              onchange={(event) => {
                show({ ...compare, rows: event.currentTarget.value });
              }}
            >
              {#each available as dimension (dimension.key)}
                <option value={dimension.key}>{dimension.label}</option>
              {/each}
            </select></label
          >
          <label
            >Columns
            <select
              value={byColumn}
              onchange={(event) => {
                show({ ...compare, columns: event.currentTarget.value });
              }}
            >
              {#each available as dimension (dimension.key)}
                <option value={dimension.key}>{dimension.label}</option>
              {/each}
            </select></label
          >
          <button
            type="button"
            onclick={() => {
              show({ ...compare, rows: byColumn, columns: byRow });
            }}>Swap rows and columns</button
          >
          {#if property.unit}
            <button
              type="button"
              aria-pressed={converted}
              onclick={() => {
                show({ ...compare, converted: !compare.converted });
              }}>Values in {unitLabel(property.unit) || "1"}</button
            >
          {/if}
        </div>
        <input
          type="search"
          class="find"
          aria-label="Find reports of {property.label}"
          placeholder="Find reports by words"
          value={selection.text}
          oninput={(event) => {
            show(compare, { ...selection, text: event.currentTarget.value });
          }}
        />
        <SelectionChips
          {selection}
          facets={FACETS}
          titles={FACET_TITLES}
          label={data.label}
          onchange={(next: Selection<Facet>) => {
            show(compare, next);
          }}
        />
        <div class="controls">
          <strong>{filtered.items.length}</strong>
          <span class="secondary">of {rows.length} reports</span>
          <span class="actions">
            <a href={link({ page: "property", key: property.key })}>Open the property</a>
            <button
              type="button"
              onclick={() => {
                download(
                  `arity-compare-${property.key}.csv`,
                  matrixCsv(matrix, titleOf(byRow), titleOf(byColumn)),
                );
              }}>Download CSV</button
            >
            <CopyLink />
          </span>
        </div>
        {#if filtered.items.length === 0}
          <p class="card muted">
            No report agrees with all filters. Remove a filter, or change the words.
          </p>
        {:else}
          <div class="matrix card">
            <table aria-label="{property.label} by {titleOf(byRow)} and {titleOf(byColumn)}">
              <thead>
                <tr>
                  <th scope="col" class="corner">{titleOf(byRow)} \ {titleOf(byColumn)}</th>
                  {#each matrix.columns as column (column)}
                    <th scope="col">{column}</th>
                  {/each}
                </tr>
              </thead>
              <tbody>
                {#each matrix.rows as label (label)}
                  <tr>
                    <th scope="row">{label}</th>
                    {#each matrix.columns as column (column)}
                      <td>
                        {#each matrix.cells.get(label)?.get(column) ?? [] as row (row.report.id)}
                          <a
                            class="value"
                            class:rejected={row.report.check === "rejected"}
                            href={link({ page: "report", id: row.report.id })}
                            title={described(row)}
                            >{shown(row)}{#if CHECK_SIGNS[row.report.check]}
                              <span aria-label={row.report.check}
                                >{CHECK_SIGNS[row.report.check]}</span
                              >{/if}{#if row.notices.length > 0}
                              <span aria-label={noticeNames(row.notices)}
                                >{markerSign(row.notices)}</span
                              >{/if}</a
                          >
                        {/each}
                      </td>
                    {/each}
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </section>
    </div>
  {/if}
{/if}

<style>
  label {
    display: inline-flex;
    gap: 0.4rem;
    align-items: center;
  }

  select {
    background: var(--surface);
    border: 1px solid var(--axis);
    border-radius: 6px;
    padding: 0.25rem 0.4rem;
    max-width: 22rem;
  }

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

  .matrix {
    overflow: auto;
    max-height: 75vh;
    padding: 0;
  }

  .matrix table {
    width: max-content;
    min-width: 100%;
  }

  .matrix th {
    background: var(--surface);
  }

  .matrix thead th {
    position: sticky;
    top: 0;
    z-index: 1;
    max-width: 16ch;
  }

  .matrix tbody th {
    position: sticky;
    left: 0;
    min-width: 18ch;
    max-width: 32ch;
    overflow-wrap: break-word;
    font-weight: 500;
  }

  .matrix .corner {
    left: 0;
    z-index: 2;
  }

  .value {
    display: block;
    white-space: nowrap;
  }

  .rejected {
    text-decoration: line-through;
  }

  @media (max-width: 960px) {
    .explorer {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
