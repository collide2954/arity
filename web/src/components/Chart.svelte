<script lang="ts">
  import embed, { type Result } from "vega-embed";
  import type { TopLevelSpec } from "vega-lite";

  let {
    spec,
    label,
    onselect,
    onrange,
    highlight,
  }: {
    spec: TopLevelSpec;
    label: string;
    onselect?: (report: number) => void;
    /** Gets the range that a person drags along the axis of values. The spec must have the parameter `range`. */
    onrange?: (range: readonly [number, number]) => void;
    /** The report whose mark the figure makes larger. The spec must have the parameter `highlight`. */
    highlight?: number | null;
  } = $props();

  let element: HTMLDivElement | undefined = $state();
  let embedded: Result | null = $state.raw(null);
  let failure: string | null = $state(null);

  function rangeOf(value: unknown): readonly [number, number] | null {
    const fields = Object.values((value ?? {}) as Record<string, unknown>);
    const range = fields[0];
    if (!Array.isArray(range) || range.length !== 2) return null;
    const [low, high] = range as unknown[];
    if (typeof low !== "number" || typeof high !== "number" || low === high) return null;
    const round = (value: number) => Number(value.toPrecision(4));
    return [round(Math.min(low, high)), round(Math.max(low, high))];
  }

  $effect(() => {
    const target = element;
    if (!target) return;
    let result: Result | null = null;
    let cancelled = false;
    let dragged: readonly [number, number] | null = null;
    const release = () => {
      if (dragged) onrange?.(dragged);
      dragged = null;
    };
    embed(target, spec, {
      renderer: "svg",
      actions: { export: true, source: false, compiled: false, editor: false },
    })
      .then((done) => {
        if (cancelled) {
          done.finalize();
          return;
        }
        result = done;
        embedded = done;
        done.view.addEventListener("click", (_event, item) => {
          const datum = (item as { datum?: { report?: unknown } } | null | undefined)?.datum;
          if (typeof datum?.report === "number") onselect?.(datum.report);
        });
        if (onrange) {
          done.view.addSignalListener("range", (_name, value: unknown) => {
            dragged = rangeOf(value);
          });
          window.addEventListener("pointerup", release);
        }
      })
      .catch((error: unknown) => {
        failure = error instanceof Error ? error.message : String(error);
      });
    return () => {
      cancelled = true;
      window.removeEventListener("pointerup", release);
      embedded = null;
      result?.finalize();
    };
  });

  $effect(() => {
    const view = embedded?.view;
    if (!view || highlight === undefined) return;
    try {
      void view.signal("highlight", highlight ?? 0).runAsync();
    } catch {
      return;
    }
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
