<script lang="ts">
  import KeyAssignPopover from "$lib/components/KeyAssignPopover.svelte";
  import GamepadArt from "$lib/components/GamepadArt.svelte";
  import { t } from "$lib/stores/i18n";

  // Clickable zones overlaid on the controller art, in viewBox coordinates.
  // Each zone opens the same assignment popover as the side panels.
  // LT/RT (L2/R2) are not visible on a front view, so they get small
  // labelled tabs at the top corners.
  interface Zone {
    button: string;
    x: number;
    y: number;
    w: number;
    h: number;
    circle?: boolean;
    zoneLabel?: string;
  }

  // Xbox 360 art: inline SVG, viewBox 256 x 175.
  const XBOX_VW = 256;
  const XBOX_VH = 175;
  const xboxZones: Zone[] = [
    { button: "LT", x: 6, y: 1, w: 34, h: 12, zoneLabel: "LT" },
    { button: "RT", x: 216, y: 1, w: 34, h: 12, zoneLabel: "RT" },
    { button: "LB", x: 36, y: 2, w: 60, h: 18 },
    { button: "RB", x: 160, y: 2, w: 60, h: 18 },
    { button: "Back", x: 103, y: 47, w: 14, h: 14, circle: true },
    { button: "Start", x: 140, y: 47, w: 14, h: 14, circle: true },
    { button: "Y", x: 185, y: 28, w: 18, h: 18, circle: true },
    { button: "X", x: 167, y: 45, w: 18, h: 18, circle: true },
    { button: "B", x: 202, y: 45, w: 18, h: 18, circle: true },
    { button: "A", x: 185, y: 62, w: 18, h: 18, circle: true },
    { button: "LeftThumb", x: 43, y: 34, w: 39, h: 39, circle: true },
    { button: "RightThumb", x: 142, y: 73, w: 39, h: 39, circle: true },
    { button: "DPadUp", x: 88, y: 77, w: 13, h: 12 },
    { button: "DPadDown", x: 88, y: 102, w: 13, h: 12 },
    { button: "DPadLeft", x: 76, y: 89, w: 12, h: 13 },
    { button: "DPadRight", x: 101, y: 89, w: 12, h: 13 },
  ];

  const zones = xboxZones;
  const vw = XBOX_VW;
  const vh = XBOX_VH;
</script>

<div class="flex flex-col items-center justify-center gap-3 py-8">
  <div class="relative">
    <div class="pointer-events-none absolute inset-0 -z-10 rounded-full bg-primary/20 blur-3xl scale-110"></div>
    <GamepadArt class="block w-[320px] text-[#3f3f46] drop-shadow-lg" />

    {#each zones as z}
      <div
        class="absolute {z.circle ? 'rounded-full' : 'rounded-md'}"
        style="left: {(z.x / vw) * 100}%; top: {(z.y / vh) * 100}%; width: {(z.w / vw) * 100}%; height: {(z.h / vh) * 100}%;"
      >
        <KeyAssignPopover button={z.button} zone zoneLabel={z.zoneLabel} />
      </div>
    {/each}
  </div>

  <p class="text-xs text-muted-foreground">
    {$t("diagram.hint")}
  </p>
</div>
