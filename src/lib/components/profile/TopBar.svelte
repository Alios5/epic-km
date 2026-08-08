<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Badge } from "$lib/components/ui/badge/index.js";
  import { activeProfileName, profile, markDirty } from "$lib/stores/profile";
  import { listProfiles, loadProfile } from "$lib/stores/profileStorage";
  import { captureModeActive } from "$lib/stores/app";
  import DownloadIcon from "@lucide/svelte/icons/download";
  import FolderIcon from "@lucide/svelte/icons/folder-open";
  import SaveIcon from "@lucide/svelte/icons/save";
  import TrashIcon from "@lucide/svelte/icons/trash-2";
  import ArrowLeftIcon from "@lucide/svelte/icons/arrow-left";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { get } from "svelte/store";
  import { layoutMap, labelForCode } from "$lib/keyLabels";
  import { t, locale } from "$lib/stores/i18n";

  interface Props {
    onBack: () => void;
    onSave: () => void;
  }

  let { onBack, onSave }: Props = $props();

  let profileName = $state($activeProfileName);
  let assigningKey = $state(false);
  let savedProfiles = $state<string[]>([]);
  let unlistenFn: (() => void) | null = null;

  onMount(async () => {
    try {
      savedProfiles = await listProfiles();
    } catch (e) {
      console.error("Failed to list profiles:", e);
    }

    // Listen for capture mode changes BEFORE starting the engine
    unlistenFn = await listen<boolean>("capture-mode-changed", (event) => {
      captureModeActive.set(event.payload);
    });

    // Initialize the hotkey watcher with the current profile
    try {
      await invoke("init_watcher", { profile: get(profile) });
    } catch (e) {
      console.error("Engine init failed:", e);
    }
  });

  onDestroy(() => {
    if (unlistenFn) unlistenFn();
    invoke("shutdown_watcher").catch(() => {});
  });

  function onNameInput(e: Event) {
    const target = e.target as HTMLInputElement;
    profileName = target.value;
    activeProfileName.set(target.value);
    markDirty();
  }

  async function handleExport() {
    try {
      const name = profileName.trim() || $t("editor.defaultName");
      const path = await save({
        defaultPath: `${name}.json`,
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (!path) return;
      await invoke("export_profile", { path, data: get(profile) });
    } catch (e) {
      console.error("Export failed:", e);
    }
  }

  async function handleOpenProfile() {
    try {
      const selected = await open({
        filters: [{ name: "Profil", extensions: ["json"] }],
        multiple: false,
      });
      if (selected && typeof selected === "string") {
        const name = selected.split(/[\\/]/).pop()?.replace(/\.json$/, "") ?? "";
        if (name) {
          await loadProfile(name);
          profileName = name;
          savedProfiles = await listProfiles();
        }
      }
    } catch (e) {
      console.error("Failed to open profile file:", e);
    }
  }

  async function handleSelectProfile(e: Event) {
    const target = e.target as HTMLSelectElement;
    const name = target.value;
    if (!name) return;
    try {
      await loadProfile(name);
      profileName = name;
    } catch (e) {
      console.error("Failed to load profile:", e);
    }
  }

  function startAssignKey() {
    assigningKey = true;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (assigningKey) {
      e.preventDefault();
      e.stopPropagation();
      profile.update((p) => ({ ...p, captureToggleKey: e.code }));
      markDirty();
      assigningKey = false;
      // Push the new profile to the engine so the hotkey is re-registered
      // immediately with the new key (no restart needed)
      invoke("reload_profile", { profile: get(profile) }).catch(() => {});
      return;
    }
    // Capture mode toggle is handled globally by the Rust backend via RegisterHotKey
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<header class="flex items-center gap-3 px-4 py-2.5 border-b border-border bg-card">
  <Button variant="ghost" size="sm" onclick={onBack}>
    <ArrowLeftIcon class="size-4" />
  </Button>

  <div class="h-5 w-px bg-border"></div>

  <!-- Left: capture status + hotkey config -->
  <div class="flex items-center gap-2">
    {#if $captureModeActive}
      <Badge variant="destructive" class="h-8">{$t("capture.active")}</Badge>
    {:else}
      <Badge variant="secondary" class="h-8">{$t("capture.inactive")}</Badge>
    {/if}

    <span class="text-xs text-muted-foreground">{$t("topbar.key")}</span>

    {#if assigningKey}
      <div class="rounded-md border border-ring bg-muted px-2 py-1 text-center text-xs animate-pulse whitespace-nowrap">
        {$t("topbar.pressKey")}
      </div>
    {:else}
      <button
        onclick={startAssignKey}
        class="rounded-md px-2 py-1 text-xs font-mono transition-colors hover:bg-accent border border-border whitespace-nowrap
        {$profile.captureToggleKey ? 'text-foreground' : 'text-muted-foreground'}"
        title={$t("topbar.toggleKeyTitle")}
      >
        {labelForCode($profile.captureToggleKey, $layoutMap, $locale)}
      </button>
    {/if}
  </div>

  <div class="h-5 w-px bg-border"></div>

  <!-- Center: profile name + saved profiles dropdown -->
  <div class="flex-1 flex items-center justify-center gap-2">
    <select
      onchange={handleSelectProfile}
      class="h-8 rounded-md border border-border bg-background px-2 text-xs text-muted-foreground cursor-pointer hover:bg-accent transition-colors"
      title={$t("topbar.recent")}
    >
      <option value="">{$t("topbar.recent")}</option>
      {#each savedProfiles as name}
        <option value={name}>{name}</option>
      {/each}
    </select>

    <Input
      value={profileName}
      oninput={onNameInput}
      placeholder={$t("topbar.profileName")}
      class="max-w-[200px] h-8 text-sm text-center border border-border"
    />
  </div>

  <!-- Right: actions -->
  <div class="flex items-center gap-1">
    <Button variant="ghost" size="sm" class="h-8 w-8 p-0" aria-label={$t("common.export")} title={$t("common.export")} onclick={handleExport}>
      <DownloadIcon class="size-4" />
    </Button>
    <Button variant="ghost" size="sm" class="h-8 w-8 p-0" aria-label={$t("common.openFile")} onclick={handleOpenProfile}>
      <FolderIcon class="size-4" />
    </Button>
    <Button variant="ghost" size="sm" class="h-8 w-8 p-0" aria-label={$t("common.delete")}>
      <TrashIcon class="size-4 text-destructive" />
    </Button>
    <Button variant="default" size="sm" class="h-8" onclick={onSave}>
      <SaveIcon class="size-4" />
      {$t("common.save")}
    </Button>
  </div>
</header>
