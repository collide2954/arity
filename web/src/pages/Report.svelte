<script lang="ts">
  import { docsLink } from "../lib/docs";
  import CheckBadge from "../components/CheckBadge.svelte";
  import CheckControls from "../components/CheckControls.svelte";
  import ConditionList from "../components/ConditionList.svelte";
  import Notices from "../components/Notices.svelte";
  import ReportCell from "../components/ReportCell.svelte";
  import { checking } from "../lib/checking.svelte";
  import { loadIndex, loadReplaced, loadReports, loadWork } from "../lib/data";
  import { conditionParts, entityName, headline, names, propertyName } from "../lib/describe";
  import {
    directionLabel,
    formatNumber,
    groupsLabel,
    imageReading,
    reportedValue,
    scoreLabel,
    significanceLabel,
    uncertaintyLabel,
    unitLabel,
  } from "../lib/format";
  import { relatedReports } from "../lib/related";
  import { REPORT_COLUMNS, reportRows } from "../lib/reports";
  import { link } from "../lib/route";
  import { agentName, runState } from "../lib/runs";
  import type { Index, ReplacedReport, Report } from "../lib/types";

  let { id }: { id: number } = $props();

  const RELATED = ["subject", "reported", "converted", "conditions", "source", "check"].flatMap(
    (key) => REPORT_COLUMNS.filter((column) => column.id === key),
  );

  const data = Promise.all([loadIndex(), loadReports()]);
</script>

