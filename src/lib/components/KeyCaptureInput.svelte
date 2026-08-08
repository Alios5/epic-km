<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { layoutMap, labelForCode } from "$lib/keyLabels";
  import { t, locale } from "$lib/stores/i18n";

  interface Props {
    value: string;
    onchange: (key: string) => void;
  }

  let { value, onchange }: Props = $props();

  let listening = $state(false);

  function startListening() {
    // Suspend the global capture hotkey while listening: pressing the current
    // toggle key must not toggle capture in the middle of the assignment.
    invoke("suspend_hotkey").catch(() => {});
    listening = true;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!listening) return;
    e.preventDefault();
    e.stopPropagation();
    listening = false;
    if (e.code !== "Escape") {
      onchange(e.code);
    }
    // The parent's onchange pushes the new profile to the engine (which
    // re-registers the hotkey); resume covers cancel/Escape paths.
    invoke("resume_hotkey").catch(() => {});
  }

  function handleBlur() {
    if (listening) {
      invoke("resume_hotkey").catch(() => {});
    }
    listening = false;
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<button
  type="button"
  onclick={startListening}
  onblur={handleBlur}
  class="inline-flex h-8 items-center justify-center rounded-lg border border-input bg-transparent px-3 text-sm transition-colors hover:bg-muted focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 outline-none select-none w-32 text-center
    {listening ? 'border-ring ring-3 ring-ring/50 bg-muted' : ''}"
>
  {#if listening}
    <span class="text-muted-foreground animate-pulse">{$t("keycap.press")}</span>
  {:else}
    {labelForCode(value, $layoutMap, $locale)}
  {/if}
</button>
