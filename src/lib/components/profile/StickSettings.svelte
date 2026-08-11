<script lang="ts">
  import * as Select from "$lib/components/ui/select/index.js";
  import { Slider } from "$lib/components/ui/slider/index.js";
  import { Checkbox } from "$lib/components/ui/checkbox/index.js";
  import { profile, markDirty, type StickConfig, type StickCurve } from "$lib/stores/profile";
  import { invoke } from "@tauri-apps/api/core";
  import { get } from "svelte/store";
  import { t } from "$lib/stores/i18n";
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";

  interface Props {
    stickKey: "leftStick" | "rightStick";
  }

  let { stickKey }: Props = $props();

  let stick = $derived($profile[stickKey]);
  let showAxis = $state(false);

  // Push the profile to the running engine (debounced: sliders fire many
  // events while dragging) so changes apply live, without saving first
  let pushTimer: ReturnType<typeof setTimeout> | null = null;
  function pushToEngine() {
    if (pushTimer) clearTimeout(pushTimer);
    pushTimer = setTimeout(() => {
      invoke("reload_profile", { profile: get(profile) }).catch(() => {});
    }, 150);
  }

  function update(patch: Partial<StickConfig>) {
    profile.update((p) => ({
      ...p,
      [stickKey]: { ...p[stickKey], ...patch },
    }));
    markDirty();
    pushToEngine();
  }

  function updateSensitivity(v: number) { update({ sensitivity: v }); }
  function updateSensitivityX(v: number) { update({ sensitivityX: v }); }
  function updateSensitivityY(v: number) { update({ sensitivityY: v }); }
  function updateDeadzone(v: number) { update({ deadzone: v }); }
  function updateSmoothing(v: number) { update({ smoothing: v }); }
  function updateRefreshInterval(v: number) { update({ refreshInterval: v }); }
  function updateCurve(v: string) {
    if (v === "linear" || v === "exponential") update({ curve: v as StickCurve });
  }
  function updateInvertY(checked: boolean) { update({ invertY: checked }); }
  function updateInvertX(checked: boolean) { update({ invertX: checked }); }
</script>

<div class="space-y-5 px-4">
  <!-- Sensitivity -->
  <div class="space-y-2 py-1">
    <div class="flex items-center justify-between">
      <span class="text-xs font-medium">{$t("stick.sensitivity")}</span>
      <span class="text-xs text-muted-foreground tabular-nums">{stick.sensitivity.toFixed(2)}</span>
    </div>
    <Slider type="single" value={stick.sensitivity} onValueChange={updateSensitivity} min={0.1} max={3} step={0.05} />

    <!-- Per-axis sensitivity (X/Y multipliers on top of the global one) -->
    <button
      type="button"
      onclick={() => (showAxis = !showAxis)}
      class="flex items-center gap-1 text-[11px] text-muted-foreground hover:text-foreground transition-colors"
    >
      <ChevronDownIcon class="size-3 transition-transform {showAxis ? 'rotate-180' : ''}" />
      {$t("stick.perAxis")}
    </button>

    {#if showAxis}
      <div class="space-y-3 rounded-md border border-border bg-muted/40 p-2.5">
        <div class="space-y-1.5">
          <div class="flex items-center justify-between">
            <span class="text-[11px] font-medium">{$t("stick.sensitivityX")}</span>
            <span class="text-[11px] text-muted-foreground tabular-nums">{stick.sensitivityX.toFixed(2)}</span>
          </div>
          <Slider type="single" value={stick.sensitivityX} onValueChange={updateSensitivityX} min={0.1} max={3} step={0.05} />
        </div>
        <div class="space-y-1.5">
          <div class="flex items-center justify-between">
            <span class="text-[11px] font-medium">{$t("stick.sensitivityY")}</span>
            <span class="text-[11px] text-muted-foreground tabular-nums">{stick.sensitivityY.toFixed(2)}</span>
          </div>
          <Slider type="single" value={stick.sensitivityY} onValueChange={updateSensitivityY} min={0.1} max={3} step={0.05} />
        </div>
      </div>
    {/if}
  </div>

  <!-- Curve -->
  <div class="space-y-2 py-1">
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

  <!-- Deadzone -->
  <div class="space-y-2 py-1">
    <div class="flex items-center justify-between">
      <span class="text-xs font-medium">{$t("stick.deadzone")}</span>
      <span class="text-xs text-muted-foreground tabular-nums">{stick.deadzone.toFixed(2)}</span>
    </div>
    <Slider type="single" value={stick.deadzone} onValueChange={updateDeadzone} min={0} max={0.5} step={0.01} />
  </div>

  <!-- Smoothing -->
  <div class="space-y-2 py-1">
    <div class="flex items-center justify-between">
      <span class="text-xs font-medium">{$t("stick.smoothing")}</span>
      <span class="text-xs text-muted-foreground tabular-nums">{Math.round(stick.smoothing * 100)}%</span>
    </div>
    <Slider type="single" value={stick.smoothing} onValueChange={updateSmoothing} min={0} max={0.95} step={0.05} />
  </div>

  <!-- Refresh interval -->
  <div class="space-y-2 py-1">
    <div class="flex items-center justify-between">
      <span class="text-xs font-medium">{$t("stick.interval")}</span>
      <span class="text-xs text-muted-foreground tabular-nums">{stick.refreshInterval}</span>
    </div>
    <Slider type="single" value={stick.refreshInterval} onValueChange={updateRefreshInterval} min={10} max={1000} step={5} />
  </div>

  <!-- Invert checkboxes -->
  <div class="space-y-3 pt-3 border-t border-border">
    <div class="flex items-center justify-between">
      <label for="invert-y-{stickKey}" class="text-xs font-medium cursor-pointer select-none">{$t("stick.invertY")}</label>
      <Checkbox id="invert-y-{stickKey}" checked={stick.invertY} onCheckedChange={updateInvertY} />
    </div>
    <div class="flex items-center justify-between">
      <label for="invert-x-{stickKey}" class="text-xs font-medium cursor-pointer select-none">{$t("stick.invertX")}</label>
      <Checkbox id="invert-x-{stickKey}" checked={stick.invertX} onCheckedChange={updateInvertX} />
    </div>
  </div>
</div>
