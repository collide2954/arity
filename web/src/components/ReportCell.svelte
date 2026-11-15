<script lang="ts">
  import { formatNumber, imageReading } from "../lib/format";
  import type { ReportRow } from "../lib/reports";
  import { link } from "../lib/route";
  import type { Column } from "../lib/view";
  import CheckBadge from "./CheckBadge.svelte";
  import NoticeChips from "./NoticeChips.svelte";

  let { row, column }: { row: ReportRow; column: Column<ReportRow> } = $props();

  const report = $derived(row.report);
  const reading = $derived(report.body.type === "value" ? imageReading(report.body.value) : null);
  const value = $derived(column.value(row));
</script>

{#if column.id === "about"}
  <span class="about">
    {#if report.kind === "value" && report.property}
      <a href={link({ page: "property", key: report.property })}>{row.about}</a>
    {:else}
      {row.about}
    {/if}
  </span>
{:else if column.id === "reported"}
  <a href={link({ page: "report", id: report.id })}>{row.reported}</a>
  {#if reading}<span class="chip" title={reading.explanation}>{reading.label}</span>{/if}
{:else if column.id === "converted"}
  {row.converted === null ? "" : `${formatNumber(row.converted)} ${row.unit}`}
{:else if column.numeric && typeof value === "number" && column.id !== "id" && column.id !== "year"}
  {formatNumber(value)}
{:else if column.id === "conditions"}
  {@const text = row.conditions.join(" \u{b7} ")}
  <span class="clamp" title={text}>{text}</span>
{:else if column.id === "source"}
  <a href={link({ page: "work", id: report.work, report: report.id })}>{row.citation}</a>
{:else if column.id === "identifier"}
  <span class="mono">{row.identifier}</span>
{:else if column.id === "check"}
  <CheckBadge state={report.check} />
{:else if column.id === "notices"}
  <NoticeChips kinds={row.notices} />
{:else if column.id === "quote"}
  <span class="clamp" title={row.quote}>{row.quote}</span>
{:else if column.id === "id"}
  <a href={link({ page: "report", id: report.id })}>{report.id}</a>
{:else}
  {value ?? ""}
{/if}

<style>
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
</style>
