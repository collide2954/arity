<script lang="ts">
  import { link } from "./lib/route";
  import { location } from "./lib/state.svelte";
  import About from "./pages/About.svelte";
  import Effects from "./pages/Effects.svelte";
  import Entities from "./pages/Entities.svelte";
  import Entity from "./pages/Entity.svelte";
  import Home from "./pages/Home.svelte";
  import Properties from "./pages/Properties.svelte";
  import Property from "./pages/Property.svelte";
  import Report from "./pages/Report.svelte";
  import Work from "./pages/Work.svelte";
  import Works from "./pages/Works.svelte";

  const route = $derived(location.route);
  const sections = [
    { page: "works", label: "Works" },
    { page: "properties", label: "Properties" },
    { page: "entities", label: "Entities" },
    { page: "effects", label: "Effects" },
    { page: "about", label: "About" },
  ] as const;

  function active(page: string): boolean {
    return (
      route.page === page ||
      (page === "works" && route.page === "work") ||
      (page === "properties" && route.page === "property") ||
      (page === "entities" && route.page === "entity")
    );
  }
</script>

<header>
  <nav aria-label="Main">
    <a class="brand" href={link({ page: "home" })}>arity</a>
    {#each sections as section (section.page)}
      <a
        href={link({ page: section.page })}
        aria-current={active(section.page) ? "page" : undefined}>{section.label}</a
      >
    {/each}
  </nav>
</header>

<main>
  {#if route.page === "home"}
    <Home />
  {:else if route.page === "works"}
    <Works />
  {:else if route.page === "work"}
    {#key route.id}
      <Work id={route.id} selected={route.report} />
    {/key}
  {:else if route.page === "report"}
    {#key route.id}
      <Report id={route.id} />
    {/key}
  {:else if route.page === "properties"}
    <Properties />
  {:else if route.page === "property"}
    {#key route.key}
      <Property key={route.key} />
    {/key}
  {:else if route.page === "entities"}
    <Entities />
  {:else if route.page === "entity"}
    {#key route.id}
      <Entity id={route.id} />
    {/key}
  {:else if route.page === "effects"}
    <Effects />
  {:else if route.page === "about"}
    <About />
  {:else}
    <h1>The page does not exist</h1>
    <p>arity has no page at <span class="mono">{route.path}</span>.</p>
    <p><a href={link({ page: "home" })}>Go to the start page</a></p>
  {/if}
</main>

<footer>
  <p>
    arity reports what each source states. It does not rate, combine, or correct the data. Code
    under the Apache License 2.0. Data under CC0 1.0. Each source keeps its own license.
  </p>
</footer>

<style>
  header {
    background: var(--surface);
    box-shadow: 0 1px 0 var(--line);
    position: sticky;
    top: 0;
    z-index: 10;
  }

  nav {
    max-width: 1280px;
    margin: 0 auto;
    padding: 0.6rem 1rem;
    display: flex;
    flex-wrap: wrap;
    gap: 1.25rem;
    align-items: baseline;
  }

  nav a {
    color: var(--secondary);
    text-decoration: none;
  }

  nav a[aria-current="page"] {
    color: var(--ink);
    text-decoration: underline;
    text-underline-offset: 0.3em;
  }

  .brand {
    color: var(--ink);
    font-weight: 700;
    font-size: 1.1rem;
    margin-right: 0.5rem;
  }

  footer {
    max-width: 1280px;
    margin: 0 auto;
    padding: 0 1rem 2rem;
    color: var(--muted);
    font-size: 0.85rem;
  }
</style>
