<script lang="ts">
  import { profile, activeProfileName } from "$lib/stores/profile";
  import { saveProfile } from "$lib/stores/profileStorage";
  import TopBar from "$lib/components/profile/TopBar.svelte";
  import BottomBar from "$lib/components/profile/BottomBar.svelte";
  import LeftPanel from "$lib/components/profile/LeftPanel.svelte";
  import RightPanel from "$lib/components/profile/RightPanel.svelte";
  import GamepadDiagram from "$lib/components/GamepadDiagram.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { get } from "svelte/store";
  import { t } from "$lib/stores/i18n";

  interface Props {
    onBack: () => void;
  }

  let { onBack }: Props = $props();

  let saveStatus = $state<"idle" | "saving" | "success" | "error">("idle");
  let saveMessage = $state("");

  async function handleSave() {
    const name = get(activeProfileName)?.trim() || $t("editor.defaultName");

    if (!name) {
      saveStatus = "error";
      saveMessage = $t("editor.emptyName");
      setTimeout(() => { saveStatus = "idle"; }, 3000);
      return;
    }

    saveStatus = "saving";
    try {
      await saveProfile(name);
      try {
        await invoke("reload_profile", { profile: get(profile) });
      } catch {
        // Engine not running — ignore
      }
      saveStatus = "success";
      saveMessage = $t("editor.saveSuccess", { name });
    } catch (e) {
      saveStatus = "error";
      saveMessage = $t("editor.saveError", { error: String(e) });
    }
    setTimeout(() => { saveStatus = "idle"; }, 3000);
  }
</script>

<div class="flex flex-col h-full bg-background text-foreground overflow-hidden">
  <TopBar onBack={onBack} onSave={handleSave} />

  <div class="flex flex-1 min-h-0 overflow-hidden">
    <!-- Left panel: LB/LT/Back, left stick, D-Pad -->
    <div class="w-72 shrink-0 border-r border-border bg-card/50 overflow-y-auto overscroll-none">
      <LeftPanel />
    </div>

    <!-- Center: gamepad diagram (visual only) -->
    <div class="flex-1 flex items-center justify-center overflow-hidden p-4">
      <GamepadDiagram />
    </div>

    <!-- Right panel: RB/RT/Start, ABXY, right stick -->
    <div class="w-72 shrink-0 border-l border-border bg-card/50 overflow-y-auto overscroll-none">
      <RightPanel />
    </div>
  </div>

  <BottomBar onSave={handleSave} />

  {#if saveStatus !== "idle"}
    <div
      class="fixed bottom-14 left-1/2 -translate-x-1/2 z-50 rounded-md border px-4 py-2 text-sm shadow-lg transition-opacity
      {saveStatus === "success" ? 'border-border bg-card text-foreground' : ''}
      {saveStatus === "error" ? 'border-destructive/50 bg-destructive/10 text-destructive' : ''}
      {saveStatus === "saving" ? 'border-border bg-card text-muted-foreground' : ''}"
    >
      {saveStatus === "saving" ? $t("editor.saving") : saveMessage}
    </div>
  {/if}
</div>
