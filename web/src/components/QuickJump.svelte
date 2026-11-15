<script lang="ts">
  import { loadIndex, loadReports } from "../lib/data";
  import { catalogueTargets, findTargets, reportTargets, type Target } from "../lib/jump";
  import { link } from "../lib/route";

  /** The largest number of targets that the box shows. */
  const LIMIT = 12;

  let dialog: HTMLDialogElement | undefined = $state();
  let field: HTMLInputElement | undefined = $state();
  let search = $state("");
  let active = $state(0);
  let targets = $state.raw<readonly Target[]>([]);
  let asked = false;

  const found = $derived(findTargets(targets, search, LIMIT));

  function load(): void {
    if (asked) return;
    asked = true;
    loadIndex()
      .then(async (index) => {
        const catalogue = catalogueTargets(index);
        targets = catalogue;
        const reports = await loadReports();
        targets = [...catalogue, ...reportTargets(index, reports)];
      })
      .catch(() => {
        asked = false;
      });
  }

  function open(): void {
    load();
    search = "";
    active = 0;
    dialog?.showModal();
    field?.focus();
  }

  function go(target: Target | undefined): void {
    if (!target) return;
    dialog?.close();
    window.location.hash = link(target.route);
  }

  function editing(target: EventTarget | null): boolean {
    return (
      target instanceof HTMLElement &&
      (target.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName))
    );
  }

  function onwindowkey(event: KeyboardEvent): void {
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
      event.preventDefault();
      open();
    } else if (event.key === "/" && !editing(event.target) && !dialog?.open) {
      const box = document.querySelector<HTMLInputElement>("main input[type='search']");
      if (box) {
        event.preventDefault();
        box.focus();
      }
    }
  }

  function onfieldkey(event: KeyboardEvent): void {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      active = Math.min(active + 1, found.length - 1);
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      active = Math.max(active - 1, 0);
    } else if (event.key === "Enter") {
      event.preventDefault();
      go(found[active]);
    }
  }
</script>

<svelte:window onkeydown={onwindowkey} />

<button
  type="button"
  class="opener"
  title="Find and open a work, a property, an entity, a report, or a page"
  onclick={open}>Jump <kbd>Ctrl K</kbd></button
>

<dialog bind:this={dialog} aria-label="Jump to a page">
  <input
    bind:this={field}
    type="text"
    role="combobox"
    aria-label="Find a work, a property, an entity, a report, or a page"
    aria-expanded={found.length > 0}
    aria-controls="jump-results"
    aria-activedescendant={found.length > 0 ? `jump-${String(active)}` : undefined}
    placeholder="Find a work, a property, an entity, a report, or a page"
    bind:value={search}
    oninput={() => {
      active = 0;
    }}
    onkeydown={onfieldkey}
  />
  <ul id="jump-results" role="listbox" aria-label="Results">
    {#each found as target, position (`${target.kind}-${link(target.route)}`)}
      <li id="jump-{position}" role="option" aria-selected={position === active}>
        <a
          href={link(target.route)}
          onclick={() => {
            dialog?.close();
          }}
          ><span class="label">{target.label}</span>
          <span class="muted">{target.detail}</span></a
        >
      </li>
    {/each}
  </ul>
  {#if search.trim() !== "" && found.length === 0}
    <p class="muted">Nothing has these words.</p>
  {/if}
  <p class="keys muted">
    ↑ and ↓ select · Enter opens · Esc closes · Ctrl+K opens this box · / goes to the search box of
    a page
  </p>
</dialog>

<style>
  .opener {
    padding: 0.15rem 0.6rem;
    font-size: 0.9rem;
  }

  kbd {
    font-family: var(--mono);
    font-size: 0.75rem;
    color: var(--muted);
  }

  dialog {
    width: min(40rem, calc(100vw - 2rem));
    margin-top: 10vh;
    padding: 0.75rem;
    border: 0;
    border-radius: 10px;
    background: var(--surface);
    color: var(--ink);
    box-shadow:
      0 0 0 1px var(--ring),
      0 12px 32px rgb(0 0 0 / 0.25);
  }

  dialog::backdrop {
    background: rgb(0 0 0 / 0.35);
  }

  input {
    width: 100%;
    background: var(--page);
    border: 1px solid var(--axis);
    border-radius: 6px;
    padding: 0.5rem 0.6rem;
  }

  ul {
    list-style: none;
    margin: 0.5rem 0 0;
    padding: 0;
    max-height: 60vh;
    overflow: auto;
  }

  li a {
    display: grid;
    padding: 0.35rem 0.5rem;
    border-radius: 6px;
    color: var(--ink);
    text-decoration: none;
  }

  li[aria-selected="true"] a,
  li a:hover {
    background: var(--mark);
  }

  .label {
    overflow-wrap: anywhere;
  }

  .keys {
    font-size: 0.8rem;
    margin: 0.5rem 0 0;
  }
</style>
