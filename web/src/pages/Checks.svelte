<script lang="ts">
  import PageHeader from "../components/PageHeader.svelte";
  import { docsLink } from "../lib/docs";
  import { checkFile, isOrcid } from "../lib/checkfile";
  import {
    checking,
    removeMark,
    removeMarks,
    setCheckMode,
    setPerson,
  } from "../lib/checking.svelte";
  import { download } from "../lib/csv";
  import { loadIndex, loadReports } from "../lib/data";
  import { headline, names } from "../lib/describe";
  import { link } from "../lib/route";
  import type { Report } from "../lib/types";

  const data = Promise.all([loadIndex(), loadReports()]);

  const person = $derived(checking.draft.person);
  const marks = $derived(checking.draft.marks);
  const orcidProblem = $derived(person.orcid.trim() !== "" && !isOrcid(person.orcid));
  const ready = $derived(person.name.trim() !== "" && marks.length > 0);

  function save(): void {
    const day = new Date().toISOString().slice(0, 10);
    download(`arity-checks-${day}.json`, checkFile(checking.draft), "application/json");
  }
</script>

<PageHeader
  title="Your checks"
  summary="Confirm or reject reports against their sources, then give your check file to arity."
>
  <p>
    A person compares each report with its source, and confirms or rejects it. In check mode, the
    page of each report and the list of reports of each work have the buttons Confirm and Reject.
    This browser keeps your marks until you download them as a check file. A person who keeps the
    data of arity then records the file with the command <span class="mono">arity check --file</span
    >, with your name and the time of each check.
  </p>
  <p><a href={docsLink("website", "check-mode")}>How check mode works</a></p>
</PageHeader>

<div class="controls">
  <button
    type="button"
    aria-pressed={checking.on}
    onclick={() => {
      setCheckMode(!checking.on);
    }}>Check mode</button
  >
  <span class="secondary">{checking.on ? "Check mode is on." : "Check mode is off."}</span>
</div>

<fieldset class="person card">
  <legend>You</legend>
  <label
    >Name
    <input
      type="text"
      autocomplete="name"
      value={person.name}
      oninput={(event) => {
        setPerson(event.currentTarget.value, person.orcid);
      }}
    /></label
  >
  <label
    >ORCID iD, if you have one
    <input
      type="text"
      placeholder="0000-0002-1825-0097"
      value={person.orcid}
      aria-invalid={orcidProblem}
      oninput={(event) => {
        setPerson(person.name, event.currentTarget.value);
      }}
    /></label
  >
  {#if orcidProblem}
    <p class="problem">
      An ORCID iD has four groups of four digits, for example 0000-0002-1825-0097. The check file
      does not include an ORCID iD that is not valid.
    </p>
  {/if}
</fieldset>

<h2>Marks ({marks.length})</h2>
{#if marks.length === 0}
  <p class="muted">
    You have no marks. Turn on check mode, then confirm or reject reports on their pages.
  </p>
{:else}
  {#await data}
    <p class="muted">arity is loading the reports.</p>
  {:then [index, reports]}
    {@const lookup = names(index)}
    {@const byId = new Map(reports.map((report): [number, Report] => [report.id, report]))}
    <div class="table-wrap card">
      <table>
        <thead>
          <tr><th>Report</th><th>Your check</th><th>Note</th><th>Time</th><th></th></tr>
        </thead>
        <tbody>
          {#each marks as mark (mark.report)}
            {@const report = byId.get(mark.report)}
            <tr>
              <td>
                <a href={link({ page: "report", id: mark.report })}
                  >{report ? headline(report, lookup) : `${mark.work} ${mark.local}`}</a
                >
                <div class="muted mono">{mark.work} · run {mark.run} · {mark.local}</div>
              </td>
              <td class={mark.state}>{mark.state === "confirmed" ? "Confirmed" : "Rejected"}</td>
              <td>{mark.note}</td>
              <td>{mark.at.slice(0, 16).replace("T", " ")}</td>
              <td>
                <button
                  type="button"
                  onclick={() => {
                    removeMark(mark.report);
                  }}>Remove</button
                >
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/await}
  <div class="controls">
    <button type="button" disabled={!ready} onclick={save}>Download the check file</button>
    <button
      type="button"
      onclick={() => {
        removeMarks();
      }}>Remove all marks</button
    >
    {#if person.name.trim() === ""}
      <span class="problem">Give your name before you download the check file.</span>
    {/if}
  </div>
{/if}

<style>
  .person {
    display: grid;
    gap: 0.6rem;
    max-width: 32rem;
    border: 0;
  }

  .person legend {
    font-weight: 600;
    padding: 0;
  }

  .person label {
    display: grid;
    gap: 0.2rem;
  }

  .person input {
    background: var(--surface);
    border: 1px solid var(--axis);
    border-radius: 6px;
    padding: 0.35rem 0.5rem;
  }

  .problem {
    color: var(--critical-ink);
    margin: 0;
  }

  .confirmed {
    color: var(--good-ink);
  }

  .rejected {
    color: var(--critical-ink);
  }
</style>
