<script lang="ts">
  import { Button } from "$lib/components/ui/button/index.js";
  import LanguageSwitcher from "$lib/components/LanguageSwitcher.svelte";
  import { t } from "$lib/stores/i18n";
  import SettingsIcon from "@lucide/svelte/icons/settings";

  interface Props {
    onConfigure: () => void;
  }

  let { onConfigure }: Props = $props();
</script>

<div class="flex flex-col items-center gap-6 px-6 py-16 max-w-md mx-auto text-center">
  <img src="/logo.svg" alt="Epic KM" class="size-20 rounded-full drop-shadow-md" draggable="false" />

  <div class="space-y-2">
    <h1 class="text-xl font-semibold tracking-tight">Epic KM</h1>
    <p class="text-sm text-muted-foreground">
      {$t("home.ready")}
    </p>
  </div>

  <Button variant="outline" onclick={onConfigure} class="w-full">
    <SettingsIcon class="size-4" />
    {$t("home.configure")}
  </Button>

  <div class="flex flex-col items-center gap-2 pt-4">
    <span class="text-xs text-muted-foreground">{$t("home.language")}</span>
    <LanguageSwitcher />
  </div>
</div>
