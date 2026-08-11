<script lang="ts">
  import * as Popover from "$lib/components/ui/popover/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import { profile, setKeyForButton, getKeyForButton } from "$lib/stores/profile";
  import { layoutMap, labelForCode } from "$lib/keyLabels";
  import { t, locale, type MessageKey } from "$lib/stores/i18n";
  import TrashIcon from "@lucide/svelte/icons/trash-2";

  interface Props {
    button: string;
    /** Render the trigger as a full-size zone (used by the gamepad diagram). */
    zone?: boolean;
    /** Always-visible label inside the zone (for elements not drawn in the art). */
    zoneLabel?: string;
  }

  let { button, zone = false, zoneLabel }: Props = $props();

  let open = $state(false);
  let listening = $state(false);

  let currentKey = $derived(getKeyForButton($profile.keyboardToButton, button));
  let currentLabel = $derived(labelForCode(currentKey, $layoutMap, $locale));
  let friendlyName = $derived.by(() => {
    // DS4 mode: prefer the real DualShock names (Croix, L1, Share…).
    if ($profile.controllerType === "ds4") {
      const ds4Key = `btn.ds4.${button}` as MessageKey;
      const ds4Label = $t(ds4Key);
      if (ds4Label !== ds4Key) return ds4Label;
    }
    const key = `btn.${button}` as MessageKey;
    const label = $t(key);
    return label === key ? button : label;
  });

  function startListening() {
    listening = true;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!listening) return;
    e.preventDefault();
    e.stopPropagation();
    listening = false;
    setKeyForButton(button, e.code);
  }

  function handleMousedown(e: MouseEvent) {
    if (!listening) return;
    e.preventDefault();
    e.stopPropagation();
    listening = false;
    const buttonNames: Record<number, string> = {
      0: "MouseLeft",
      1: "MouseMiddle",
      2: "MouseRight",
      3: "MouseX1",
      4: "MouseX2",
    };
    const name = buttonNames[e.button] ?? `MouseButton${e.button}`;
    setKeyForButton(button, name);
  }

  function handleClear() {
    setKeyForButton(button, "");
  }
</script>

<svelte:window onkeydown={handleKeydown} onmousedown={handleMousedown} />

<Popover.Root bind:open>
  <Popover.Trigger>
    {#snippet child({ props })}
      {#if zone}
        <button
          {...props}
          class="group relative flex h-full w-full items-center justify-center transition
            {zoneLabel
              ? 'rounded-md border border-dashed border-muted-foreground/60 bg-background/60 hover:border-primary'
              : 'rounded-[inherit] hover:bg-primary/15 hover:ring-2 hover:ring-primary/70'}"
          title={friendlyName}
        >
          <span
            class="pointer-events-none select-none font-semibold
              {zoneLabel
                ? 'text-[9px] text-muted-foreground'
                : 'rounded border border-border bg-background/90 px-1 py-0.5 text-[9px] opacity-0 group-hover:opacity-100'}"
          >
            {zoneLabel ?? friendlyName}
          </span>
          {#if currentKey}
            <span class="absolute bottom-0.5 right-0.5 size-1.5 rounded-full bg-primary"></span>
          {/if}
        </button>
      {:else}
        <button
          {...props}
          class="rounded-md px-2 py-1 text-xs font-medium transition-colors hover:bg-accent hover:text-accent-foreground
          {currentKey ? 'text-foreground' : 'text-muted-foreground'}"
        >
          {currentLabel}
        </button>
      {/if}
    {/snippet}
  </Popover.Trigger>
  <Popover.Content class="w-56 p-3">
    <div class="space-y-3">
      <p class="text-sm font-medium">{friendlyName}</p>
      <p class="text-xs text-muted-foreground">
        {$t("pop.currentKey")} <span class="font-mono text-foreground">{currentKey ? currentLabel : $t("pop.none")}</span>
      </p>
      {#if listening}
        <div class="rounded-md border border-ring bg-muted px-3 py-2 text-center text-sm animate-pulse">
          {$t("pop.pressOrClick")}
        </div>
      {:else}
        <Button size="sm" class="w-full" onclick={startListening}>
          {currentKey ? $t("pop.change") : $t("pop.assign")}
        </Button>
      {/if}
      {#if currentKey}
        <Button variant="outline" size="sm" class="w-full" onclick={handleClear}>
          <TrashIcon class="size-3.5" />
          {$t("pop.clear")}
        </Button>
      {/if}
    </div>
  </Popover.Content>
</Popover.Root>
