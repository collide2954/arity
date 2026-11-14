<script lang="ts">
  import type { CheckState } from "../lib/types";

  let { state }: { state: CheckState } = $props();

  const labels: Record<CheckState, { icon: string; text: string; title: string }> = {
    confirmed: {
      icon: "\u{2713}",
      text: "Confirmed",
      title: "A person compared this report with its source and confirmed it.",
    },
    unchecked: {
      icon: "\u{25cb}",
      text: "Not checked",
      title: "No person has compared this report with its source yet.",
    },
    rejected: {
      icon: "\u{2717}",
      text: "Rejected",
      title: "A person compared this report with its source and found that it does not agree.",
    },
  };

  const label = $derived(labels[state]);
</script>

<span class="badge {state}" title={label.title}
  ><span aria-hidden="true">{label.icon}</span> {label.text}</span
>

<style>
  .badge {
    display: inline-flex;
    gap: 0.25rem;
    align-items: center;
    font-size: 0.8rem;
    border-radius: 999px;
    padding: 0 0.5rem;
    box-shadow: 0 0 0 1px var(--ring);
    white-space: nowrap;
  }

  .confirmed {
    color: var(--good-ink);
  }

  .unchecked {
    color: var(--warning-ink);
  }

  .rejected {
    color: var(--critical-ink);
  }
</style>
