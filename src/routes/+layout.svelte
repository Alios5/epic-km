<script lang="ts">
  import "../app.css";
  import TitleBar from "$lib/components/TitleBar.svelte";
  let { children } = $props();
</script>

<div class="flex flex-col h-screen bg-background text-foreground overflow-hidden">
  <TitleBar />
  <div class="flex-1 min-h-0 overflow-hidden">
    {@render children?.()}
  </div>
</div>
