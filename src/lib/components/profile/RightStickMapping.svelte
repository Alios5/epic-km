<script lang="ts">
  import * as Select from "$lib/components/ui/select/index.js";
  import { Slider } from "$lib/components/ui/slider/index.js";
  import { Checkbox } from "$lib/components/ui/checkbox/index.js";
  import { profile, markDirty, type AxisInputMode } from "$lib/stores/profile";
  import { invoke } from "@tauri-apps/api/core";
  import { get } from "svelte/store";
  import { t } from "$lib/stores/i18n";

  function updateXMode(mode: AxisInputMode) {
    profile.update((p) => ({ ...p, rightStickXMode: mode }));
    markDirty();
    pushToEngine();
  }

  function updateYMode(mode: AxisInputMode) {
    profile.update((p) => ({ ...p, rightStickYMode: mode }));
    markDirty();
    pushToEngine();
  }

  // Push the profile to the running engine (debounced) so slider changes
  // and DSU toggles apply live, without saving first
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

  function updateSensX(v: number) {
    profile.update((p) => ({
      ...p,
      rightStick: { ...p.rightStick, sensitivityX: v },
    }));
    markDirty();
    pushToEngine();
  }

  function updateSensY(v: number) {
    profile.update((p) => ({
      ...p,
      rightStick: { ...p.rightStick, sensitivityY: v },
    }));
    markDirty();
    pushToEngine();
  }

  function updateSmoothing(v: number) {
    profile.update((p) => ({
      ...p,
      rightStick: { ...p.rightStick, smoothing: v },
    }));
    markDirty();
    pushToEngine();
  }

  function updateAyLock(v: number) {
    profile.update((p) => ({ ...p, gyroAyLock: v }));
    markDirty();
    pushToEngine();
  }

  function updateRecalibDelay(v: number) {
    profile.update((p) => ({ ...p, gyroRecalibDelay: v }));
    markDirty();
    pushToEngine();
  }
</script>

<!-- Gyroscope axis modes: the XUSB (Xbox 360) HID report has no motion
     channels, but the values still reach games via the DSU/Cemuhook UDP
     server, so this section is available for the Xbox 360 controller. -->
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
      {#if $profile.rightStickXMode === "gyroscope"}
        <div class="space-y-1">
          <div class="flex items-center justify-between">
            <span class="text-[11px] font-medium">{$t("rsm.gyroSensX")}</span>
            <span class="text-[11px] text-muted-foreground tabular-nums">{$profile.rightStick.sensitivityX.toFixed(2)}</span>
          </div>
          <Slider type="single" value={$profile.rightStick.sensitivityX} onValueChange={updateSensX} min={0.1} max={10} step={0.1} />
        </div>
      {/if}
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
      {#if $profile.rightStickYMode === "gyroscope"}
        <div class="space-y-1">
          <div class="flex items-center justify-between">
            <span class="text-[11px] font-medium">{$t("rsm.gyroSensY")}</span>
            <span class="text-[11px] text-muted-foreground tabular-nums">{$profile.rightStick.sensitivityY.toFixed(2)}</span>
          </div>
          <Slider type="single" value={$profile.rightStick.sensitivityY} onValueChange={updateSensY} min={0.1} max={10} step={0.1} />
        </div>
      {/if}
    </div>

    <!-- Shared gyro smoothing -->
    {#if $profile.rightStickXMode === "gyroscope" || $profile.rightStickYMode === "gyroscope"}
      <div class="space-y-1.5 border-t border-border/15 pt-2">
        <div class="flex items-center justify-between">
          <span class="text-xs font-medium">{$t("rsm.gyroSmoothing")}</span>
          <span class="text-xs text-muted-foreground tabular-nums">{Math.round($profile.rightStick.smoothing * 100)}%</span>
        </div>
        <Slider type="single" value={$profile.rightStick.smoothing} onValueChange={updateSmoothing} min={0} max={0.95} step={0.05} />
      </div>
    {/if}

    <!-- Recalibration settings -->
    {#if $profile.rightStickXMode === "gyroscope" || $profile.rightStickYMode === "gyroscope"}
      <div class="space-y-2 border-t border-border/15 pt-2">
        <div class="space-y-1">
          <div class="flex items-center justify-between">
            <span class="text-[11px] font-medium">{$t("rsm.ayLock")}</span>
            <span class="text-[11px] text-muted-foreground tabular-nums">{$profile.gyroAyLock.toFixed(2)}</span>
          </div>
          <Slider type="single" value={$profile.gyroAyLock} onValueChange={updateAyLock} min={-2.0} max={-1.0} step={0.01} />
        </div>
        <div class="space-y-1">
          <div class="flex items-center justify-between">
            <span class="text-[11px] font-medium">{$t("rsm.recalibDelay")}</span>
            <span class="text-[11px] text-muted-foreground tabular-nums">{$profile.gyroRecalibDelay.toFixed(1)}s</span>
          </div>
          <Slider type="single" value={$profile.gyroRecalibDelay} onValueChange={updateRecalibDelay} min={0} max={10} step={0.5} />
        </div>
      </div>
    {/if}

    <!-- DSU (Cemuhook) motion server -->
    <div class="space-y-1.5 pt-1 border-t border-border/15">
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
