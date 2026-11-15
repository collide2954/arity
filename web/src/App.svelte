<script lang="ts">
  import QuickJump from "./components/QuickJump.svelte";
  import { checking, setCheckMode } from "./lib/checking.svelte";
  import { docsLink } from "./lib/docs";
  import { link } from "./lib/route";
  import { location } from "./lib/state.svelte";
  import About from "./pages/About.svelte";
  import Checks from "./pages/Checks.svelte";
  import Compare from "./pages/Compare.svelte";
  import Effects from "./pages/Effects.svelte";
  import Entities from "./pages/Entities.svelte";
  import Entity from "./pages/Entity.svelte";
  import Home from "./pages/Home.svelte";
  import Properties from "./pages/Properties.svelte";
  import Property from "./pages/Property.svelte";
  import Report from "./pages/Report.svelte";
  import Reports from "./pages/Reports.svelte";
  import Work from "./pages/Work.svelte";
  import Works from "./pages/Works.svelte";

  const route = $derived(location.route);
  const sections = [
    { page: "works", label: "Works" },
    { page: "reports", label: "Reports" },
    { page: "properties", label: "Properties" },
    { page: "entities", label: "Entities" },
    { page: "effects", label: "Effects" },
    { page: "compare", label: "Compare" },
  ] as const;

  function active(page: string): boolean {
    return (
      route.page === page ||
      (page === "works" && route.page === "work") ||
      (page === "reports" && route.page === "report") ||
      (page === "properties" && route.page === "property") ||
      (page === "entities" && route.page === "entity")
    );
  }
</script>

