<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { fileDialogState, answerFileDialog } from "$lib/stores/dialogs";
  import { t } from "$lib/stores/i18n";
  import FolderIcon from "~icons/solar/folder-bold-duotone";
  import FileIcon from "~icons/solar/file-text-bold-duotone";
  import UpIcon from "~icons/solar/alt-arrow-up-bold-duotone";
  import DriveIcon from "~icons/solar/monitor-bold-duotone";

  interface DirEntry {
    name: string;
    path: string;
    is_dir: boolean;
  }

  // currentDir === "" means we're showing filesystem roots (drive letters)
  let currentDir = $state("");
  let entries = $state<DirEntry[]>([]);
  let selected = $state<DirEntry | null>(null);
  let filename = $state("");
  let loading = $state(false);
  let error = $state("");

  let opts = $derived($fileDialogState);
  let isSave = $derived(opts.mode === "save");

  let visibleEntries = $derived(
    opts.extension
      ? entries.filter((e) => e.is_dir || e.name.toLowerCase().endsWith(`.${opts.extension}`))
      : entries,
  );

  let canConfirm = $derived(
    isSave ? filename.trim().length > 0 : selected !== null,
  );

  $effect(() => {
    if (opts.open) {
      filename = opts.defaultFilename ?? "";
      error = "";
      init();
    }
  });

  async function init() {
    try {
      const dir = await invoke<string>("default_dir");
      await navigate(dir || undefined);
    } catch {
      await showRoots();
    }
  }

  async function navigate(dir?: string) {
    if (!dir) return showRoots();
    loading = true;
    error = "";
    try {
      entries = await invoke<DirEntry[]>("list_dir", { path: dir });
      currentDir = dir;
      selected = null;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function showRoots() {
    entries = await invoke<DirEntry[]>("list_roots");
    currentDir = "";
    selected = null;
  }

  async function goUp() {
    if (!currentDir) return;
    const parent = await invoke<string | null>("parent_dir", { path: currentDir });
    if (parent) await navigate(parent);
    else await showRoots();
  }

  function handleEntryClick(e: DirEntry) {
    if (e.is_dir) {
      navigate(e.path);
    } else if (isSave) {
      filename = e.name;
      selected = e;
    } else {
      selected = e;
    }
  }

  function joinPath(dir: string, name: string): string {
    if (dir.endsWith("\\") || dir.endsWith("/")) return dir + name;
    return dir + (dir.includes("\\") ? "\\" : "/") + name;
  }

  function confirm() {
    if (isSave) {
      let name = filename.trim();
      if (!name) return;
      if (opts.extension && !name.toLowerCase().endsWith(`.${opts.extension}`)) {
        name += `.${opts.extension}`;
      }
      answerFileDialog(joinPath(currentDir, name));
    } else if (selected) {
      answerFileDialog(selected.path);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!opts.open) return;
    if (e.code === "Escape") {
      e.preventDefault();
      answerFileDialog(null);
    } else if (e.code === "Enter" && canConfirm) {
      e.preventDefault();
      confirm();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if opts.open}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    role="presentation"
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
    onmousedown={(e) => e.target === e.currentTarget && answerFileDialog(null)}
  >
    <div role="dialog" aria-modal="true" aria-label={opts.title} class="elevated-panel rounded-xl p-5 w-full max-w-md mx-4 flex flex-col gap-3">
      <h2 class="text-lg font-semibold">{opts.title}</h2>

      <!-- Location bar -->
      <div class="flex items-center gap-2">
        <button
          onclick={goUp}
          disabled={!currentDir}
          class="flex size-7 shrink-0 items-center justify-center rounded-md border border-border text-muted-foreground transition-colors hover:bg-accent disabled:opacity-40"
          aria-label={$t("file.up")}
          title={$t("file.up")}
        >
          <UpIcon class="size-3.5" />
        </button>
        <div class="min-w-0 flex-1 truncate rounded-md border border-border bg-muted/40 px-2.5 py-1.5 font-mono text-xs text-muted-foreground">
          {currentDir || $t("file.drives")}
        </div>
      </div>

      <!-- Entries -->
      <div class="h-56 overflow-y-auto rounded-lg border border-border/60 bg-background/40">
        {#if loading}
          <div class="flex h-full items-center justify-center text-xs text-muted-foreground">…</div>
        {:else if error}
          <div class="flex h-full items-center justify-center px-4 text-center text-xs text-destructive">{error}</div>
        {:else if visibleEntries.length === 0}
          <div class="flex h-full items-center justify-center text-xs text-muted-foreground">{$t("file.empty")}</div>
        {:else}
          {#each visibleEntries as entry (entry.path)}
            <button
              onclick={() => handleEntryClick(entry)}
              ondblclick={() => !entry.is_dir && !isSave && (selected = entry) && confirm()}
              class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-xs transition-colors
                {selected === entry ? 'bg-accent text-foreground' : 'text-muted-foreground hover:bg-accent/60 hover:text-foreground'}"
            >
              {#if entry.is_dir}
                {#if currentDir === ""}
                  <DriveIcon class="size-3.5 shrink-0 text-primary/80" />
                {:else}
                  <FolderIcon class="size-3.5 shrink-0 text-primary/80" />
                {/if}
              {:else}
                <FileIcon class="size-3.5 shrink-0" />
              {/if}
              <span class="truncate">{entry.name}</span>
            </button>
          {/each}
        {/if}
      </div>

      <!-- Filename (save mode) -->
      {#if isSave}
        <div class="space-y-1.5">
          <span class="text-xs font-medium text-muted-foreground">{$t("file.filename")}</span>
          <Input bind:value={filename} class="h-8 text-xs" />
        </div>
      {/if}

      <div class="flex justify-end gap-2 pt-1">
        <Button variant="outline" size="sm" onclick={() => answerFileDialog(null)}>
          {opts.cancelLabel || $t("common.cancel")}
        </Button>
        <Button size="sm" onclick={confirm} disabled={!canConfirm}>
          {opts.okLabel || (isSave ? $t("file.saveBtn") : $t("file.openBtn"))}
        </Button>
      </div>
    </div>
  </div>
{/if}
