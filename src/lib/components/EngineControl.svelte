<script lang="ts">
  import { Button } from "$lib/components/ui/button/index.js";
  import LanguageSwitcher from "$lib/components/LanguageSwitcher.svelte";
  import { t } from "$lib/stores/i18n";
  import SettingsIcon from "~icons/solar/settings-bold-duotone";

  interface Props {
    onConfigure: () => void;
  }

  let { onConfigure }: Props = $props();
</script>

<div class="flex flex-col items-center gap-7 px-6 py-16 max-w-md mx-auto text-center">
  <div class="relative flex items-center justify-center">
    <div class="absolute inset-0 rounded-full bg-primary/25 blur-2xl scale-150"></div>
    <img
      src="/logo.svg"
      alt="Epic KM"
      class="relative size-20 rounded-full ring-1 ring-border shadow-lg"
      draggable="false"
    />
  </div>

  <div class="space-y-2.5">
    <h1 class="text-2xl font-semibold tracking-tight">Epic KM</h1>
    <div class="inline-flex items-center gap-1.5 rounded-full border border-border bg-card/80 px-3 py-1 shadow-sm">
      <span class="relative flex size-1.5">
        <span class="absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-500 opacity-75"></span>
        <span class="relative inline-flex size-1.5 rounded-full bg-emerald-500"></span>
      </span>
      <p class="text-xs text-muted-foreground">
        {$t("home.ready")}
      </p>
    </div>
  </div>

  <Button variant="outline" onclick={onConfigure} class="w-full h-10 shadow-sm">
    <SettingsIcon class="size-4" />
    {$t("home.configure")}
  </Button>

  <div class="flex flex-col items-center gap-2 pt-4">
    <span class="text-xs text-muted-foreground">{$t("home.language")}</span>
    <LanguageSwitcher />
  </div>
</div>
