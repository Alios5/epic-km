<script lang="ts">
  import * as Select from "$lib/components/ui/select/index.js";
  import { Slider } from "$lib/components/ui/slider/index.js";
  import { Checkbox } from "$lib/components/ui/checkbox/index.js";
  import { profile, markDirty, type AxisInputMode, type GyroRestAccel } from "$lib/stores/profile";
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

  // Push the profile to the running engine (debounced: sliders fire many
  // events while dragging) so trim changes apply live, without saving first
  let pushTimer: ReturnType<typeof setTimeout> | null = null;
  function pushToEngine() {
    if (pushTimer) clearTimeout(pushTimer);
    pushTimer = setTimeout(() => {
      invoke("reload_profile", { profile: get(profile) }).catch(() => {});
    }, 150);
  }

  function updateBiasPitch(v: number) {
    profile.update((p) => ({ ...p, gyroBiasPitch: v }));
    markDirty();
    pushToEngine();
  }

  function updateBiasYaw(v: number) {
    profile.update((p) => ({ ...p, gyroBiasYaw: v }));
    markDirty();
    pushToEngine();
  }

  function updateRestAccel(v: GyroRestAccel) {
    profile.update((p) => ({ ...p, gyroRestAccel: v }));
    markDirty();
    pushToEngine();
  }

  function updateDsuEnabled(checked: boolean) {
    profile.update((p) => ({ ...p, dsuEnabled: checked }));
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
      <p class="text-[11px] text-muted-foreground">{$t("rsm.dsuHint")}</p>
    </div>

    <!-- Gyro drift compensation: rest-offset trims (raw LSB) letting the
         user zero out the residual bias their motion reader assumes. -->
    <div class="space-y-3 pt-1 border-t border-border">
      <span class="text-xs text-muted-foreground">{$t("rsm.trimTitle")}</span>
      <!-- Rest gravity orientation: when the drift ignores the gyro trims,
           the reader's horizon correction is pulling on the accelerometer —
           switching axis live pinpoints the convention it expects. -->
      <div class="space-y-1.5">
        <span class="text-[11px] font-medium">{$t("rsm.restAccel")}</span>
        <Select.Root
          type="single"
          value={$profile.gyroRestAccel}
          onValueChange={(v) => v && updateRestAccel(v as GyroRestAccel)}
        >
          <Select.Trigger class="h-8 w-full text-xs">
            {$profile.gyroRestAccel === "neg_y"
              ? $t("rsm.restAccelNegY")
              : $profile.gyroRestAccel === "pos_y"
                ? $t("rsm.restAccelPosY")
                : $profile.gyroRestAccel === "neg_z"
                  ? $t("rsm.restAccelNegZ")
                  : $profile.gyroRestAccel === "pos_z"
                    ? $t("rsm.restAccelPosZ")
                    : $t("rsm.restAccelZero")}
          </Select.Trigger>
          <Select.Content>
            <Select.Item value="neg_y" label={$t("rsm.restAccelNegY")} />
            <Select.Item value="pos_y" label={$t("rsm.restAccelPosY")} />
            <Select.Item value="neg_z" label={$t("rsm.restAccelNegZ")} />
            <Select.Item value="pos_z" label={$t("rsm.restAccelPosZ")} />
            <Select.Item value="zero" label={$t("rsm.restAccelZero")} />
          </Select.Content>
        </Select.Root>
        <p class="text-[11px] text-muted-foreground">{$t("rsm.restAccelHint")}</p>
      </div>
      <div class="space-y-1.5">
        <div class="flex items-center justify-between">
          <span class="text-[11px] font-medium">{$t("rsm.trimPitch")}</span>
          <span class="text-[11px] text-muted-foreground tabular-nums">{$profile.gyroBiasPitch}</span>
        </div>
        <Slider type="single" value={$profile.gyroBiasPitch} onValueChange={updateBiasPitch} min={-8} max={8} step={1} />
      </div>
      <div class="space-y-1.5">
        <div class="flex items-center justify-between">
          <span class="text-[11px] font-medium">{$t("rsm.trimYaw")}</span>
          <span class="text-[11px] text-muted-foreground tabular-nums">{$profile.gyroBiasYaw}</span>
        </div>
        <Slider type="single" value={$profile.gyroBiasYaw} onValueChange={updateBiasYaw} min={-8} max={8} step={1} />
      </div>
      <p class="text-[11px] text-muted-foreground">{$t("rsm.trimHint")}</p>
    </div>
  </section>
{/if}
