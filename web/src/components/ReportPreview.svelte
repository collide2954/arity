<script lang="ts">
  import { noticeNames } from "../lib/notices";
  import ConditionList from "./ConditionList.svelte";
  import type { ReportRow } from "../lib/reports";
  import { link } from "../lib/route";

  let { row }: { row: ReportRow } = $props();

  const report = $derived(row.report);
</script>

<div class="preview">
  {#if report.evidence.length === 0}
    <p class="muted">The report has no quotes.</p>
  {:else}
    <ol>
      {#each report.evidence as evidence, position (position)}
        <li>
          <blockquote>{evidence.quote}</blockquote>
          <span class="muted small">block <span class="mono">{evidence.block}</span></span>
        </li>
      {/each}
    </ol>
  {/if}
  <dl>
    {#if row.conditionParts.length > 0}
      <dt>Conditions</dt>
      <dd><ConditionList parts={row.conditionParts} /></dd>
    {/if}
    {#if row.method}
      <dt>Method</dt>
      <dd>{row.method}</dd>
    {/if}
    {#if report.body.note}
      <dt>Note of the reader</dt>
      <dd>{report.body.note}</dd>
    {/if}
    {#if row.notices.length > 0}
      <dt>Notices of the source</dt>
      <dd>{noticeNames(row.notices)}</dd>
    {/if}
  </dl>
  <p class="links">
    <a href={link({ page: "work", id: report.work, report: report.id })}
      >Show at its place in the source</a
    >
    <a href={link({ page: "report", id: report.id })}>Details and provenance</a>
  </p>
</div>

<style>
  .preview {
    display: grid;
    gap: 0.5rem;
    max-width: 90ch;
  }

  ol {
    margin: 0;
    padding-left: 1.25rem;
    display: grid;
    gap: 0.5rem;
  }

  blockquote {
    margin: 0;
    padding-left: 0.75rem;
    border-left: 3px solid var(--accent);
  }

  dl {
    display: grid;
    grid-template-columns: max-content minmax(0, 1fr);
    gap: 0.2rem 1rem;
    margin: 0;
  }

  dt {
    color: var(--secondary);
  }

  dd {
    margin: 0;
  }

  .links {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
    margin: 0;
  }

  .small {
    font-size: 0.85rem;
  }
</style>
