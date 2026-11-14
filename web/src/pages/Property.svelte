<script lang="ts">
  import CheckBadge from "../components/CheckBadge.svelte";
  import Chart from "../components/Chart.svelte";
  import NoticeChips from "../components/NoticeChips.svelte";
  import { valueSpec } from "../lib/charts";
  import { download, toCsv } from "../lib/csv";
  import { loadIndex, loadReports } from "../lib/data";
  import { formatNumber, reportedValue, unitLabel } from "../lib/format";
  import { link } from "../lib/route";
  import { theme } from "../lib/state.svelte";
  import type { Report } from "../lib/types";
  import { figureRows, valueRows, type ValueTableRow } from "../lib/values";

  let { key }: { key: string } = $props();

  let logarithmic = $state(false);
  let showRejected = $state(false);

  const data = Promise.all([loadIndex(), loadReports()]);

  function save(rows: readonly ValueTableRow[], unit: string): void {
    download(
      `arity-${key}.csv`,
      toCsv(
        [
          "report",
          "source",
          "subject",
          "conditions",
          "as_reported",
          `value_in_${unit}`,
          "interval_low",
          "interval_high",
          "read_from_figure",
          "check",
          "notices",
        ],
        rows.map((row) => [
          row.report.id,
          row.citation,
          row.subject,
          row.conditions.join("; "),
          reportedValue(row.body.value),
          row.report.display?.value,
          row.report.display?.interval_low,
          row.report.display?.interval_high,
          row.body.value.read_from_figure !== undefined,
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

{#await data}
  <p class="muted">arity is loading the values.</p>
{:then [index, reports]}
  {@const property = index.properties.find((candidate) => candidate.key === key)}
  {#if !property}
    <h1>The property does not exist</h1>
    <p>arity has no property <span class="mono">{key}</span>.</p>
  {:else}
    {@const rows = valueRows(index, reports, key, showRejected)}
    {@const unit = property.unit ? unitLabel(property.unit) || "1" : ""}
    {@const plotted = figureRows(rows)}
    {@const positive = plotted.every((row) => row.value > 0 && (row.low ?? 1) > 0)}
    <h1>{property.label}</h1>
    <p class="secondary">
      Key <span class="mono">{property.key}</span>
      {#if property.unit}· values in {unit} · dimension
        <span class="mono">{property.dimension}</span>{/if}
    </p>
    <p class="lede">
      Each dot is one report of one source. arity does not average or weigh the reports. A filled
      dot is a report that a person confirmed; a hollow dot is a report that no person checked yet.
      A diamond is a value that the reader read from a plot. If the source has a notice, for example
      a retraction, the label starts with the name of the notice.
    </p>
    <div class="controls">
      <button
        type="button"
        aria-pressed={logarithmic && positive}
        disabled={!positive}
        onclick={() => {
          logarithmic = !logarithmic;
        }}>Logarithmic scale</button
      >
      <button
        type="button"
        aria-pressed={showRejected}
        onclick={() => {
          showRejected = !showRejected;
        }}>Show rejected reports</button
      >
      <button
        type="button"
        onclick={() => {
          save(rows, property.unit ?? "1");
        }}>Download CSV</button
      >
    </div>
    {#if plotted.length > 0}
      <div class="card">
        <Chart
          spec={valueSpec(plotted, {
            unit: unit || "value",
            logarithmic: logarithmic && positive,
            mode: theme.mode,
          })}
          label="The values of {property.label} in each report"
          onselect={(report: number) => {
            open(report, reports);
          }}
        />
      </div>
    {/if}
    <h2>Table</h2>
    <div class="table-wrap card">
      <table>
        <thead>
          <tr>
            <th>Source</th>
            <th>Subject</th>
            <th>Conditions</th>
            <th>As reported</th>
            {#if property.unit}<th>In {unit}</th>{/if}
            <th>Check</th>
            <th>Notices</th>
          </tr>
        </thead>
        <tbody>
          {#each rows as row (row.report.id)}
            <tr>
              <td>
                <a href={link({ page: "work", id: row.report.work, report: row.report.id })}
                  >{row.citation}</a
                >
              </td>
              <td>{row.subject}</td>
              <td>{row.conditions.join(" · ")}</td>
              <td>
                {reportedValue(row.body.value)}{#if row.body.value.read_from_figure}
                  <span class="chip">Read from figure</span>{/if}
              </td>
              {#if property.unit}
                <td>
                  {row.report.display?.value === null || row.report.display?.value === undefined
                    ? ""
                    : formatNumber(row.report.display.value)}
                </td>
              {/if}
              <td><CheckBadge state={row.report.check} /></td>
              <td><NoticeChips kinds={row.notices} /></td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
{/await}
