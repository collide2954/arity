<script lang="ts">
  import Notices from "../components/Notices.svelte";
  import PdfViewer from "../components/PdfViewer.svelte";
  import ReportItem from "../components/ReportItem.svelte";
  import TextView from "../components/TextView.svelte";
  import { coverageLine } from "../lib/coverage";
  import { dataUrl, loadIndex, loadReports, loadWork } from "../lib/data";
  import { names } from "../lib/describe";
  import { highlightsOf } from "../lib/highlight";
  import { link } from "../lib/route";

  let { id, selected: initial }: { id: number; selected: number | null } = $props();

  // svelte-ignore state_referenced_locally
  let selected: number | null = $state(initial);
  let view: "pdf" | "text" | null = $state(null);
  let located: ReadonlySet<number> | null = $state(null);

  // svelte-ignore state_referenced_locally
  const data = Promise.all([loadIndex(), loadReports(), loadWork(id)]);

  function select(report: number): void {
    selected = report;
    history.replaceState(null, "", link({ page: "work", id, report }));
    document.getElementById(`report-${String(report)}`)?.scrollIntoView({ block: "nearest" });
  }
</script>

{#await data}
  <p class="muted">arity is loading the work.</p>
{:then [index, reports, page]}
  {@const lookup = names(index)}
  {@const own = reports.filter((report) => report.work === id)}
  {@const highlights = highlightsOf(own, lookup)}
  {@const metadata = page.work.metadata}
  {@const shown = view ?? (page.pdf ? "pdf" : "text")}
  <header>
    <h1>{metadata.title}</h1>
    <p class="secondary">
      {page.attribution.citation}
      {#if metadata.is_preprint}<span class="chip">Preprint</span>{/if}
    </p>
    <p class="chips">
      {#if page.attribution.doi_url}<a class="chip" href={page.attribution.doi_url}
          >DOI {metadata.doi}</a
        >{/if}
      {#if metadata.pmcid}<a
          class="chip"
          href="https://pmc.ncbi.nlm.nih.gov/articles/{metadata.pmcid}/">{metadata.pmcid}</a
        >{/if}
      {#if metadata.pmid}<a class="chip" href="https://pubmed.ncbi.nlm.nih.gov/{metadata.pmid}/"
          >PubMed {metadata.pmid}</a
        >{/if}
      {#if page.attribution.license}
        <a class="chip" href={page.attribution.license_url ?? undefined}
          >{page.attribution.license}</a
        >
      {:else}
        <span class="chip">License not stated</span>
      {/if}
    </p>
    {#if page.coverage}
      <p
        class="muted coverage"
        title="arity counts the numbers in table cells, and the numbers in the text that have a unit or a sign. In the text view, a dotted line marks each number that no report quotes yet."
      >
        {coverageLine(page.coverage)}
      </p>
    {/if}
  </header>

  <Notices notices={page.work.notices} />

  <div class="layout">
    <section class="source" aria-label="Source">
      <div class="controls" role="group" aria-label="View">
        {#if page.pdf}
          <button type="button" aria-pressed={shown === "pdf"} onclick={() => (view = "pdf")}
            >PDF</button
          >
        {/if}
        {#if page.document}
          <button type="button" aria-pressed={shown === "text"} onclick={() => (view = "text")}
            >Text</button
          >
        {/if}
        <span class="muted"
          >Each marked part is the quote of a report. Select a mark to see the report.</span
        >
      </div>
      {#if shown === "pdf" && page.pdf}
        <PdfViewer
          url={dataUrl(page.pdf)}
          blocks={page.document?.blocks ?? []}
          {highlights}
          {selected}
          onselect={select}
          onlocate={(found: ReadonlySet<number>) => {
            located = found;
          }}
        />
      {:else if page.document}
        <TextView
          document={page.document}
          coverage={page.coverage}
          {highlights}
          {selected}
          onselect={select}
        />
      {:else}
        <p class="card">
          The license of this work does not let arity share its full text. The quote of each report
          is on the page of the report.
        </p>
      {/if}
    </section>

    <aside class="reports" aria-label="Reports">
      <h2>Reports from this work ({own.length})</h2>
      {#if own.length === 0}
        <p class="muted">No reader has submitted reports from this work yet.</p>
      {/if}
      <div class="list">
        {#each own as report (report.id)}
          <ReportItem
            {report}
            {lookup}
            selected={selected === report.id}
            located={shown !== "pdf" || located === null || located.has(report.id)}
            onselect={select}
          />
        {/each}
      </div>
    </aside>
  </div>

  <section>
    <h2>Provenance of the source files</h2>
    <p class="secondary">
      arity downloaded these files and recorded the SHA-256 digest of each one. The canonical text
      has the digest <span class="mono">{page.work.text_sha256 ?? "none"}</span>.
    </p>
    <div class="table-wrap card">
      <table>
        <thead>
          <tr><th>File</th><th>Address</th><th>Retrieved</th><th>SHA-256</th><th>Bytes</th></tr>
        </thead>
        <tbody>
          {#each page.work.retrievals as retrieval (retrieval.sha256 + retrieval.role)}
            <tr>
              <td
                >{retrieval.role.replace("_", " ")}{retrieval.supplement_key
                  ? ` (${retrieval.supplement_key})`
                  : ""}</td
              >
              <td class="mono"><a href={retrieval.url}>{retrieval.url}</a></td>
              <td>{retrieval.retrieved_at}</td>
              <td class="mono">{retrieval.sha256}</td>
              <td>{retrieval.bytes}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
    {#if page.attribution.statement || page.attribution.copyright}
      <p class="muted">{page.attribution.copyright} {page.attribution.statement}</p>
    {/if}
  </section>
{:catch error}
  <p>arity cannot load the work: {error instanceof Error ? error.message : String(error)}</p>
{/await}

<style>
  header h1 {
    max-width: 60ch;
  }

  .coverage {
    margin: 0.25rem 0 0;
    font-size: 0.9rem;
  }

  .layout {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(18rem, 26rem);
    gap: 1rem;
    align-items: start;
  }

  .reports {
    position: sticky;
    top: 4rem;
    max-height: calc(100vh - 5rem);
    overflow: auto;
  }

  .reports h2 {
    margin-top: 0;
  }

  .list {
    display: grid;
    gap: 0.5rem;
  }

  @media (max-width: 960px) {
    .layout {
      grid-template-columns: minmax(0, 1fr);
    }

    .reports {
      position: static;
      max-height: none;
    }
  }
</style>
