<script lang="ts">
  import ReportItem from "../components/ReportItem.svelte";
  import { loadIndex, loadReplaced, loadReports } from "../lib/data";
  import { names } from "../lib/describe";
  import { docsLink } from "../lib/docs";
  import { noticesOfWorks } from "../lib/notices";
  import { link, type Route } from "../lib/route";
  import type { Index, Report } from "../lib/types";

  let { id }: { id: number } = $props();

  const data = Promise.all([loadIndex(), loadReports()]);

  function uses(report: Report): boolean {
    return Object.values(report.entities).includes(id);
  }
</script>

{#await data}
  <p class="muted">arity is loading the entity.</p>
{:then [index, reports]}
  {@const entity = [...index.entities, ...index.former_entities].find(
    (candidate) => candidate.id === id,
  )}
  {#if !entity}
    <h1>The entity does not exist</h1>
    <p>arity has no entity {id}.</p>
  {:else}
    <h1>{entity.label}</h1>
    <p class="secondary">{entity.kind.replaceAll("_", " ")}</p>
    <p class="chips">
      {#each entity.xrefs as xref (xref)}
        <a class="chip mono" href="https://bioregistry.io/{xref}">{xref}</a>
      {/each}
    </p>
    {#if entity.names.length > 0}
      <p class="secondary">The sources name it: {entity.names.join(", ")}.</p>
    {/if}
    {#if entity.current}
      {@const related = reports.filter(uses)}
      <h2>Reports about {entity.label} ({related.length})</h2>
      {@render list(index, related, (report) => ({
        page: "work",
        id: report.work,
        report: report.id,
      }))}
    {:else}
      <aside class="correction" aria-label="Correction">
        <p>
          <strong>No current run names this entity.</strong> Later runs corrected the runs that named
          it. arity keeps the entity so that links to it continue to work, and leaves it out of the lists.
        </p>
        <p><a href={docsLink("submissions", "corrections")}>How corrections work</a></p>
      </aside>
      {#await loadReplaced()}
        <p class="muted">arity is loading the reports of the replaced runs.</p>
      {:then replaced}
        {@const earlier = replaced.filter(uses)}
        <h2>Reports of replaced runs about {entity.label} ({earlier.length})</h2>
        {@render list(index, earlier, (report) => ({ page: "report", id: report.id }))}
      {:catch error}
        <p>
          arity cannot load the reports of the replaced runs: {error instanceof Error
            ? error.message
            : String(error)}
        </p>
      {/await}
    {/if}
  {/if}
{:catch error}
  <p>arity cannot load the entity: {error instanceof Error ? error.message : String(error)}</p>
{/await}

{#snippet list(index: Index, related: Report[], target: (report: Report) => Route)}
  {@const lookup = names(index)}
  {@const noticesOf = noticesOfWorks(index.works)}
  {@const works = new Map(index.works.map((work) => [work.id, work]))}
  <div class="list">
    {#each related as report (report.id)}
      <ReportItem
        {report}
        {lookup}
        work={works.get(report.work)}
        notices={noticesOf(report.work)}
        onselect={() => {
          window.location.hash = link(target(report));
        }}
      />
    {/each}
  </div>
{/snippet}

<style>
  .list {
    display: grid;
    gap: 0.5rem;
    max-width: 60rem;
  }
</style>
