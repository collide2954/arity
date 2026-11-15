<script lang="ts">
  import { loadIndex, loadReports } from "../lib/data";
  import { headline, names } from "../lib/describe";
  import { emptySelection, filter, selectionQuery } from "../lib/filter";
  import { findInCatalogue } from "../lib/find";
  import { REPORT_FACETS, reportRows, type ReportRow } from "../lib/reports";
  import { link } from "../lib/route";
  import type { Index } from "../lib/types";

  /** The number of reports that the start page lists for a search. */
  const FIRST = 5;

  let query = $state("");
  let index = $state.raw<Index | null>(null);
  let failure = $state<string | null>(null);
  let rows = $state.raw<readonly ReportRow[] | null>(null);
  let asked = false;

  loadIndex()
    .then((loaded) => {
      index = loaded;
    })
    .catch((error: unknown) => {
      failure = error instanceof Error ? error.message : String(error);
    });

  const found = $derived(index ? findInCatalogue(index, query) : null);
  const lookup = $derived(index ? names(index) : null);
  const reports = $derived(
    found && rows ? filter(rows, { ...emptySelection(), text: query }, REPORT_FACETS).items : null,
  );
  const empty = $derived(
    found !== null &&
      reports !== null &&
      found.works.length + found.properties.length + found.entities.length + reports.length === 0,
  );

  $effect(() => {
    const catalogue = index;
    if (!found || !catalogue || asked) return;
    asked = true;
    loadReports()
      .then((loaded) => {
        rows = reportRows(catalogue, loaded);
      })
      .catch(() => {
        asked = false;
      });
  });
</script>

<h1>Values that open-access research reports, with their sources</h1>
<p class="lede">
  arity collects the values and effects that open-access research articles report. Each value keeps
  its source: the article, the place in the article, and the exact text. arity does not rate,
  combine, or correct the data. It shows what each source states, and each notice about the source,
  for example a retraction.
</p>

{#if failure}
  <p>arity cannot load the catalogue: {failure}</p>
{:else if !index}
  <p class="muted">arity is loading the catalogue.</p>
{:else}
  {@const total = index.works.reduce((sum, work) => sum + work.reports, 0)}
  <div class="stats">
    <a class="card stat" href={link({ page: "works" })}
      ><strong>{index.works.length}</strong><span>works</span></a
    >
    <a class="card stat" href={link({ page: "reports" })}
      ><strong>{total}</strong><span>reports</span></a
    >
    <a class="card stat" href={link({ page: "properties" })}
      ><strong>{index.properties.length}</strong><span>properties</span></a
    >
    <a class="card stat" href={link({ page: "entities" })}
      ><strong>{index.entities.length}</strong><span>entities</span></a
    >
  </div>

  <label for="search"><h2>Find a work, a property, an entity, or a report</h2></label>
  <input
    id="search"
    type="search"
    placeholder="For example: doxorubicin, IC50, MCF-7, or a quote"
    bind:value={query}
  />
  {#if found}
    <div class="results">
      {#each found.properties as property (property.key)}
        <a href={link({ page: "property", key: property.key })}
          >{property.label} <span class="muted">property · {property.reports} reports</span></a
        >
      {/each}
      {#each found.entities as entity (entity.id)}
        <a href={link({ page: "entity", id: entity.id })}
          >{entity.label}
          <span class="muted">{entity.kind.replaceAll("_", " ")} · {entity.reports} reports</span
          ></a
        >
      {/each}
      {#each found.works as work (work.id)}
        <a href={link({ page: "work", id: work.id, report: null })}
          >{work.title} <span class="muted">{work.citation}</span></a
        >
      {/each}
      {#if reports === null}
        <p class="muted">arity is loading the reports.</p>
      {:else if reports.length > 0 && lookup}
        <h3>
          {reports.length}
          {reports.length === 1 ? "report has" : "reports have"} these words
        </h3>
        {#each reports.slice(0, FIRST) as row (row.report.id)}
          <a href={link({ page: "report", id: row.report.id })}
            >{headline(row.report, lookup)} <span class="muted">{row.citation}</span></a
          >
        {/each}
        <a
          class="all"
          href={link({
            page: "reports",
            params: selectionQuery({ ...emptySelection(), text: query }, REPORT_FACETS),
          })}
          >Show {reports.length > FIRST ? `all ${String(reports.length)}` : "them"} in the report explorer</a
        >
      {/if}
      {#if empty}
        <p class="muted">Nothing has the words "{query.trim()}".</p>
      {/if}
    </div>
  {/if}

  <h2>How to read the data</h2>
  <ul class="secondary">
    <li>Each report is one statement of one source, with the quote that supports it.</li>
    <li>
      A person compares each report with its source. A report that no person has checked has the
      label "Not checked".
    </li>
    <li>
      A statement of no effect shows as 0 in the effect matrix. It is a report of no effect, not a
      measured value of zero.
    </li>
    <li>
      Converted values are calculations of arity from the reported value and unit. Each page also
      shows the value as the source reports it.
    </li>
  </ul>
  <p class="muted">
    Data of {index.generated_at.slice(0, 10)} from {index.software}.
    <a href={`${import.meta.env.BASE_URL}data/${index.database}`}>Download the full database</a> (SQLite).
  </p>
{/if}

<style>
  .stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(9rem, 1fr));
    gap: 0.75rem;
    margin: 1.5rem 0;
  }

  .stat {
    display: grid;
    text-decoration: none;
    color: var(--ink);
  }

  .stat strong {
    font-size: 1.8rem;
    font-weight: 600;
  }

  .stat span {
    color: var(--secondary);
  }

  .results {
    display: grid;
    gap: 0.35rem;
    margin-top: 0.75rem;
  }

  .results h3 {
    margin: 0.75rem 0 0.1rem;
  }

  .all {
    font-weight: 600;
  }
</style>
