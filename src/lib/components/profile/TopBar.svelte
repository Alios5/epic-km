<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Badge } from "$lib/components/ui/badge/index.js";
  import { activeProfileName, profile, markDirty, markClean, getDefaultProfile } from "$lib/stores/profile";
  import { listProfiles, loadProfile, deleteProfile } from "$lib/stores/profileStorage";
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
  let showDeleteDialog = $state(false);
  let deleteStatus = $state<"idle" | "deleting" | "success" | "error">("idle");
  let deleteMessage = $state("");

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

  async function handleDeleteProfile() {
    const name = profileName.trim();
    if (!name) return;
    deleteStatus = "deleting";
    try {
      await deleteProfile(name);
      // Reset to default profile
      const def = getDefaultProfile();
      profile.set(def);
      activeProfileName.set($t("editor.defaultName"));
      profileName = $t("editor.defaultName");
      markClean();
      // Reload engine with default profile
      try {
        await invoke("reload_profile", { profile: def });
      } catch {
        // Engine not running — ignore
      }
      // Refresh saved profiles list
      savedProfiles = await listProfiles();
      // Close dialog immediately
      showDeleteDialog = false;
      deleteStatus = "idle";
      deleteMessage = "";
    } catch (e) {
      deleteStatus = "error";
      deleteMessage = $t("delete.error", { error: String(e) });
    }
  }

  function startAssignKey() {
    // Suspend the global hotkey while assigning: pressing the current toggle
    // key must not toggle capture in the middle of the assignment.
    invoke("suspend_hotkey").catch(() => {});
    assigningKey = true;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (showDeleteDialog && e.code === "Escape" && deleteStatus !== "deleting") {
      showDeleteDialog = false;
      return;
    }
    if (assigningKey) {
      e.preventDefault();
      e.stopPropagation();
      assigningKey = false;
      if (e.code !== "Escape") {
        profile.update((p) => ({ ...p, captureToggleKey: e.code }));
        markDirty();
      }
      // Push the profile to the engine so the hotkey is re-registered
      // immediately — with the new key, or the old one if cancelled (Escape).
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
    <Button variant="ghost" size="sm" class="h-8 w-8 p-0" aria-label={$t("common.delete")} title={$t("common.delete")} onclick={() => showDeleteDialog = true}>
      <TrashIcon class="size-4 text-destructive" />
    </Button>
    <Button variant="default" size="sm" class="h-8" onclick={onSave}>
      <SaveIcon class="size-4" />
      {$t("common.save")}
    </Button>
  </div>
</header>

{#if showDeleteDialog}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
    <div role="dialog" aria-modal="true" aria-label={$t("delete.title")} class="rounded-lg border border-border bg-card p-6 shadow-xl max-w-sm w-full mx-4">
      <h2 class="text-lg font-semibold mb-2">{$t("delete.title")}</h2>
      <p class="text-sm text-muted-foreground mb-6">
        {$t("delete.message", { name: profileName.trim() || $t("editor.defaultName") })}
      </p>
      {#if deleteStatus === "error"}
        <div class="mb-4 rounded-md border border-destructive/50 bg-destructive/10 px-3 py-2 text-sm text-destructive">
          {deleteMessage}
        </div>
      {/if}
      <div class="flex justify-end gap-2">
        <Button variant="outline" size="sm" onclick={() => showDeleteDialog = false} disabled={deleteStatus === "deleting"}>
          {$t("common.cancel")}
        </Button>
        <Button variant="destructive" size="sm" onclick={handleDeleteProfile} disabled={deleteStatus === "deleting"}>
          {deleteStatus === "deleting" ? "…" : $t("common.confirm")}
        </Button>
      </div>
    </div>
  </div>
{/if}
