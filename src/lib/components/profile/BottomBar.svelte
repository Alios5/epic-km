<script lang="ts">
  import { Slider } from "$lib/components/ui/slider/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Checkbox } from "$lib/components/ui/checkbox/index.js";
  import * as Select from "$lib/components/ui/select/index.js";
  import { profile, markDirty, type ControllerType } from "$lib/stores/profile";
  import KeyCaptureInput from "$lib/components/KeyCaptureInput.svelte";
  import { t } from "$lib/stores/i18n";
  import { invoke } from "@tauri-apps/api/core";
  import { get } from "svelte/store";
  import SaveIcon from "@lucide/svelte/icons/save";

  interface Props {
    onSave: () => void;
  }

  let { onSave }: Props = $props();

  // Push the current profile to the running engine (debounced) so the
  // capture hotkey and trigger threshold apply immediately, without saving.
  let pushTimer: ReturnType<typeof setTimeout> | null = null;
  function pushToEngine() {
    if (pushTimer) clearTimeout(pushTimer);
    pushTimer = setTimeout(() => {
      invoke("reload_profile", { profile: get(profile) }).catch(() => {});
    }, 150);
  }

  function updateThreshold(v: number) {
    markDirty();
    profile.update((p) => ({ ...p, triggerThreshold: v }));
    pushToEngine();
  }

  function updateToggleKey(key: string) {
    markDirty();
    profile.update((p) => ({ ...p, captureToggleKey: key }));
    pushToEngine();
  }

  function updateHideCursor(v: boolean) {
    markDirty();
    profile.update((p) => ({ ...p, hideCursor: v }));
    pushToEngine();
  }

  function updateControllerType(v: string) {
    if (v !== "xbox360" && v !== "ds4") return;
    markDirty();
    profile.update((p) => ({ ...p, controllerType: v as ControllerType }));
    pushToEngine();
  }
</script>

<footer class="flex items-center gap-4 px-4 py-2.5 border-t border-border bg-card">
  <span class="text-xs font-medium whitespace-nowrap">{$t("bottombar.triggerThreshold")}</span>
  <Slider
    type="single"
    value={$profile.triggerThreshold}
    onValueChange={updateThreshold}
    min={0}
    max={1}
    step={0.05}
    class="flex-1 max-w-xs"
  />
  <span class="text-xs text-muted-foreground tabular-nums w-10">
    {$profile.triggerThreshold.toFixed(2)}
  </span>

  <div class="h-5 w-px bg-border"></div>

  <div class="flex items-center gap-2">
    <span class="text-xs font-medium whitespace-nowrap">{$t("bottombar.captureKey")}</span>
    <KeyCaptureInput
      value={$profile.captureToggleKey}
      onchange={updateToggleKey}
    />
  </div>

  <div class="h-5 w-px bg-border"></div>

  <div class="flex items-center gap-2">
    <Checkbox
      id="hide-cursor"
      checked={$profile.hideCursor}
      onCheckedChange={(v) => updateHideCursor(v === true)}
      class="size-4"
    />
    <label
      for="hide-cursor"
      class="text-xs font-medium whitespace-nowrap cursor-pointer select-none"
      >{$t("bottombar.hideCursor")}</label
    >
  </div>

  <div class="h-5 w-px bg-border"></div>

  <div class="flex items-center gap-2">
    <span class="text-xs font-medium whitespace-nowrap">{$t("bottombar.controller")}</span>
    <Select.Root
      type="single"
      value={$profile.controllerType}
      onValueChange={(v) => v && updateControllerType(v)}
    >
      <Select.Trigger class="h-8 w-40 text-xs">
        {$profile.controllerType === "ds4" ? $t("controller.ds4") : $t("controller.xbox360")}
      </Select.Trigger>
      <Select.Content>
        <Select.Item value="xbox360" label={$t("controller.xbox360")} />
        <Select.Item value="ds4" label={$t("controller.ds4")} />
      </Select.Content>
    </Select.Root>
  </div>

  <div class="ml-auto">
    <Button size="sm" onclick={onSave}>
      <SaveIcon class="size-4" />
      {$t("common.save")}
    </Button>
  </div>
</footer>
