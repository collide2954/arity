<script lang="ts">
  import { loadIndex, loadReports } from "../lib/data";
  import { headline, names } from "../lib/describe";
  import { emptySelection, filter, selectionQuery } from "../lib/filter";
  import { findInCatalogue } from "../lib/find";
  import { REPORT_FACETS, reportRows, type ReportRow } from "../lib/reports";
  import { docsLink } from "../lib/docs";
  import { link } from "../lib/route";
  import type { Index } from "../lib/types";

  /** The number of reports that the start page lists for a search. */
  const FIRST = 5;
  const EXAMPLES = ["doxorubicin MCF-7", "IC50", "band gap", "retraction"];

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

<section class="hero">
  <p class="eyebrow">Open research data, each value with its source</p>
  <h1>The values that research reports, and where each one comes from</h1>
  <p class="lede">
    arity collects the values and effects that open-access articles report. Each value keeps its
    article, its place in the article, and the exact text. arity does not rate, combine, or correct
    the data, and it shows each notice about a source, for example a retraction.
  </p>
  <label class="search">
    <span class="hidden">Find a work, a property, an entity, or a report</span>
    <svg class="icon" viewBox="0 0 20 20" aria-hidden="true">
      <circle cx="8.5" cy="8.5" r="5.5" />
      <path d="m12.8 12.8 4 4" />
    </svg>
    <input
      type="search"
      placeholder="Find a work, a property, an entity, a value, or a quote"
      bind:value={query}
    />
  </label>
  <p class="examples">
    <span class="muted">For example</span>
    {#each EXAMPLES as example (example)}
      <button
        type="button"
        class="chip"
        onclick={() => {
          query = example;
        }}>{example}</button
      >
    {/each}
  </p>
</section>

{#if failure}
  <p>arity cannot load the catalogue: {failure}</p>
{:else if !index}
  <p class="muted">arity is loading the catalogue.</p>
{:else}
  {#if found}
    <section class="results card" aria-label="Results">
      {#if found.properties.length > 0}
        <h3>Properties</h3>
        <ul>
          {#each found.properties as property (property.key)}
            <li>
              <a href={link({ page: "property", key: property.key })}>{property.label}</a>
              <span class="muted">{property.reports} reports</span>
            </li>
          {/each}
        </ul>
      {/if}
      {#if found.entities.length > 0}
        <h3>Entities</h3>
        <ul>
          {#each found.entities.slice(0, 12) as entity (entity.id)}
            <li>
              <a href={link({ page: "entity", id: entity.id })}>{entity.label}</a>
              <span class="muted"
                >{entity.kind.replaceAll("_", " ")} · {entity.reports} reports</span
              >
            </li>
          {/each}
        </ul>
      {/if}
      {#if found.works.length > 0}
        <h3>Works</h3>
        <ul>
          {#each found.works as work (work.id)}
            <li>
              <a href={link({ page: "work", id: work.id, report: null })}>{work.title}</a>
              <span class="muted">{work.citation}</span>
            </li>
          {/each}
        </ul>
      {/if}
      {#if reports === null}
        <p class="muted">arity is loading the reports.</p>
      {:else if reports.length > 0 && lookup}
        <h3>Reports <span class="muted">{reports.length}</span></h3>
        <ul>
          {#each reports.slice(0, FIRST) as row (row.report.id)}
            <li>
              <a href={link({ page: "report", id: row.report.id })}
                >{headline(row.report, lookup)}</a
              >
              <span class="muted">{row.citation}</span>
            </li>
          {/each}
        </ul>
        <a
          class="all"
          href={link({
            page: "reports",
            params: selectionQuery({ ...emptySelection(), text: query }, REPORT_FACETS),
          })}
          >{reports.length > FIRST ? `Show all ${String(reports.length)} reports` : "Show them"} in the
          report explorer →</a
        >
      {/if}
      {#if empty}
        <p class="muted">Nothing has the words "{query.trim()}".</p>
      {/if}
    </section>
  {/if}

  {@const total = index.works.reduce((sum, work) => sum + work.reports, 0)}
  <section class="metrics" aria-label="The catalogue">
    <a href={link({ page: "works" })}><strong>{index.works.length}</strong><span>works</span></a>
    <a href={link({ page: "reports" })}><strong>{total}</strong><span>reports</span></a>
    <a href={link({ page: "properties" })}
      ><strong>{index.properties.length}</strong><span>properties</span></a
    >
    <a href={link({ page: "entities" })}
      ><strong>{index.entities.length}</strong><span>entities</span></a
    >
  </section>

  <section aria-label="Explore">
    <h2>Explore</h2>
    <div class="explore">
      <a class="card entry" href={link({ page: "reports" })}>
        <strong>Reports</strong>
        <span>Find, filter, sort, and download every value and effect.</span>
      </a>
      <a class="card entry" href={link({ page: "compare" })}>
        <strong>Compare</strong>
        <span>Put the reports of one property in a matrix of your choice.</span>
      </a>
      <a class="card entry" href={link({ page: "effects" })}>
        <strong>Effects</strong>
        <span>See which agents increase or decrease which targets.</span>
      </a>
      <a class="card entry" href={link({ page: "works" })}>
        <strong>Works</strong>
        <span>Read each source, with a mark on each quote.</span>
      </a>
    </div>
  </section>

  <section aria-label="How to read the data">
    <h2>How to read the data</h2>
    <ul class="rules">
      <li>Each report is one statement of one source, with the quote that supports it.</li>
      <li>
        A person compares each report with its source. A report that no person has checked has the
        label "Not checked".
      </li>
      <li>
        A statement of no effect shows as 0. It is a report of no effect, not a measured value of
        zero.
      </li>
      <li>
        Converted values are calculations of arity from the reported value and unit. Each page also
        shows the value as the source reports it.
      </li>
    </ul>
    <p class="muted small">
      Data of {index.generated_at.slice(0, 10)} from {index.software}.
      <a href={`${import.meta.env.BASE_URL}data/${index.database}`}>Download the full database</a>
      (SQLite) · <a href={docsLink()}>Read the documentation</a>
    </p>
  </section>
{/if}

<style>
  .hero {
    padding: var(--space-5) 0 var(--space-4);
    max-width: 52rem;
  }

  .hero h1 {
    font-size: var(--text-3xl);
    letter-spacing: -0.025em;
    margin-bottom: var(--space-3);
  }

  .search {
    position: relative;
    display: block;
    max-width: 44rem;
    margin-top: var(--space-5);
  }

  .search .icon {
    position: absolute;
    left: 0.9rem;
    top: 50%;
    width: 1.15rem;
    height: 1.15rem;
    transform: translateY(-50%);
    fill: none;
    stroke: var(--muted);
    stroke-width: 2;
    stroke-linecap: round;
  }

  .search input {
    max-width: none;
    min-height: 3.1rem;
    padding-left: 2.6rem;
    font-size: var(--text-lg);
    border-radius: 12px;
    box-shadow: var(--shadow-1);
  }

  .hidden {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
  }

  .examples {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    align-items: center;
    margin: var(--space-3) 0 0;
    font-size: var(--text-sm);
  }

  .examples .chip {
    cursor: pointer;
    min-height: 1.6rem;
    font-size: var(--text-xs);
  }

  .results {
    max-width: 52rem;
    margin: var(--space-2) 0 var(--space-5);
  }

  .results h3 {
    font-size: var(--text-xs);
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--muted);
    margin: var(--space-3) 0 var(--space-1);
  }

  .results h3:first-child {
    margin-top: 0;
  }

  .results ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 0.3rem;
  }

  .results li {
    display: flex;
    flex-wrap: wrap;
    gap: 0 0.6rem;
    align-items: baseline;
  }

  .results li .muted {
    font-size: var(--text-sm);
  }

  .all {
    display: inline-block;
    margin-top: var(--space-3);
    font-weight: 600;
  }

  .metrics {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: var(--space-3);
    margin: var(--space-5) 0;
  }

  .metrics a {
    display: grid;
    gap: 0.1rem;
    padding: var(--space-4) var(--space-4);
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: var(--radius);
    box-shadow: var(--shadow-1);
    text-decoration: none;
    color: var(--ink);
    transition: border-color 120ms ease;
  }

  .metrics a:hover {
    border-color: var(--accent);
  }

  .metrics strong {
    font-size: var(--text-2xl);
    font-weight: 650;
    letter-spacing: -0.02em;
    font-variant-numeric: tabular-nums;
  }

  .metrics span {
    color: var(--secondary);
    font-size: var(--text-sm);
  }

  .explore {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(14rem, 1fr));
    gap: var(--space-3);
  }

  .entry {
    display: grid;
    gap: 0.3rem;
    text-decoration: none;
    color: var(--ink);
    transition:
      border-color 120ms ease,
      transform 120ms ease;
  }

  .entry:hover {
    border-color: var(--accent);
  }

  .entry strong {
    font-size: var(--text-lg);
  }

  .entry span {
    color: var(--secondary);
    font-size: var(--text-sm);
  }

  .rules {
    color: var(--secondary);
    max-width: 75ch;
    padding-left: 1.2rem;
    display: grid;
    gap: 0.35rem;
  }

  .small {
    font-size: var(--text-sm);
  }

  @media (max-width: 760px) {
    .metrics {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .hero h1 {
      font-size: var(--text-2xl);
    }
  }
</style>
