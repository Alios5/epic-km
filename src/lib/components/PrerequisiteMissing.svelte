<script lang="ts">
  import { onDestroy } from "svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import LanguageSwitcher from "$lib/components/LanguageSwitcher.svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { checkPrerequisites, vigemStatus, VIGEMBUS_REPO_URL } from "$lib/stores/app";
  import { t } from "$lib/stores/i18n";

  let autoChecking = $state(false);
  let pollInterval: ReturnType<typeof setInterval> | null = null;

  async function handleInstall() {
    await openUrl(VIGEMBUS_REPO_URL);
    // Start auto-polling every 3 seconds
    autoChecking = true;
    pollInterval = setInterval(async () => {
      await checkPrerequisites();
    }, 3000);
  }

  async function handleRecheck() {
    await checkPrerequisites();
  }

  onDestroy(() => {
    if (pollInterval) clearInterval(pollInterval);
  });
</script>

<div class="flex flex-col items-center gap-6 px-6 py-16 max-w-md mx-auto text-center">
  <div class="flex size-16 items-center justify-center rounded-full bg-destructive/10">
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="32"
      height="32"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
      class="text-destructive"
    >
      <path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
      <line x1="12" y1="9" x2="12" y2="13" />
      <line x1="12" y1="17" x2="12.01" y2="17" />
    </svg>
  </div>

  <div class="space-y-2">
    <h1 class="text-xl font-semibold tracking-tight">
      {$t("prereq.title")}
    </h1>
    <p class="text-sm text-muted-foreground leading-relaxed">
      {$t("prereq.intro")}
      <strong class="text-foreground">ViGEmBus</strong>. {$t("prereq.purpose")}
    </p>
  </div>

  <div class="flex flex-col gap-2 w-full">
    <Button onclick={handleInstall} class="w-full">
      {$t("prereq.download")}
    </Button>
    <Button variant="outline" onclick={handleRecheck} class="w-full">
      {$t("prereq.recheck")}
    </Button>
  </div>

  {#if $vigemStatus === "not-responding"}
    <p class="text-xs text-amber-500 leading-relaxed">
      {$t("prereq.notResponding")}
    </p>
  {/if}

  {#if autoChecking}
    <div class="flex items-center gap-2 text-xs text-muted-foreground">
      <div class="size-3 animate-spin rounded-full border border-muted-foreground border-t-transparent"></div>
      {$t("prereq.autoDetect")}
    </div>
  {:else}
    <p class="text-xs text-muted-foreground">
      {$t("prereq.manualHint")}
    </p>
  {/if}

  <LanguageSwitcher />
</div>
