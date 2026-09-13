<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import TitleBar from "$lib/components/TitleBar.svelte";
  import ResizeHandles from "$lib/components/ResizeHandles.svelte";
  import ConfirmDialog from "$lib/components/ConfirmDialog.svelte";
  import FileDialog from "$lib/components/FileDialog.svelte";
  let { children } = $props();

  let version = $state("");
  onMount(async () => {
    version = await getVersion();
  });
</script>

<div class="flex flex-col h-screen bg-background text-foreground overflow-hidden">
  <ResizeHandles />
  <TitleBar />
  <div class="flex-1 min-h-0 overflow-hidden">
    {@render children?.()}
  </div>
  {#if version}
    <span
      class="fixed bottom-1 left-2 z-50 text-[10px] leading-none text-muted-foreground/50 select-none pointer-events-none"
      >v{version}</span
    >
  {/if}
  <ConfirmDialog />
  <FileDialog />
</div>
