<script lang="ts">
  import { isSerious, noticeLabel, noticeSign, seriousFirst } from "../lib/notices";

  let { kinds }: { kinds: readonly string[] } = $props();
</script>

{#if kinds.length > 0}
  <span class="chips">
    {#each seriousFirst(kinds) as kind (kind)}
      <span class="chip" class:serious={isSerious(kind)}
        ><span aria-hidden="true">{noticeSign(kind)}</span>{noticeLabel(kind)}</span
      >
    {/each}
  </span>
{/if}

<style>
  .serious {
    color: var(--critical-ink);
  }
</style>
