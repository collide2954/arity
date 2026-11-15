<script lang="ts">
  import { markOf, type Verdict } from "../lib/checkfile";
  import { addMark, checking, removeMark } from "../lib/checking.svelte";
  import { link } from "../lib/route";
  import type { Report, WorkSummary } from "../lib/types";

  let { report, work }: { report: Report; work: WorkSummary } = $props();

  let note = $state("");

  const mark = $derived(checking.draft.marks.find((candidate) => candidate.report === report.id));

  function choose(state: Verdict): void {
    addMark(markOf(report, work, state, note, new Date().toISOString()));
    note = "";
  }
</script>

<div class="check" role="group" aria-label="Your check of this report">
  {#if mark}
    <span class="result {mark.state}"
      >{mark.state === "confirmed"
        ? "You confirmed this report."
        : "You rejected this report."}</span
    >
    {#if mark.note}<q>{mark.note}</q>{/if}
    <button
      type="button"
      onclick={() => {
        removeMark(report.id);
      }}>Remove your mark</button
    >
    <a href={link({ page: "checks" })}>Your checks</a>
  {:else}
    <input
      type="text"
      aria-label="Note for your check"
      placeholder="Note, for example the place where the source states the value"
      bind:value={note}
    />
    <button
      type="button"
      class="confirm"
      onclick={() => {
        choose("confirmed");
      }}>Confirm</button
    >
    <button
      type="button"
      class="reject"
      onclick={() => {
        choose("rejected");
      }}>Reject</button
    >
  {/if}
</div>

<style>
  .check {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    align-items: center;
    padding: 0.4rem 0.5rem;
    border-radius: 6px;
    background: var(--notice-bg);
  }

  input {
    flex: 1 1 14rem;
    min-width: 0;
    background: var(--surface);
    border: 1px solid var(--axis);
    border-radius: 6px;
    padding: 0.25rem 0.4rem;
  }

  .confirm {
    color: var(--good-ink);
  }

  .reject {
    color: var(--critical-ink);
  }

  .result.confirmed {
    color: var(--good-ink);
  }

  .result.rejected {
    color: var(--critical-ink);
  }
</style>
