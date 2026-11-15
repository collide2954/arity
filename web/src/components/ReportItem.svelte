<script lang="ts">
  import { checking } from "../lib/checking.svelte";
  import { conditions, headline, type Names } from "../lib/describe";
  import { imageReading } from "../lib/format";
  import { link } from "../lib/route";
  import type { Report, WorkSummary } from "../lib/types";
  import CheckBadge from "./CheckBadge.svelte";
  import CheckControls from "./CheckControls.svelte";
  import NoticeChips from "./NoticeChips.svelte";

  let {
    report,
    lookup,
    work,
    notices = [],
    selected = false,
    located = true,
    onselect,
  }: {
    report: Report;
    lookup: Names;
    /** The work of the report. With it, check mode shows the controls of a check. */
    work?: WorkSummary | undefined;
    notices?: readonly string[];
    selected?: boolean;
    located?: boolean;
    onselect?: (report: number) => void;
  } = $props();

  const reading = $derived(report.body.type === "value" ? imageReading(report.body.value) : null);
</script>

<article class:selected id="report-{report.id}">
  <button class="headline" type="button" onclick={() => onselect?.(report.id)}>
    {headline(report, lookup)}
  </button>
  <div class="details">
    <CheckBadge state={report.check} />
    <NoticeChips kinds={notices} />
    {#if reading}
      <span class="chip" title={reading.explanation}>{reading.label}</span>
    {/if}
    {#each conditions(report, lookup) as condition (condition)}
      <span class="chip">{condition}</span>
    {/each}
  </div>
  {#if !located}
    <p class="muted small">arity cannot find this quote in the PDF. The text view shows it.</p>
  {/if}
  <a class="small" href={link({ page: "report", id: report.id })}>Details and provenance</a>
  {#if checking.on && work}
    <CheckControls {report} {work} />
  {/if}
</article>

<style>
  article {
    padding: 0.6rem 0.75rem;
    border-radius: 6px;
    box-shadow: 0 0 0 1px var(--ring);
    background: var(--surface);
    display: grid;
    gap: 0.35rem;
  }

  article.selected {
    box-shadow: 0 0 0 2px var(--accent);
  }

  .headline {
    all: unset;
    cursor: pointer;
    font-weight: 600;
  }

  .headline:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .details {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
  }

  .small {
    font-size: 0.85rem;
    margin: 0;
  }
</style>
