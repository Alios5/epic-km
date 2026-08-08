<script lang="ts">
  import * as Select from "$lib/components/ui/select/index.js";
  import { Slider } from "$lib/components/ui/slider/index.js";
  import { Switch } from "$lib/components/ui/switch/index.js";
  import { profile, type StickCurve } from "$lib/stores/profile";

  function updateSensitivity(v: number) {
    profile.update((p) => ({ ...p, rightStick: { ...p.rightStick, sensitivity: v } }));
  }

  function updateDeadzone(v: number) {
    profile.update((p) => ({ ...p, rightStick: { ...p.rightStick, deadzone: v } }));
  }

  function updateCurve(v: string) {
    if (v === "linear" || v === "exponential") {
      profile.update((p) => ({ ...p, rightStick: { ...p.rightStick, curve: v as StickCurve } }));
    }
  }

  function updateInvertY(checked: boolean) {
    profile.update((p) => ({ ...p, rightStick: { ...p.rightStick, invertY: checked } }));
  }
</script>

<section class="space-y-4">
  <h2 class="text-lg font-semibold tracking-tight">Joystick droit (souris)</h2>

  <div class="rounded-lg border border-border p-5 space-y-6">
    <!-- Sensitivity -->
    <div class="space-y-2">
      <div class="flex items-center justify-between">
        <span class="text-sm font-medium">Sensibilité</span>
        <span class="text-sm text-muted-foreground tabular-nums">
          {$profile.rightStick.sensitivity.toFixed(2)}
        </span>
      </div>
      <Slider
        type="single"
        value={$profile.rightStick.sensitivity}
        onValueChange={updateSensitivity}
        min={0.1}
        max={3}
        step={0.05}
      />
    </div>

    <!-- Curve -->
    <div class="space-y-2">
      <span class="text-sm font-medium">Courbe</span>
      <Select.Root
        type="single"
        value={$profile.rightStick.curve}
        onValueChange={(v) => v && updateCurve(v)}
      >
        <Select.Trigger class="w-48">
          {$profile.rightStick.curve === "linear" ? "Linéaire" : "Exponentielle"}
        </Select.Trigger>
        <Select.Content>
          <Select.Item value="linear" label="Linéaire" />
          <Select.Item value="exponential" label="Exponentielle" />
        </Select.Content>
      </Select.Root>
    </div>

    <!-- Deadzone -->
    <div class="space-y-2">
      <div class="flex items-center justify-between">
        <span class="text-sm font-medium">Zone morte (deadzone)</span>
        <span class="text-sm text-muted-foreground tabular-nums">
          {$profile.rightStick.deadzone.toFixed(2)}
        </span>
      </div>
      <Slider
        type="single"
        value={$profile.rightStick.deadzone}
        onValueChange={updateDeadzone}
        min={0}
        max={0.5}
        step={0.01}
      />
    </div>

    <!-- Invert Y -->
    <div class="flex items-center justify-between">
      <span class="text-sm font-medium">Inverser l'axe Y</span>
      <Switch
        checked={$profile.rightStick.invertY}
        onCheckedChange={updateInvertY}
      />
    </div>
  </div>
</section>
