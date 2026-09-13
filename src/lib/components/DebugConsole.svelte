<script lang="ts">
  import { onMount, onDestroy, tick } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { debugConsoleOpen } from "$lib/stores/app";
  import { t } from "$lib/stores/i18n";
  import CloseIcon from "~icons/solar/close-circle-bold-duotone";
  import TrashIcon from "~icons/solar/trash-bin-trash-bold-duotone";
  import CopyIcon from "~icons/solar/copy-bold-duotone";
  import CheckIcon from "~icons/solar/check-circle-bold-duotone";

  const MAX_LINES = 1000;

  let lines = $state<string[]>([]);
  let unlisten: (() => void) | null = null;
  let logEl: HTMLDivElement | undefined = $state();
  let autoScroll = $state(true);
  let copied = $state(false);

  function timestamp(): string {
    const d = new Date();
    return d.toTimeString().slice(0, 8);
  }

  onMount(async () => {
    unlisten = await listen<string>("engine-log", (event) => {
      lines.push(`[${timestamp()}] ${event.payload}`);
      if (lines.length > MAX_LINES) {
        lines.splice(0, lines.length - MAX_LINES);
      }
      lines = lines;
      if (autoScroll) {
        tick().then(() => {
          if (logEl) logEl.scrollTop = logEl.scrollHeight;
        });
      }
    });
  });

  onDestroy(() => {
    if (unlisten) unlisten();
  });

  function handleScroll() {
    if (!logEl) return;
    autoScroll = logEl.scrollHeight - logEl.scrollTop - logEl.clientHeight < 24;
  }

  function clearLines() {
    lines = [];
  }

  async function copyLines() {
    try {
      await navigator.clipboard.writeText(lines.join("\n"));
      copied = true;
      setTimeout(() => { copied = false; }, 1500);
    } catch (e) {
      console.error("Failed to copy debug console:", e);
    }
  }
</script>

{#if $debugConsoleOpen}
  <div class="flex flex-col border-t border-border/25 bg-black/90 backdrop-blur-sm h-56 shrink-0">
    <div class="flex items-center justify-between px-3 py-1.5 border-b border-border/15 bg-black/40">
      <span class="text-xs font-mono text-muted-foreground uppercase tracking-wide">
        {$t("debug.title")}
      </span>
      <div class="flex items-center gap-1">
        <button
          onclick={copyLines}
          class="rounded-md p-1 text-muted-foreground hover:text-foreground hover:bg-white/10 transition-colors"
          title={$t("debug.copy")}
          aria-label={$t("debug.copy")}
        >
          {#if copied}
            <CheckIcon class="size-3.5 text-green-400" />
          {:else}
            <CopyIcon class="size-3.5" />
          {/if}
        </button>
        <button
          onclick={clearLines}
          class="rounded-md p-1 text-muted-foreground hover:text-foreground hover:bg-white/10 transition-colors"
          title={$t("debug.clear")}
          aria-label={$t("debug.clear")}
        >
          <TrashIcon class="size-3.5" />
        </button>
        <button
          onclick={() => debugConsoleOpen.set(false)}
          class="rounded-md p-1 text-muted-foreground hover:text-foreground hover:bg-white/10 transition-colors"
          title={$t("debug.close")}
          aria-label={$t("debug.close")}
        >
          <CloseIcon class="size-3.5" />
        </button>
      </div>
    </div>
    <div
      bind:this={logEl}
      onscroll={handleScroll}
      class="flex-1 overflow-y-auto px-3 py-2 font-mono text-[11px] leading-relaxed text-green-400 whitespace-pre-wrap break-all"
    >
      {#if lines.length === 0}
        <span class="text-muted-foreground">{$t("debug.empty")}</span>
      {:else}
        {#each lines as line}
          <div>{line}</div>
        {/each}
      {/if}
    </div>
  </div>
{/if}
