<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Badge } from "$lib/components/ui/badge/index.js";
  import * as Select from "$lib/components/ui/select/index.js";
  import { activeProfileName, profile, hasUnsavedChanges, markDirty, markClean, getDefaultProfile } from "$lib/stores/profile";
  import { listProfiles, loadProfile, deleteProfile } from "$lib/stores/profileStorage";
  import { captureModeActive, debugConsoleOpen } from "$lib/stores/app";
  import DownloadIcon from "~icons/solar/download-minimalistic-bold-duotone";
  import FolderIcon from "~icons/solar/folder-open-bold-duotone";
  import SaveIcon from "~icons/solar/diskette-bold-duotone";
  import TrashIcon from "~icons/solar/trash-bin-trash-bold-duotone";
  import ArrowLeftIcon from "~icons/solar/alt-arrow-left-bold-duotone";
  import FilePlusIcon from "~icons/solar/document-add-bold-duotone";
  import BugIcon from "~icons/solar/bug-bold-duotone";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { confirmDialog, fileDialog } from "$lib/stores/dialogs";
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
      const path = await fileDialog({
        mode: "save",
        title: $t("file.exportTitle"),
        extension: "json",
        defaultFilename: `${name}.json`,
      });
      if (!path) return;
      await invoke("export_profile", { path, data: get(profile) });
    } catch (e) {
      console.error("Export failed:", e);
    }
  }

  async function handleOpenProfile() {
    try {
      const selected = await fileDialog({
        mode: "open",
        title: $t("file.importTitle"),
        extension: "json",
      });
      if (selected) {
        // Copies the picked file into the internal profiles directory (under
        // its own name) and returns the name it was saved as, so it shows up
        // in the "Recents" list right away.
        const name = await invoke<string>("import_profile", { path: selected });
        await loadProfile(name);
        profileName = name;
        savedProfiles = await listProfiles();
      }
    } catch (e) {
      console.error("Failed to open profile file:", e);
    }
  }

  async function handleNewProfile() {
    if (get(hasUnsavedChanges)) {
      const proceed = await confirmDialog({
        title: $t("new.confirmTitle"),
        message: $t("new.confirmMsg", { name: profileName.trim() || $t("editor.defaultName") }),
        kind: "warning",
        okLabel: $t("new.create"),
        cancelLabel: $t("common.cancel"),
      });
      if (!proceed) return;
    }

    const def = getDefaultProfile();
    profile.set(def);
    activeProfileName.set("");
    profileName = "";
    markClean();

    try {
      await invoke("reload_profile", { profile: def });
    } catch {
      // Engine not running — ignore
    }
  }

  async function handleSelectProfile(name: string) {
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

<header class="flex items-center gap-3 px-4 py-2.5 border-b border-border/15 bg-card shadow-sm">
  <Button variant="ghost" size="sm" onclick={onBack}>
    <ArrowLeftIcon class="size-4" />
  </Button>

  <div class="h-5 w-px bg-border/25"></div>

  <!-- Left: capture status + hotkey config -->
  <div class="flex items-center gap-2">
    {#if $captureModeActive}
      <Badge variant="destructive" class="h-8 gap-1.5">
        <span class="relative flex size-1.5">
          <span class="absolute inline-flex h-full w-full animate-ping rounded-full bg-destructive opacity-75"></span>
          <span class="relative inline-flex size-1.5 rounded-full bg-destructive"></span>
        </span>
        {$t("capture.active")}
      </Badge>
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

  <div class="h-5 w-px bg-border/25"></div>

  <!-- Center: profile name + saved profiles dropdown -->
  <div class="flex-1 flex items-center justify-center gap-2">
    <Select.Root type="single" onValueChange={handleSelectProfile}>
      <Select.Trigger class="h-8 text-xs" title={$t("topbar.recent")}>
        {$t("topbar.recent")}
      </Select.Trigger>
      <Select.Content>
        {#each savedProfiles as name}
          <Select.Item value={name} label={name} />
        {/each}
      </Select.Content>
    </Select.Root>

    <Input
      value={profileName}
      oninput={onNameInput}
      placeholder={$t("topbar.profileName")}
      class="max-w-[200px] h-8 text-sm text-center shadow-sm"
    />
  </div>

  <!-- Right: actions -->
  <div class="flex items-center gap-1 rounded-lg border border-border bg-background/45 backdrop-blur-sm p-1">
    <Button variant="ghost" size="sm" class="h-7 w-7 p-0" aria-label={$t("topbar.newTitle")} title={$t("topbar.newTitle")} onclick={handleNewProfile}>
      <FilePlusIcon class="size-4" />
    </Button>
    <Button variant="ghost" size="sm" class="h-7 w-7 p-0" aria-label={$t("common.export")} title={$t("common.export")} onclick={handleExport}>
      <DownloadIcon class="size-4" />
    </Button>
    <Button variant="ghost" size="sm" class="h-7 w-7 p-0" aria-label={$t("common.openFile")} onclick={handleOpenProfile}>
      <FolderIcon class="size-4" />
    </Button>
    <Button variant="ghost" size="sm" class="h-7 w-7 p-0" aria-label={$t("common.delete")} title={$t("common.delete")} onclick={() => showDeleteDialog = true}>
      <TrashIcon class="size-4 text-destructive" />
    </Button>
    <div class="h-5 w-px bg-border/25 mx-0.5"></div>
    <Button
      variant={$debugConsoleOpen ? "secondary" : "ghost"}
      size="sm"
      class="h-7 w-7 p-0"
      aria-label={$t("debug.toggle")}
      title={$t("debug.toggle")}
      onclick={() => debugConsoleOpen.update((v) => !v)}
    >
      <BugIcon class="size-4" />
    </Button>
    <div class="h-5 w-px bg-border/25 mx-0.5"></div>
    <Button variant="default" size="sm" class="h-7 shadow-sm" onclick={onSave}>
      <SaveIcon class="size-4" />
      {$t("common.save")}
    </Button>
  </div>
</header>

{#if showDeleteDialog}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
    <div role="dialog" aria-modal="true" aria-label={$t("delete.title")} class="elevated-panel rounded-xl p-6 max-w-sm w-full mx-4">
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
