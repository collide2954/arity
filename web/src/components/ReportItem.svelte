<script lang="ts">
  import { conditions, headline, type Names } from "../lib/describe";
  import { link } from "../lib/route";
  import type { Report } from "../lib/types";
  import CheckBadge from "./CheckBadge.svelte";

  let {
    report,
    lookup,
    selected = false,
    located = true,
    onselect,
  }: {
    report: Report;
    lookup: Names;
    selected?: boolean;
    located?: boolean;
    onselect?: (report: number) => void;
  } = $props();

  const fromFigure = $derived(
    report.body.type === "value" && report.body.value.read_from_figure !== undefined,
  );
</script>

<article class:selected id="report-{report.id}">
  <button class="headline" type="button" onclick={() => onselect?.(report.id)}>
    {headline(report, lookup)}
  </button>
  <div class="details">
    <CheckBadge state={report.check} />
    {#if fromFigure}
      <span
        class="chip"
        title="The reader read this value from a plot. The source does not state the number."
        >Read from figure</span
      >
    {/if}
    {#each conditions(report, lookup) as condition (condition)}
      <span class="chip">{condition}</span>
    {/each}
  </div>
  {#if !located}
    <p class="muted small">arity cannot find this quote in the PDF. The text view shows it.</p>
  {/if}
  <a class="small" href={link({ page: "report", id: report.id })}>Details and provenance</a>
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