{#await data}
  <p class="muted">arity is loading the report.</p>
{:then [index, reports]}
  {@const current = reports.find((candidate) => candidate.id === id)}
  {#if current}
    {@render content(index, reports, current, null)}
  {:else}
    {#await loadReplaced()}
      <p class="muted">arity is loading the report.</p>
    {:then replaced}
      {@const earlier = replaced.find((candidate) => candidate.id === id)}
      {#if earlier}
        {@render content(index, reports, earlier, earlier)}
      {:else}
        <h1>The report does not exist</h1>
        <p>arity has no report {id}.</p>
      {/if}
    {:catch error}
      <p>arity cannot load the report: {error instanceof Error ? error.message : String(error)}</p>
    {/await}
  {/if}
{:catch error}
  <p>arity cannot load the report: {error instanceof Error ? error.message : String(error)}</p>
{/await}

{#snippet content(index: Index, reports: Report[], report: Report, replaced: ReplacedReport | null)}
  {@const lookup = names(index)}
  {@const body = report.body}
  <h1>{headline(report, lookup)}</h1>
  {#if replaced}
    <aside class="correction" aria-label="Correction">
      <p>
        <strong>A later run corrected this report.</strong> Run {replaced.replaced_by} replaces run
        {report.run}. arity keeps this report so that links to it continue to work, and leaves it
        out of the lists and the figures.
      </p>
      <p>
        {#if replaced.successor !== null}
          <a href={link({ page: "report", id: replaced.successor })}>See the current report</a>
        {:else}
          <a href={link({ page: "work", id: report.work, report: null })}
            >See the current reports of the work</a
          >
        {/if}
        · <a href={docsLink("submissions", "corrections")}>How corrections work</a>
      </p>
    </aside>
  {:else}
    <p class="controls">
      <CheckBadge state={report.check} />
      <a href={link({ page: "work", id: report.work, report: report.id })}
        >Show at its place in the source</a
      >
    </p>
    {@const work = index.works.find((candidate) => candidate.id === report.work)}
    {#if checking.on && work}
      <CheckControls {report} {work} />
    {/if}
  {/if}

  <section class="card">
    <h2>What the source reports</h2>
    <dl>
      {#if body.type === "value"}
        <dt>Property</dt>
        <dd>
          <a href={link({ page: "property", key: body.property })}
            >{propertyName(body.property, lookup)}</a
          >
        </dd>
        <dt>Subject</dt>
        <dd>
          {#each body.subject as local, position (local)}
            {#if position > 0}
              +
            {/if}
            {@const entity = report.entities[local]}
            {#if entity === undefined}{local}{:else}<a href={link({ page: "entity", id: entity })}
                >{entityName(report, local, lookup)}</a
              >{/if}
          {/each}
        </dd>
        <dt>As reported</dt>
        <dd><strong>{reportedValue(body.value)}</strong></dd>
        {#if uncertaintyLabel(body.value)}
          <dt>Uncertainty</dt>
          <dd>{uncertaintyLabel(body.value)}</dd>
        {/if}
        {#if body.value.statistic}<dt>Statistic</dt>
          <dd>{body.value.statistic}</dd>{/if}
        {#if body.value.n !== undefined}<dt>Number of samples</dt>
          <dd>{body.value.n}</dd>{/if}
        {@const reading = imageReading(body.value)}
        {#if reading && body.value.read_from_figure}
          <dt>{reading.label}</dt>
          <dd>
            {reading.explanation} Method: "{body.value.read_from_figure.method}". Resolution:
            {body.value.read_from_figure.resolution}. Image: {body.value.read_from_figure.figure}.
          </dd>
        {/if}
        {#if report.display?.value !== null && report.display?.value !== undefined}
          <dt>Calculated</dt>
          <dd>
            {formatNumber(report.display.value)}
            {unitLabel(report.display.unit)}
            <span class="muted"
              >(arity converts the reported value to the unit of the property)</span
            >
          </dd>
        {/if}
      {:else}
        <dt>Agents</dt>
        <dd>
          {body.agents
            .map((agent) => {
              const name = entityName(report, agent.entity, lookup);
              return agent.dose ? `${name} (${reportedValue(agent.dose)})` : name;
            })
            .join(" + ")}
        </dd>
        <dt>Target</dt>
        <dd>
          {entityName(report, body.target.entity, lookup)}
          {#if body.target.property}
            · {propertyName(body.target.property, lookup)}{/if}
        </dd>
        <dt>Direction</dt>
        <dd><strong>{directionLabel(body.direction)} ({scoreLabel(report.score)})</strong></dd>
        <dt>Relation</dt>
        <dd>{body.relation === "association" ? "association" : "effect"}</dd>
        {#if body.magnitude}
          <dt>Size</dt>
          <dd>
            {body.magnitude.measure.replaceAll("_", " ")}: {body.magnitude.number}
            {body.magnitude.unit_text ?? ""}
          </dd>
        {/if}
        {#if body.comparator}<dt>Compared with</dt>
          <dd>{body.comparator}</dd>{/if}
        {#if significanceLabel(body.significance)}
          <dt>Significance</dt>
          <dd>
            {significanceLabel(body.significance)}{#if body.significance?.test}&nbsp;({body
                .significance.test}){/if}
          </dd>
        {/if}
        {#if body.n !== undefined}<dt>Number of samples</dt>
          <dd>{body.n}</dd>{/if}
        {#if groupsLabel(body.groups)}<dt>Groups</dt>
          <dd>{groupsLabel(body.groups)}</dd>{/if}
      {/if}
      {#if conditionParts(report, lookup).length > 0}
        <dt>Conditions</dt>
        <dd><ConditionList parts={conditionParts(report, lookup)} /></dd>
      {/if}
      {#if body.method}<dt>Method</dt>
        <dd>{body.method.label}</dd>{/if}
      {#if body.note}<dt>Note of the reader</dt>
        <dd>{body.note}</dd>{/if}
    </dl>
  </section>

  <section>
    <h2>Evidence</h2>
    <p class="secondary">
      The checks of arity found each quote at this place in the canonical text of the source. <a
        href={docsLink("checks")}>What arity checks</a
      >.
    </p>
    <ol class="evidence">
      {#each report.evidence as evidence, position (position)}
        <li>
          <blockquote>{evidence.quote}</blockquote>
          <span class="muted"
            >block <span class="mono">{evidence.block}</span>, characters {evidence.start}
            to {evidence.end}</span
          >
        </li>
      {/each}
    </ol>
  </section>

  {@const related = reportRows(index, relatedReports(report, reports))}
  <section>
    <h2>The same in other works ({related.length})</h2>
    {#if related.length === 0}
      <p class="muted">
        No other work reports {body.type === "value"
          ? "this property for this subject"
          : "an effect of these agents on this target"}.
      </p>
    {:else}
      <p class="secondary">
        {body.type === "value"
          ? "Reports of other works with the same property and a subject in common."
          : "Effects that other works report, with the same target and an agent in common."}
        arity does not combine them.
      </p>
      <div class="table-wrap card">
        <table>
          <thead>
            <tr>
              {#each RELATED as column (column.id)}<th scope="col">{column.label}</th>{/each}
            </tr>
          </thead>
          <tbody>
            {#each related as row (row.report.id)}
              <tr>
                {#each RELATED as column (column.id)}
                  <td><ReportCell {row} {column} /></td>
                {/each}
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </section>

  <section>
    <h2>Checks by people</h2>
    {#if report.checks.length === 0}
      <p class="muted">No person has compared this report with its source yet.</p>
    {:else}
      <ul>
        {#each report.checks as check (check.at + check.person)}
          <li>
            <CheckBadge state={check.state} /> by {check.person}{#if check.orcid}
              (<a href="https://orcid.org/{check.orcid}">ORCID</a>){/if} on {check.at}{#if check.note}:
              "{check.note}"{/if}
          </li>
        {/each}
      </ul>
    {/if}
  </section>

  {#await loadWork(report.work)}
    <p class="muted">arity is loading the provenance.</p>
  {:then page}
    {@const run = page.runs.find((candidate) => candidate.id === report.run)}
    <Notices notices={page.work.notices} />
    <section>
      <h2>Provenance</h2>
      <p class="secondary">
        <a href={docsLink("provenance")}>What arity records about each step</a>.
      </p>
      <dl class="card">
        <dt>Source</dt>
        <dd>
          <a href={link({ page: "work", id: report.work, report: null })}
            >{page.work.metadata.title}</a
          >
          <div class="muted">
            {page.attribution.citation} · {page.attribution.license ?? "license not stated"}
          </div>
        </dd>
        <dt>Canonical text</dt>
        <dd class="mono">{run?.text_sha256 ?? ""}</dd>
        <dt>Parser</dt>
        <dd>{run?.parser ?? ""}</dd>
        {#if run}
          <dt>Run</dt>
          <dd>{run.id}: {runState(run)}</dd>
          {#if run.note}
            <dt>Note of the reader on the run</dt>
            <dd>{run.note}</dd>
          {/if}
          <dt>Read by</dt>
          <dd>
            {agentName(run.reader)}
            <span class="muted">({run.reader.kind === "software" ? "software" : "person"})</span>
          </dd>
          <dt>Responsible person</dt>
          <dd>
            {run.responsible.name}{#if run.responsible.orcid}&nbsp;(<a
                href="https://orcid.org/{run.responsible.orcid}">ORCID</a
              >){/if}
          </dd>
          <dt>Submitted</dt>
          <dd>{run.submitted_at}, checked by {run.software}</dd>
          <dt>Submission</dt>
          <dd class="mono">{run.submission_sha256}</dd>
          {#if report.warnings.length > 0}
            <dt>Warnings of the checks</dt>
            <dd>
              <ul>
                {#each report.warnings as warning (warning.path + warning.code)}
                  <li>{warning.message} <span class="mono muted">{warning.path}</span></li>
                {/each}
              </ul>
            </dd>
          {/if}
        {/if}
      </dl>
    </section>
    <details class="submitted">
      <summary>The report as the reader submitted it</summary>
      <pre class="mono">{JSON.stringify(report.body, null, 2)}</pre>
    </details>
  {/await}
{/snippet}

<style>
  dl {
    display: grid;
    grid-template-columns: max-content minmax(0, 1fr);
    gap: 0.35rem 1rem;
    margin: 0;
  }

  dt {
    color: var(--secondary);
  }

  dd {
    margin: 0;
  }

  blockquote {
    margin: 0.25rem 0;
    padding-left: 0.75rem;
    border-left: 3px solid var(--accent);
  }

  .evidence li {
    margin-bottom: 0.75rem;
  }

  .submitted {
    margin-top: 1.75rem;
  }

  .submitted summary {
    cursor: pointer;
    font-weight: 600;
  }

  pre {
    white-space: pre-wrap;
    background: var(--surface);
    padding: 0.75rem;
    border-radius: 6px;
  }

  @media (max-width: 640px) {
    dl {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
