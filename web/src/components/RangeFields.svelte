<script lang="ts">
  let {
    legend,
    min,
    max,
    onchange,
  }: {
    legend: string;
    min: number | null;
    max: number | null;
    onchange: (min: number | null, max: number | null) => void;
  } = $props();

  function bound(text: string): number | null {
    if (text.trim() === "") return null;
    const value = Number(text);
    return Number.isFinite(value) ? value : null;
  }
</script>

<fieldset class="range">
  <legend>{legend}</legend>
  <label
    >From <input
      type="number"
      step="any"
      value={min ?? ""}
      onchange={(event) => {
        onchange(bound(event.currentTarget.value), max);
      }}
    /></label
  >
  <label
    >to <input
      type="number"
      step="any"
      value={max ?? ""}
      onchange={(event) => {
        onchange(min, bound(event.currentTarget.value));
      }}
    /></label
  >
</fieldset>

<style>
  .range {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    align-items: center;
    border: 0;
    padding: 0;
    margin: 0.5rem 0;
  }

  legend {
    font-size: 0.85rem;
    color: var(--secondary);
    padding: 0;
    margin-bottom: 0.25rem;
  }

  input {
    width: 8rem;
    background: var(--surface);
    border: 1px solid var(--axis);
    border-radius: 6px;
    padding: 0.25rem 0.4rem;
  }
</style>