<header class="site">
  <div class="bar">
    <a class="brand" href={link({ page: "home" })} aria-label="arity, start page">
      <svg class="logo" viewBox="0 0 24 24" aria-hidden="true">
        <rect width="24" height="24" rx="6" />
        <circle cx="7" cy="15" r="2.2" />
        <circle cx="12" cy="9" r="2.2" />
        <circle cx="17" cy="12.5" r="2.2" />
      </svg>
      <span>arity</span>
    </a>
    <nav aria-label="Main">
      {#each sections as section (section.page)}
        <a
          href={link({ page: section.page })}
          aria-current={active(section.page) ? "page" : undefined}>{section.label}</a
        >
      {/each}
    </nav>
    <div class="tools">
      <QuickJump />
      {#if checking.on || checking.draft.marks.length > 0}
        <a
          class="checks"
          href={link({ page: "checks" })}
          aria-current={route.page === "checks" ? "page" : undefined}
          >Your checks <span class="count">{checking.draft.marks.length}</span></a
        >
      {/if}
      <button
        type="button"
        class="switch"
        role="switch"
        aria-checked={checking.on}
        title="In check mode, you can confirm or reject reports."
        onclick={() => {
          setCheckMode(!checking.on);
        }}><span class="track" aria-hidden="true"></span>Check mode</button
      >
      <a class="docs" href={docsLink()}>Docs</a>
    </div>
  </div>
</header>
<main>
  {#if route.page === "home"}
    <Home />
  {:else if route.page === "works"}
    <Works params={route.params ?? ""} />
  {:else if route.page === "work"}
    {#key route.id}
      <Work id={route.id} selected={route.report} />
    {/key}
  {:else if route.page === "reports"}
    <Reports params={route.params ?? ""} />
  {:else if route.page === "report"}
    {#key route.id}
      <Report id={route.id} />
    {/key}
  {:else if route.page === "properties"}
    <Properties params={route.params ?? ""} />
  {:else if route.page === "property"}
    {#key route.key}
      <Property key={route.key} params={route.params ?? ""} />
    {/key}
  {:else if route.page === "entities"}
    <Entities params={route.params ?? ""} />
  {:else if route.page === "entity"}
    {#key route.id}
      <Entity id={route.id} />
    {/key}
  {:else if route.page === "effects"}
    <Effects params={route.params ?? ""} />
  {:else if route.page === "compare"}
    <Compare params={route.params ?? ""} />
  {:else if route.page === "checks"}
    <Checks />
  {:else if route.page === "about"}
    <About />
  {:else}
    <h1>The page does not exist</h1>
    <p>arity has no page at <span class="mono">{route.path}</span>.</p>
    <p><a href={link({ page: "home" })}>Go to the start page</a></p>
  {/if}
</main>

<footer class="site">
  <div class="bar">
    <p>
      <strong>arity</strong> reports what each source states. It does not rate, combine, or correct the
      data.
    </p>
    <nav aria-label="More">
      <a href={docsLink()}>Documentation</a>
      <a href={link({ page: "about" })}>About</a>
      <a href="https://github.com/collide2954/arity">Source code</a>
      <a href="https://github.com/collide2954/arity-data">Data</a>
    </nav>
    <p class="muted">
      Code under the Apache License 2.0. Data under CC0 1.0. Each source keeps its own license.
    </p>
  </div>
</footer>

<style>
  header.site {
    position: sticky;
    top: 0;
    z-index: 10;
    background: color-mix(in srgb, var(--surface) 88%, transparent);
    backdrop-filter: saturate(1.4) blur(10px);
    border-bottom: 1px solid var(--line);
  }

  .bar {
    max-width: var(--content);
    margin: 0 auto;
    padding: 0 var(--space-5);
  }

  header .bar {
    display: flex;
    align-items: center;
    gap: var(--space-5);
    min-height: var(--header);
  }

  .brand {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    color: var(--ink);
    font-weight: 750;
    font-size: 1.15rem;
    letter-spacing: -0.03em;
    text-decoration: none;
  }

  .logo {
    width: 1.5rem;
    height: 1.5rem;
  }

  .logo rect {
    fill: var(--accent);
  }

  .logo circle {
    fill: var(--on-accent);
  }

  nav[aria-label="Main"] {
    display: flex;
    align-self: stretch;
    gap: var(--space-1);
  }

  nav[aria-label="Main"] a {
    display: inline-flex;
    align-items: center;
    padding: 0 0.7rem;
    font-size: var(--text-sm);
    font-weight: 550;
    color: var(--secondary);
    text-decoration: none;
    border-bottom: 2px solid transparent;
    margin-bottom: -1px;
  }

  nav[aria-label="Main"] a:hover {
    color: var(--ink);
  }

  nav[aria-label="Main"] a[aria-current="page"] {
    color: var(--ink);
    border-bottom-color: var(--accent);
  }

  .tools {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .checks,
  .docs {
    font-size: var(--text-sm);
    font-weight: 550;
    color: var(--secondary);
    text-decoration: none;
    white-space: nowrap;
  }

  .checks:hover,
  .docs:hover,
  .checks[aria-current="page"] {
    color: var(--ink);
  }

  .count {
    display: inline-block;
    min-width: 1.25rem;
    padding: 0 0.35rem;
    margin-left: 0.2rem;
    border-radius: 999px;
    background: var(--accent);
    color: var(--on-accent);
    font-size: var(--text-xs);
    text-align: center;
  }

  .switch {
    gap: 0.5rem;
    border: 0;
    background: transparent;
    padding: 0 0.25rem;
    color: var(--secondary);
    white-space: nowrap;
  }

  .switch:hover:not(:disabled) {
    background: transparent;
    color: var(--ink);
  }

  .track {
    position: relative;
    width: 1.9rem;
    height: 1.1rem;
    border-radius: 999px;
    background: var(--surface-3);
    border: 1px solid var(--line-strong);
    transition: background 150ms ease;
  }

  .track::after {
    content: "";
    position: absolute;
    top: 1px;
    left: 1px;
    width: 0.8rem;
    height: 0.8rem;
    border-radius: 50%;
    background: var(--surface);
    box-shadow: var(--shadow-1);
    transition: transform 150ms ease;
  }

  .switch[aria-checked="true"] .track {
    background: var(--accent);
    border-color: var(--accent);
  }

  .switch[aria-checked="true"] .track::after {
    transform: translateX(0.8rem);
  }

  footer.site {
    border-top: 1px solid var(--line);
    background: var(--surface);
    margin-top: var(--space-6);
  }

  footer .bar {
    display: grid;
    gap: var(--space-2);
    padding-top: var(--space-5);
    padding-bottom: var(--space-6);
    font-size: var(--text-sm);
    color: var(--secondary);
  }

  footer p {
    margin: 0;
  }

  footer nav {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-4);
  }

  footer nav a {
    color: var(--secondary);
  }

  @media (max-width: 1100px) {
    header .bar {
      flex-wrap: wrap;
      gap: var(--space-2) var(--space-4);
      padding-top: var(--space-2);
    }

    nav[aria-label="Main"] {
      order: 3;
      width: 100%;
      overflow-x: auto;
      min-height: 2.5rem;
    }

    .tools {
      margin-left: auto;
    }
  }

  @media (max-width: 640px) {
    .bar {
      padding: 0 16px;
    }

    .switch {
      font-size: 0;
    }
  }
</style>
