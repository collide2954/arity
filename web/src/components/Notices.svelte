<script lang="ts">
  import type { Notice } from "../lib/types";

  let { notices }: { notices: Notice[] } = $props();

  const kinds: Record<string, string> = {
    retraction: "Retraction",
    partial_retraction: "Partial retraction",
    withdrawal: "Withdrawal",
    removal: "Removal",
    expression_of_concern: "Expression of concern",
    correction: "Correction",
    erratum: "Erratum",
    corrigendum: "Corrigendum",
    addendum: "Addendum",
    clarification: "Clarification",
    new_version: "New version",
    new_edition: "New edition",
    reinstatement: "Reinstatement",
    other: "Other notice",
  };

  const sources: Record<string, string> = {
    crossref: "Crossref",
    europe_pmc: "Europe PMC",
    pmc: "PubMed Central",
    retraction_watch: "Retraction Watch",
    publisher: "the publisher",
  };

  const serious = new Set(["retraction", "partial_retraction", "withdrawal", "removal"]);
</script>

{#if notices.length > 0}
  <section class="notices" aria-label="Notices about this work">
    <h2>Notices about this work</h2>
    <p class="secondary">
      Each notice is listed as its source states it. arity does not remove or change data because of
      a notice.
    </p>
    <ul>
      {#each notices as notice, index (index)}
        <li class:serious={serious.has(notice.kind)}>
          <span class="icon" aria-hidden="true"
            >{serious.has(notice.kind) ? "\u{26a0}" : "\u{2139}"}</span
          >
          <strong>{kinds[notice.kind] ?? notice.kind}</strong>
          {#if notice.date}<span> · {notice.date}</span>{/if}
          <span class="secondary">
            · reported by {sources[notice.source] ?? notice.source} as "{notice.reported_type}"</span
          >
          {#if notice.notice_doi}
            · <a href="https://doi.org/{notice.notice_doi}">the notice</a>
          {/if}
          {#if notice.notice_citation}
            <div class="muted citation">{notice.notice_citation}</div>
          {/if}
        </li>
      {/each}
    </ul>
  </section>
{/if}

<style>
  .notices {
    background: var(--notice-bg);
    border-radius: 8px;
    padding: 0.75rem 1rem;
    margin: 1rem 0;
  }

  h2 {
    margin-top: 0;
  }

  ul {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  li {
    padding: 0.3rem 0;
  }

  li.serious strong {
    color: var(--critical-ink);
  }

  .icon {
    margin-right: 0.25rem;
  }

  .citation {
    font-size: 0.85rem;
  }
</style>
