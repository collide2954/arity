<script lang="ts">
  let copied = $state(false);

  function copy(): void {
    navigator.clipboard
      .writeText(window.location.href)
      .then(() => {
        copied = true;
        setTimeout(() => {
          copied = false;
        }, 2000);
      })
      .catch(() => {
        copied = false;
      });
  }
</script>

<button type="button" onclick={copy}>{copied ? "Copied" : "Copy link"}</button>
