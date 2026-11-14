<script lang="ts">
  import CheckBadge from "../components/CheckBadge.svelte";
  import Chart from "../components/Chart.svelte";
  import NoticeChips from "../components/NoticeChips.svelte";
  import { effectSpec } from "../lib/charts";
  import { download, toCsv } from "../lib/csv";
  import { loadIndex, loadReports } from "../lib/data";
  import { effectRows, matrixRows, type EffectTableRow } from "../lib/effects";
  import { directionLabel, scoreLabel } from "../lib/format";
  import { link } from "../lib/route";
  import { theme } from "../lib/state.svelte";
  import type { Report } from "../lib/types";

  const data = Promise.all([loadIndex(), loadReports()]);

  function save(rows: readonly EffectTableRow[]): void {
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

<h1>Effects</h1>
<p class="lede">
  Each mark is one effect or association that one source reports. Up is an increase (+1), down is a
  decrease (−1), and a dot is a statement of no change or no effect (0). Two sources that disagree
  show as two marks in the same cell: arity does not combine them. If the source has a notice, for
  example a retraction, a sign shows above the mark.
</p>

{#await data}
  <p class="muted">arity is loading the effects.</p>
{:then [index, reports]}
  {@const rows = effectRows(index, reports)}
  {#if rows.length === 0}
    <p class="muted">No reader has submitted effects yet.</p>
  {:else}
    <div class="controls">
      <button
        type="button"
        onclick={() => {
          save(rows);
        }}>Download CSV</button
      >
    </div>
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
    <div class="table-wrap card">
      <table>
        <thead>
          <tr>
            <th>Agent</th>
            <th>Target</th>
            <th>Direction</th>
            <th>Details</th>
            <th>Conditions</th>
            <th>Source</th>
            <th>Check</th>
            <th>Notices</th>
          </tr>
        </thead>
        <tbody>
          {#each rows as row (row.report.id)}
            <tr>
              <td>{row.agent}</td>
              <td>{row.target}</td>
              <td>
                <strong>{scoreLabel(row.report.score)}</strong>
                {directionLabel(row.body.direction)}
              </td>
              <td>{row.detail}</td>
              <td>{row.context}</td>
              <td>
                <a href={link({ page: "work", id: row.report.work, report: row.report.id })}
                  >{row.citation}</a
                >
              </td>
              <td><CheckBadge state={row.report.check} /></td>
              <td><NoticeChips kinds={row.notices} /></td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
{/await}
