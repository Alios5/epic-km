<script lang="ts">
  import { Button } from "$lib/components/ui/button/index.js";
  import LanguageSwitcher from "$lib/components/LanguageSwitcher.svelte";
  import GamepadArt from "$lib/components/GamepadArt.svelte";
  import { t } from "$lib/stores/i18n";
  import GlobeIcon from "~icons/solar/earth-bold-duotone";
  import PlayIcon from "~icons/solar/play-bold-duotone";

  interface Props {
    onConfigure: () => void;
  }

  let { onConfigure }: Props = $props();
</script>

<div class="relative h-full overflow-hidden">
  <!-- Decorative controller art, faded into the backdrop -->
  <GamepadArt
    class="pointer-events-none absolute -right-14 top-1/2 w-[520px] max-w-none -translate-y-1/2 -rotate-8 text-primary/10"
  />
  <GamepadArt
    class="pointer-events-none absolute -bottom-30 -left-24 w-[360px] max-w-none rotate-12 text-foreground/4"
  />
  <div class="pointer-events-none absolute right-0 top-1/2 size-[420px] -translate-y-1/2 rounded-full bg-primary/10 blur-3xl"></div>

  <!-- Content column, left-aligned -->
  <div class="relative flex h-full flex-col px-14 py-10">
    <div class="flex items-center gap-3">
      <div class="relative flex items-center justify-center">
        <div class="absolute inset-0 rounded-full bg-primary/25 blur-xl scale-140"></div>
        <img
          src="/logo.svg"
          alt="Epic KM"
          class="relative size-11 rounded-full ring-1 ring-border shadow-lg"
          draggable="false"
        />
      </div>
      <div class="inline-flex items-center gap-1.5 rounded-full border border-border bg-card/60 backdrop-blur-sm px-3 py-1 shadow-sm">
        <span class="relative flex size-1.5">
          <span class="absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-500 opacity-75"></span>
          <span class="relative inline-flex size-1.5 rounded-full bg-emerald-500"></span>
        </span>
        <p class="text-xs text-muted-foreground">
          {$t("home.ready")}
        </p>
      </div>
    </div>

    <div class="flex flex-1 flex-col justify-center gap-6 max-w-md">
      <div class="space-y-3">
        <h1 class="font-serif text-5xl tracking-tight">Epic KM</h1>
        <p class="text-sm leading-relaxed text-muted-foreground">
          {$t("home.tagline")}
        </p>
      </div>

      <Button onclick={onConfigure} class="h-11 w-fit px-6 shadow-sm">
        <PlayIcon class="size-4" />
        {$t("home.configure")}
      </Button>
    </div>

    <div class="flex items-center gap-2.5">
      <GlobeIcon class="size-3.5 text-muted-foreground" />
      <span class="text-xs text-muted-foreground">{$t("home.language")}</span>
      <LanguageSwitcher />
    </div>
  </div>
</div>
