<script lang="ts">
  import * as Select from "$lib/components/ui/select/index.js";
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
</script>

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
</section>
