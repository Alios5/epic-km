<script lang="ts">
  import { profile } from "$lib/stores/profile";
  import ButtonAssignRow from "$lib/components/profile/ButtonAssignRow.svelte";
  import StickSettings from "$lib/components/profile/StickSettings.svelte";
  import LeftStickMapping from "$lib/components/profile/LeftStickMapping.svelte";
  import { t } from "$lib/stores/i18n";
  import GamepadIcon from "~icons/solar/gamepad-minimalistic-bold-duotone";
  import LeftTriggerIcon from "~icons/solar/double-alt-arrow-left-bold-duotone";
  import CompassIcon from "~icons/solar/compass-bold-duotone";
</script>

<aside class="flex flex-col h-full overflow-y-auto">
  <!-- Section: Left bumpers/triggers + Back -->
  <div class="px-3 py-3 border-b border-border/15">
    <div class="flex items-center gap-2 mb-2.5">
      <div class="flex items-center justify-center size-5 rounded-md bg-accent">
        <LeftTriggerIcon class="size-3 text-accent-foreground" />
      </div>
      <h2 class="text-xs font-semibold text-muted-foreground uppercase tracking-wide">{$t("panel.leftTriggers")}</h2>
    </div>
    <div class="space-y-1.5">
      <ButtonAssignRow button="LB" label="LB" />
      <ButtonAssignRow button="LT" label="LT" />
      <ButtonAssignRow button="Back" label="Back (○)" />
    </div>
  </div>

  <!-- Section: Left stick -->
  <div class="px-3 py-3 border-b border-border/15">
    <div class="flex items-center gap-2 mb-2.5">
      <div class="flex items-center justify-center size-5 rounded-md bg-accent">
        <GamepadIcon class="size-3 text-accent-foreground" />
      </div>
      <h2 class="text-xs font-semibold text-muted-foreground uppercase tracking-wide">{$t("panel.leftStick")}</h2>
    </div>
    <div class="space-y-3">
      <ButtonAssignRow button="LeftThumb" label={$t("panel.stickClick")} />
      <StickSettings stickKey="leftStick" />
      <LeftStickMapping />
    </div>
  </div>

  <!-- Section: D-Pad -->
  <div class="px-3 py-3">
    <div class="flex items-center gap-2 mb-2.5">
      <div class="flex items-center justify-center size-5 rounded-md bg-accent">
        <CompassIcon class="size-3 text-accent-foreground" />
      </div>
      <h2 class="text-xs font-semibold text-muted-foreground uppercase tracking-wide">{$t("panel.dpad")}</h2>
    </div>
    <div class="space-y-1.5">
      <ButtonAssignRow button="DPadUp" label={$t("dir.up")} />
      <ButtonAssignRow button="DPadDown" label={$t("dir.down")} />
      <ButtonAssignRow button="DPadLeft" label={$t("dir.left")} />
      <ButtonAssignRow button="DPadRight" label={$t("dir.right")} />
    </div>
  </div>
</aside>
