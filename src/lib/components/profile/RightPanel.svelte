<script lang="ts">
  import ButtonAssignRow from "$lib/components/profile/ButtonAssignRow.svelte";
  import RightStickSettings from "$lib/components/profile/RightStickSettings.svelte";
  import { profile } from "$lib/stores/profile";
  import { t } from "$lib/stores/i18n";
  import GamepadIcon from "~icons/solar/gamepad-minimalistic-bold-duotone";
</script>

<aside class="flex flex-col h-full overflow-y-auto">
  <!-- Section: Right bumpers/triggers + Start -->
  <div class="px-3 py-3 border-b border-border/70">
    <h2 class="flex items-center gap-1.5 text-xs font-semibold text-muted-foreground uppercase tracking-wide mb-2.5">
      <span class="size-1 rounded-full bg-primary"></span>
      {$t("panel.rightTriggers")}
    </h2>
    <div class="space-y-1.5">
      <ButtonAssignRow button="RB" label="RB" />
      <ButtonAssignRow button="RT" label="RT" />
      <ButtonAssignRow button="Start" label="Start (▶)" />
    </div>
  </div>

  <!-- Section: Face buttons A/B/X/Y -->
  <div class="px-3 py-3 border-b border-border/70">
    <h2 class="flex items-center gap-1.5 text-xs font-semibold text-muted-foreground uppercase tracking-wide mb-2.5">
      <span class="size-1 rounded-full bg-primary"></span>
      {$t("panel.faceButtons")}
    </h2>
    <div class="space-y-1.5">
      <ButtonAssignRow button="A" label="A" />
      <ButtonAssignRow button="B" label="B" />
      <ButtonAssignRow button="X" label="X" />
      <ButtonAssignRow button="Y" label="Y" />
    </div>
  </div>

  <!-- Section: Right stick -->
  <div class="px-3 py-3">
    <div class="flex items-center gap-2 mb-2.5">
      <div class="flex items-center justify-center size-5 rounded-md bg-accent">
        <GamepadIcon class="size-3 text-accent-foreground" />
      </div>
      <h2 class="text-xs font-semibold text-muted-foreground uppercase tracking-wide">{$t("panel.rightStickMouse")}</h2>
    </div>
    <div class="space-y-3">
      <ButtonAssignRow button="RightThumb" label={$t("panel.stickClick")} />
      <RightStickSettings />
    </div>
  </div>
</aside>
