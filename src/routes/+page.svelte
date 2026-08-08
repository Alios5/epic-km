<script lang="ts">
  import { onMount } from "svelte";
  import { checkState, prerequisitesInstalled, checkPrerequisites } from "$lib/stores/app";
  import PrerequisiteMissing from "$lib/components/PrerequisiteMissing.svelte";
  import EngineControl from "$lib/components/EngineControl.svelte";
  import ProfileEditor from "$lib/components/ProfileEditor.svelte";
  import { t } from "$lib/stores/i18n";

  type Screen = "home" | "profile";
  let screen = $state<Screen>("home");

  onMount(() => {
    checkPrerequisites();
  });
</script>

<main class="flex-1 min-h-0 h-full bg-background text-foreground">
  {#if screen === "profile"}
    <ProfileEditor onBack={() => (screen = "home")} />
  {:else if $checkState === "checking" || $checkState === "idle"}
    <div class="flex flex-col items-center justify-center gap-4 h-full">
      <div class="size-8 animate-spin rounded-full border-2 border-muted border-t-primary"></div>
      <p class="text-sm text-muted-foreground">{$t("app.checking")}</p>
    </div>
  {:else if $prerequisitesInstalled}
    <div class="h-full overflow-y-auto overscroll-none">
      <EngineControl onConfigure={() => (screen = "profile")} />
    </div>
  {:else}
    <div class="h-full overflow-y-auto overscroll-none">
      <PrerequisiteMissing />
    </div>
  {/if}
</main>
