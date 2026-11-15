<script lang="ts">
  import Chart from "./Chart.svelte";
  import { histogramSpec } from "../lib/charts";
  import { formatNumber } from "../lib/format";
  import { theme } from "../lib/state.svelte";
  import { histogram, summary } from "../lib/stats";

  let {
    values,
    unit,
    logarithmic,
  }: {
    /** The values of the selected reports, in the unit of the property, without rejected reports. */
    values: readonly number[];
    unit: string;
    logarithmic: boolean;
  } = $props();

  const found = $derived(summary(values));
  const usable = $derived(logarithmic ? values.filter((value) => value > 0) : values);
  const parts = $derived(
    found
      ? [
          ["Smallest", found.min],
          ["First quartile", found.q1],
          ["Median", found.median],
          ["Third quartile", found.q3],
          ["Largest", found.max],
        ]
      : [],
  );
</script>

{#if found}
  <section class="card statistics" aria-labelledby="statistics-title">
    <h2 id="statistics-title">Statistics of the selected values</h2>
    <dl>
      <div>
        <dt>Values</dt>
        <dd>{found.count}</dd>
      </div>
      {#each parts as [name, value] (name)}
        <div>
          <dt>{name}</dt>
          <dd>{formatNumber(Number(value))} <span class="unit">{unit}</span></dd>
        </div>
      {/each}
    </dl>
    {#if found.count >= 5}
      <Chart
        spec={histogramSpec(histogram(usable, logarithmic), {
          unit,
          logarithmic,
          mode: theme.mode,
        })}
        label="The number of selected values in each range of values"
      />
    {/if}
    <p class="muted">
      arity calculates these numbers from the values of the selected reports, without the reports
      that a person rejected. They describe the selection. They are not a consensus value, and arity
      does not weigh the reports.
    </p>
  </section>
{/if}

<style>
  dl {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(7.5rem, 1fr));
    gap: var(--space-3);
    margin: 0 0 var(--space-3);
  }

  dl div {
    display: grid;
    gap: var(--space-1);
  }

  dt {
    color: var(--secondary);
    font-size: var(--text-sm);
  }

  dd {
    margin: 0;
    font-size: var(--text-lg);
    font-variant-numeric: tabular-nums;
  }

  .unit {
    color: var(--secondary);
    font-size: var(--text-sm);
  }

  p {
    margin: var(--space-3) 0 0;
    font-size: var(--text-sm);
  }
</style>
