<script lang="ts">
  import ReportItem from "../components/ReportItem.svelte";
  import { loadIndex, loadReports } from "../lib/data";
  import { names } from "../lib/describe";
  import { noticesOfWorks } from "../lib/notices";
  import { link } from "../lib/route";

  let { id }: { id: number } = $props();

  const data = Promise.all([loadIndex(), loadReports()]);

  function open(report: number, work: number): void {
    window.location.hash = link({ page: "work", id: work, report });
  }
</script>

{#await data}
  <p class="muted">arity is loading the entity.</p>
{:then [index, reports]}
  {@const entity = index.entities.find((candidate) => candidate.id === id)}
  {#if !entity}
    <h1>The entity does not exist</h1>
    <p>arity has no entity {id}.</p>
  {:else}
    {@const lookup = names(index)}
    {@const noticesOf = noticesOfWorks(index.works)}
    {@const related = reports.filter((report) => Object.values(report.entities).includes(id))}
    {@const works = new Map(index.works.map((work) => [work.id, work]))}
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
    <h2>Reports about {entity.label} ({related.length})</h2>
    <div class="list">
      {#each related as report (report.id)}
        <ReportItem
          {report}
          {lookup}
          work={works.get(report.work)}
          notices={noticesOf(report.work)}
          onselect={(selected: number) => {
            open(selected, report.work);
          }}
        />
      {/each}
    </div>
  {/if}
{/await}

<style>
  .list {
    display: grid;
    gap: 0.5rem;
    max-width: 60rem;
  }
</style>
