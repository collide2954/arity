<script lang="ts">
  import embed, { type Result } from "vega-embed";
  import type { TopLevelSpec } from "vega-lite";

  let {
    spec,
    label,
    onselect,
  }: { spec: TopLevelSpec; label: string; onselect?: (report: number) => void } = $props();

  let element: HTMLDivElement | undefined = $state();
  let failure: string | null = $state(null);

  $effect(() => {
    const target = element;
    if (!target) return;
    let result: Result | null = null;
    let cancelled = false;
    embed(target, spec, {
      renderer: "svg",
      actions: { export: true, source: false, compiled: false, editor: false },
    })
      .then((embedded) => {
        if (cancelled) {
          embedded.finalize();
          return;
        }
        result = embedded;
        embedded.view.addEventListener("click", (_event, item) => {
          const datum = (item as { datum?: { report?: unknown } } | null | undefined)?.datum;
          if (typeof datum?.report === "number") onselect?.(datum.report);
        });
      })
      .catch((error: unknown) => {
        failure = error instanceof Error ? error.message : String(error);
      });
    return () => {
      cancelled = true;
      result?.finalize();
    };
  });
</script>

<figure aria-label={label}>
  <div class="chart" bind:this={element}></div>
  {#if failure}
    <p class="muted">The figure cannot show: {failure}. The table below has the same data.</p>
  {/if}
</figure>

<style>
  figure {
    margin: 0;
  }

  .chart {
    width: 100%;
  }
</style>
