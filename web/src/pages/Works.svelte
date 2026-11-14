<script lang="ts">
  import { download, toCsv } from "../lib/csv";
  import { loadIndex } from "../lib/data";
  import { link } from "../lib/route";
  import type { WorkSummary } from "../lib/types";

  const notices: Record<string, string> = {
    retraction: "Retraction",
    partial_retraction: "Partial retraction",
    expression_of_concern: "Expression of concern",
    correction: "Correction",
    erratum: "Erratum",
    corrigendum: "Corrigendum",
    withdrawal: "Withdrawal",
  };

  function save(works: WorkSummary[]): void {
    download(
      "arity-works.csv",
      toCsv(
        [
          "id",
          "title",
          "citation",
          "year",
          "doi",
          "pmcid",
          "license",
          "preprint",
          "notices",
          "reports",
        ],
        works.map((work) => [
          work.id,
          work.title,
          work.citation,
          work.year,
          work.doi,
          work.pmcid,
          work.license,
          work.is_preprint,
          work.notices.join("; "),
          work.reports,
        ]),
      ),
    );
  }
</script>

<h1>Works</h1>
<p class="lede">The open-access works in the catalogue, in the order that arity found them.</p>

{#await loadIndex()}
  <p class="muted">arity is loading the works.</p>
{:then index}
  <div class="controls">
    <button
      type="button"
      onclick={() => {
        save(index.works);
      }}>Download CSV</button
    >
  </div>
  <div class="table-wrap card">
    <table>
      <thead>
        <tr>
          <th>Work</th>
          <th>Year</th>
          <th>License</th>
          <th>Notices</th>
          <th>Reports</th>
        </tr>
      </thead>
      <tbody>
        {#each index.works as work (work.id)}
          <tr>
            <td>
              <a href={link({ page: "work", id: work.id, report: null })}>{work.title}</a>
              <div class="muted">
                {work.citation}
                {#if work.pmcid}· {work.pmcid}{/if}
                {#if work.is_preprint}· <span class="chip">Preprint</span>{/if}
              </div>
            </td>
            <td>{work.year ?? ""}</td>
            <td>{work.license ?? "not stated"}</td>
            <td>
              <div class="chips">
                {#each work.notices as notice (notice)}
                  <span class="chip notice">{notices[notice] ?? notice}</span>
                {/each}
              </div>
            </td>
            <td>{work.reports}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{:catch error}
  <p>arity cannot load the works: {error instanceof Error ? error.message : String(error)}</p>
{/await}

<style>
  .notice {
    color: var(--critical-ink);
  }
</style>
