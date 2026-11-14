<script lang="ts">
  import { loadIndex } from "../lib/data";
  import { link } from "../lib/route";
  import type { Index } from "../lib/types";

  let query = $state("");

  function search(index: Index, text: string) {
    const needle = text.trim().toLowerCase();
    if (needle.length < 2) return null;
    const has = (value: string | null | undefined) => (value ?? "").toLowerCase().includes(needle);
    return {
      works: index.works.filter((work) => has(work.title) || has(work.pmcid) || has(work.doi)),
      properties: index.properties.filter((property) => has(property.label) || has(property.key)),
      entities: index.entities.filter(
        (entity) => has(entity.label) || entity.names.some(has) || entity.xrefs.some(has),
      ),
    };
  }
</script>

<h1>Values that open-access research reports, with their sources</h1>
<p class="lede">
  arity collects the values and effects that open-access research articles report. Each value keeps
  its source: the article, the place in the article, and the exact text. arity does not rate,
  combine, or correct the data. It shows what each source states, and each notice about the source,
  for example a retraction.
</p>

{#await loadIndex()}
  <p class="muted">arity is loading the catalogue.</p>
{:then index}
  {@const reports = index.works.reduce((total, work) => total + work.reports, 0)}
  <div class="stats">
    <a class="card stat" href={link({ page: "works" })}
      ><strong>{index.works.length}</strong><span>works</span></a
    >
    <span class="card stat"><strong>{reports}</strong><span>reports</span></span>
    <a class="card stat" href={link({ page: "properties" })}
      ><strong>{index.properties.length}</strong><span>properties</span></a
    >
    <a class="card stat" href={link({ page: "entities" })}
      ><strong>{index.entities.length}</strong><span>entities</span></a
    >
  </div>

  <label for="search"><h2>Find a work, a property, or an entity</h2></label>
  <input
    id="search"
    type="search"
    placeholder="For example: doxorubicin, IC50, MCF-7"
    bind:value={query}
  />
  {@const found = search(index, query)}
  {#if found}
    <div class="results">
      {#each found.properties as property (property.key)}
        <a href={link({ page: "property", key: property.key })}
          >{property.label} <span class="muted">property · {property.reports} reports</span></a
        >
      {/each}
      {#each found.entities as entity (entity.id)}
        <a href={link({ page: "entity", id: entity.id })}
          >{entity.label} <span class="muted">{entity.kind} · {entity.reports} reports</span></a
        >
      {/each}
      {#each found.works as work (work.id)}
        <a href={link({ page: "work", id: work.id, report: null })}
          >{work.title} <span class="muted">{work.citation}</span></a
        >
      {/each}
      {#if found.works.length + found.properties.length + found.entities.length === 0}
        <p class="muted">Nothing matches "{query}".</p>
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
{:catch error}
  <p>arity cannot load the catalogue: {error instanceof Error ? error.message : String(error)}</p>
{/await}

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
</style>
