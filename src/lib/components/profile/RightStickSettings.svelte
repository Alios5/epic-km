<script lang="ts">
  import * as Select from "$lib/components/ui/select/index.js";
  import { Slider } from "$lib/components/ui/slider/index.js";
  import { Checkbox } from "$lib/components/ui/checkbox/index.js";
  import { profile, markDirty, type AxisInputMode, type StickCurve } from "$lib/stores/profile";
  import { invoke } from "@tauri-apps/api/core";
  import { get } from "svelte/store";
  import { t } from "$lib/stores/i18n";

  let pushTimer: ReturnType<typeof setTimeout> | null = null;
  function pushToEngine() {
    if (pushTimer) clearTimeout(pushTimer);
    pushTimer = setTimeout(() => {
      invoke("reload_profile", { profile: get(profile) }).catch(() => {});
    }, 150);
  }

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

  function updateSensitivity(v: number) {
    profile.update((p) => ({ ...p, rightStick: { ...p.rightStick, sensitivity: v } }));
    markDirty();
    pushToEngine();
  }

  function updateSensitivityX(v: number) {
    profile.update((p) => ({ ...p, rightStick: { ...p.rightStick, sensitivityX: v } }));
    markDirty();
    pushToEngine();
  }

  function updateSensitivityY(v: number) {
    profile.update((p) => ({ ...p, rightStick: { ...p.rightStick, sensitivityY: v } }));
    markDirty();
    pushToEngine();
  }

  function updateDeadzone(v: number) {
    profile.update((p) => ({ ...p, rightStick: { ...p.rightStick, deadzone: v } }));
    markDirty();
    pushToEngine();
  }

  function updateSmoothing(v: number) {
    profile.update((p) => ({ ...p, rightStick: { ...p.rightStick, smoothing: v } }));
    markDirty();
    pushToEngine();
  }

  function updateRefreshInterval(v: number) {
    profile.update((p) => ({ ...p, rightStick: { ...p.rightStick, refreshInterval: v } }));
    markDirty();
    pushToEngine();
  }

  function updateCurve(v: string) {
    if (v === "linear" || v === "exponential") {
      profile.update((p) => ({ ...p, rightStick: { ...p.rightStick, curve: v as StickCurve } }));
      markDirty();
      pushToEngine();
    }
  }

  function updateInvertY(checked: boolean) {
    profile.update((p) => ({ ...p, rightStick: { ...p.rightStick, invertY: checked } }));
    markDirty();
    pushToEngine();
  }

  function updateInvertX(checked: boolean) {
    profile.update((p) => ({ ...p, rightStick: { ...p.rightStick, invertX: checked } }));
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

  let xAnalog = $derived($profile.rightStickXMode === "analog");
  let yAnalog = $derived($profile.rightStickYMode === "analog");
  let xGyro = $derived($profile.rightStickXMode === "gyroscope");
  let yGyro = $derived($profile.rightStickYMode === "gyroscope");
  let anyAnalog = $derived(xAnalog || yAnalog);
  let anyGyro = $derived(xGyro || yGyro);
  let stick = $derived($profile.rightStick);
</script>

<section class="space-y-4 px-4">
  <!-- X axis: mode selector + per-mode settings -->
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
      {xAnalog ? $t("rsm.analogHint") : $t("rsm.gyroHint")}
    </p>

    {#if xAnalog}
      <div class="space-y-1 pt-1">
        <div class="flex items-center justify-between">
          <span class="text-[11px] font-medium">{$t("stick.sensitivityX")}</span>
          <span class="text-[11px] text-muted-foreground tabular-nums">{stick.sensitivityX.toFixed(2)}</span>
        </div>
        <Slider type="single" value={stick.sensitivityX} onValueChange={updateSensitivityX} min={0.1} max={3} step={0.05} />
      </div>
      <div class="space-y-1">
        <div class="flex items-center justify-between">
          <span class="text-[11px] font-medium">{$t("stick.deadzone")}</span>
          <span class="text-[11px] text-muted-foreground tabular-nums">{stick.deadzone.toFixed(2)}</span>
        </div>
        <Slider type="single" value={stick.deadzone} onValueChange={updateDeadzone} min={0} max={0.5} step={0.01} />
      </div>
    {:else}
      <div class="space-y-1 pt-1">
        <div class="flex items-center justify-between">
          <span class="text-[11px] font-medium">{$t("rsm.gyroSensX")}</span>
          <span class="text-[11px] text-muted-foreground tabular-nums">{stick.sensitivityX.toFixed(2)}</span>
        </div>
        <Slider type="single" value={stick.sensitivityX} onValueChange={updateSensitivityX} min={0.1} max={10} step={0.1} />
      </div>
    {/if}
  </div>

  <!-- Y axis: mode selector + per-mode settings -->
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
      {yAnalog ? $t("rsm.analogHint") : $t("rsm.gyroHint")}
    </p>

    {#if yAnalog}
      <div class="space-y-1 pt-1">
        <div class="flex items-center justify-between">
          <span class="text-[11px] font-medium">{$t("stick.sensitivityY")}</span>
          <span class="text-[11px] text-muted-foreground tabular-nums">{stick.sensitivityY.toFixed(2)}</span>
        </div>
        <Slider type="single" value={stick.sensitivityY} onValueChange={updateSensitivityY} min={0.1} max={3} step={0.05} />
      </div>
    {:else}
      <div class="space-y-1 pt-1">
        <div class="flex items-center justify-between">
          <span class="text-[11px] font-medium">{$t("rsm.gyroSensY")}</span>
          <span class="text-[11px] text-muted-foreground tabular-nums">{stick.sensitivityY.toFixed(2)}</span>
        </div>
        <Slider type="single" value={stick.sensitivityY} onValueChange={updateSensitivityY} min={0.1} max={10} step={0.1} />
      </div>
    {/if}
  </div>

  <!-- Shared analog settings -->
  {#if anyAnalog}
    <div class="space-y-3 border-t border-border pt-3">
      <!-- Global sensitivity -->
      <div class="space-y-1.5">
        <div class="flex items-center justify-between">
          <span class="text-xs font-medium">{$t("stick.sensitivity")}</span>
          <span class="text-xs text-muted-foreground tabular-nums">{stick.sensitivity.toFixed(2)}</span>
        </div>
        <Slider type="single" value={stick.sensitivity} onValueChange={updateSensitivity} min={0.1} max={3} step={0.05} />
      </div>

      <!-- Curve -->
      <div class="space-y-1.5">
        <span class="text-xs font-medium">{$t("stick.curve")}</span>
        <Select.Root type="single" value={stick.curve} onValueChange={(v) => v && updateCurve(v)}>
          <Select.Trigger class="w-full h-8 text-xs">
            {stick.curve === "linear" ? $t("stick.linear") : $t("stick.exponential")}
          </Select.Trigger>
          <Select.Content>
            <Select.Item value="linear" label={$t("stick.linear")} />
            <Select.Item value="exponential" label={$t("stick.exponential")} />
          </Select.Content>
        </Select.Root>
      </div>

      <!-- Smoothing -->
      <div class="space-y-1.5">
        <div class="flex items-center justify-between">
          <span class="text-xs font-medium">{$t("stick.smoothing")}</span>
          <span class="text-xs text-muted-foreground tabular-nums">{Math.round(stick.smoothing * 100)}%</span>
        </div>
        <Slider type="single" value={stick.smoothing} onValueChange={updateSmoothing} min={0} max={0.95} step={0.05} />
      </div>

      <!-- Refresh interval -->
      <div class="space-y-1.5">
        <div class="flex items-center justify-between">
          <span class="text-xs font-medium">{$t("stick.interval")}</span>
          <span class="text-xs text-muted-foreground tabular-nums">{stick.refreshInterval}</span>
        </div>
        <Slider type="single" value={stick.refreshInterval} onValueChange={updateRefreshInterval} min={10} max={1000} step={5} />
      </div>

      <!-- Invert checkboxes -->
      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <label for="rs-invert-y" class="text-xs font-medium cursor-pointer select-none">{$t("stick.invertY")}</label>
          <Checkbox id="rs-invert-y" checked={stick.invertY} onCheckedChange={updateInvertY} />
        </div>
        <div class="flex items-center justify-between">
          <label for="rs-invert-x" class="text-xs font-medium cursor-pointer select-none">{$t("stick.invertX")}</label>
          <Checkbox id="rs-invert-x" checked={stick.invertX} onCheckedChange={updateInvertX} />
        </div>
      </div>
    </div>
  {/if}

  <!-- Shared gyro settings -->
  {#if anyGyro}
    <div class="space-y-3 border-t border-border pt-3">
      <!-- Gyro smoothing -->
      <div class="space-y-1.5">
        <div class="flex items-center justify-between">
          <span class="text-xs font-medium">{$t("rsm.gyroSmoothing")}</span>
          <span class="text-xs text-muted-foreground tabular-nums">{Math.round(stick.smoothing * 100)}%</span>
        </div>
        <Slider type="single" value={stick.smoothing} onValueChange={updateSmoothing} min={0} max={0.95} step={0.05} />
      </div>

      <!-- Anti-recalibration -->
      <div class="space-y-1.5">
        <div class="flex items-center justify-between">
          <span class="text-xs font-medium">{$t("rsm.ayLock")}</span>
          <span class="text-xs text-muted-foreground tabular-nums">{$profile.gyroAyLock.toFixed(2)}</span>
        </div>
        <Slider type="single" value={$profile.gyroAyLock} onValueChange={updateAyLock} min={-2.0} max={-1.0} step={0.01} />
      </div>

      <!-- Natural recalibration delay -->
      <div class="space-y-1.5">
        <div class="flex items-center justify-between">
          <span class="text-xs font-medium">{$t("rsm.recalibDelay")}</span>
          <span class="text-xs text-muted-foreground tabular-nums">{$profile.gyroRecalibDelay.toFixed(1)}s</span>
        </div>
        <Slider type="single" value={$profile.gyroRecalibDelay} onValueChange={updateRecalibDelay} min={0} max={10} step={0.5} />
      </div>
    </div>
  {/if}

  <!-- DSU (Cemuhook) motion server -->
  <div class="space-y-1.5 border-t border-border pt-3">
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
