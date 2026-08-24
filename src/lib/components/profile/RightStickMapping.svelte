<script lang="ts">
  import * as Select from "$lib/components/ui/select/index.js";
  import { Checkbox } from "$lib/components/ui/checkbox/index.js";
  import { profile, markDirty, type AxisInputMode } from "$lib/stores/profile";
  import { invoke } from "@tauri-apps/api/core";
  import { get } from "svelte/store";
  import { t } from "$lib/stores/i18n";

  function updateXMode(mode: AxisInputMode) {
    profile.update((p) => ({ ...p, rightStickXMode: mode }));
    markDirty();
    invoke("reload_profile", { profile: get(profile) }).catch(() => {});
  }

  function updateYMode(mode: AxisInputMode) {
    profile.update((p) => ({ ...p, rightStickYMode: mode }));
    markDirty();
    invoke("reload_profile", { profile: get(profile) }).catch(() => {});
  }

  // Push the profile to the running engine (debounced) so DSU toggle
  // changes apply live, without saving first
  let pushTimer: ReturnType<typeof setTimeout> | null = null;
  function pushToEngine() {
    if (pushTimer) clearTimeout(pushTimer);
    pushTimer = setTimeout(() => {
      invoke("reload_profile", { profile: get(profile) }).catch(() => {});
    }, 150);
  }

  function updateDsuEnabled(checked: boolean) {
    profile.update((p) => ({ ...p, dsuEnabled: checked }));
    markDirty();
    pushToEngine();
  }

  function updateDsuGravity(checked: boolean) {
    profile.update((p) => ({ ...p, dsuGravity: checked }));
    markDirty();
    pushToEngine();
  }
</script>

<!-- Gyroscope axis modes only exist for the DS4 target (an XUSB pad has
     no motion channels), so the whole section is hidden in Xbox 360 mode. -->
{#if $profile.controllerType === "ds4"}
  <section class="space-y-3">
    <!-- X axis mode selector -->
    <div class="space-y-1.5">
      <span class="text-xs text-muted-foreground">{$t("rsm.xAxis")}</span>
      <Select.Root
        type="single"
        value={$profile.rightStickXMode}
        onValueChange={(v) => v && updateXMode(v as AxisInputMode)}
      >
        <Select.Trigger class="h-8 w-full text-xs">
          {$profile.rightStickXMode === "analog" ? $t("rsm.analog") : $t("rsm.gyroscope")}
        </Select.Trigger>
        <Select.Content>
          <Select.Item value="analog" label={$t("rsm.analog")} />
          <Select.Item value="gyroscope" label={$t("rsm.gyroscope")} />
        </Select.Content>
      </Select.Root>
      <p class="text-[11px] text-muted-foreground">
        {$profile.rightStickXMode === "analog" ? $t("rsm.analogHint") : $t("rsm.gyroHint")}
      </p>
    </div>

    <!-- Y axis mode selector -->
    <div class="space-y-1.5">
      <span class="text-xs text-muted-foreground">{$t("rsm.yAxis")}</span>
      <Select.Root
        type="single"
        value={$profile.rightStickYMode}
        onValueChange={(v) => v && updateYMode(v as AxisInputMode)}
      >
        <Select.Trigger class="h-8 w-full text-xs">
          {$profile.rightStickYMode === "analog" ? $t("rsm.analog") : $t("rsm.gyroscope")}
        </Select.Trigger>
        <Select.Content>
          <Select.Item value="analog" label={$t("rsm.analog")} />
          <Select.Item value="gyroscope" label={$t("rsm.gyroscope")} />
        </Select.Content>
      </Select.Root>
      <p class="text-[11px] text-muted-foreground">
        {$profile.rightStickYMode === "analog" ? $t("rsm.analogHint") : $t("rsm.gyroHint")}
      </p>
    </div>

    <!-- DSU (Cemuhook) motion server: streams the gyro as plain floats over
         UDP 26760 — no HID calibration involved, so no rest drift. The
         trim/gravity settings below only affect the HID path. -->
    <div class="space-y-1.5 pt-1 border-t border-border">
      <div class="flex items-center justify-between">
        <label for="dsu-enable" class="text-xs font-medium cursor-pointer select-none">{$t("rsm.dsuEnable")}</label>
        <Checkbox id="dsu-enable" checked={$profile.dsuEnabled} onCheckedChange={updateDsuEnabled} />
      </div>
      <div class="flex items-center justify-between">
        <label for="dsu-gravity" class="text-xs font-medium cursor-pointer select-none">{$t("rsm.dsuGravity")}</label>
        <Checkbox id="dsu-gravity" checked={$profile.dsuGravity} onCheckedChange={updateDsuGravity} />
      </div>
      <p class="text-[11px] text-muted-foreground">{$t("rsm.dsuHint")}</p>
      <p class="text-[11px] text-muted-foreground">{$t("rsm.dsuGravityHint")}</p>
    </div>
  </section>
{/if}
