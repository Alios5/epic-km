<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { listen } from "@tauri-apps/api/event";
  import { confirm } from "@tauri-apps/plugin-dialog";
  import { hasUnsavedChanges, activeProfileName } from "$lib/stores/profile";
  import { saveProfile } from "$lib/stores/profileStorage";
  import { t } from "$lib/stores/i18n";
  import { get } from "svelte/store";
  import { onMount } from "svelte";
  import MinusIcon from "@lucide/svelte/icons/minus";
  import SquareIcon from "@lucide/svelte/icons/square";
  import XIcon from "@lucide/svelte/icons/x";
  import CopyIcon from "@lucide/svelte/icons/copy";

  const appWindow = getCurrentWindow();

  let maximized = $state(false);
  let confirmingClose = $state(false);

  onMount(async () => {
    maximized = await appWindow.isMaximized();

    await listen("close-requested", async () => {
      if (confirmingClose) return;
      confirmingClose = true;
      await requestClose();
      confirmingClose = false;
    });
  });

  async function requestClose() {
    const dirty = get(hasUnsavedChanges);

    if (dirty) {
      const shouldSave = await confirm(
        $t("close.unsavedMsg"),
        {
          title: $t("close.unsavedTitle"),
          kind: "warning",
          okLabel: $t("close.saveAndClose"),
          cancelLabel: $t("close.closeWithoutSaving"),
        }
      );

      if (shouldSave) {
        const name = get(activeProfileName) || $t("editor.defaultName");
        try {
          await saveProfile(name);
        } catch (e) {
          console.error("Failed to save on close:", e);
          // Still allow closing even if save fails
        }
      }
    } else {
      const confirmed = await confirm($t("close.confirmMsg"), {
        title: $t("close.confirmTitle"),
        kind: "warning",
        okLabel: $t("common.close"),
        cancelLabel: $t("common.cancel"),
      });
      if (!confirmed) return;
    }

    // Destroy the window directly since close is intercepted
    await appWindow.destroy();
  }

  async function checkMaximized() {
    maximized = await appWindow.isMaximized();
  }

  async function handleMinimize() {
    await appWindow.minimize();
  }

  async function handleToggleMaximize() {
    await appWindow.toggleMaximize();
    await checkMaximized();
  }

  async function handleClose() {
    if (confirmingClose) return;
    confirmingClose = true;
    await requestClose();
    confirmingClose = false;
  }
</script>

<div
  class="flex items-center justify-between h-9 px-2 bg-card border-b border-border select-none"
  data-tauri-drag-region
  role="button"
  tabindex="0"
  ondblclick={handleToggleMaximize}
>
  <!-- Left: app identity -->
  <div class="flex items-center gap-2 px-2" data-tauri-drag-region>
    <img src="/logo.svg" alt="Epic KM" class="size-5" draggable="false" />
    <span class="text-sm font-medium" data-tauri-drag-region>Epic KM</span>
  </div>

  <!-- Right: window controls -->
  <div class="flex items-center">
    <button
      onclick={handleMinimize}
      class="flex items-center justify-center size-8 rounded-sm hover:bg-accent transition-colors"
      aria-label={$t("win.minimize")}
    >
      <MinusIcon class="size-4" />
    </button>
    <button
      onclick={handleToggleMaximize}
      class="flex items-center justify-center size-8 rounded-sm hover:bg-accent transition-colors"
      aria-label={$t("win.maximize")}
    >
      {#if maximized}
        <CopyIcon class="size-3.5" />
      {:else}
        <SquareIcon class="size-3.5" />
      {/if}
    </button>
    <button
      onclick={handleClose}
      class="flex items-center justify-center size-8 rounded-sm hover:bg-destructive hover:text-destructive-foreground transition-colors"
      aria-label={$t("common.close")}
    >
      <XIcon class="size-4" />
    </button>
  </div>
</div>
