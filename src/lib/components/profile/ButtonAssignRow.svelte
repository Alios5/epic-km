<script lang="ts">
  import KeyAssignPopover from "$lib/components/KeyAssignPopover.svelte";
  import { profile } from "$lib/stores/profile";
  import { t, type MessageKey } from "$lib/stores/i18n";

  interface Props {
    button: string;
    label?: string;
  }

  let { button, label }: Props = $props();

  // In DS4 mode the real DualShock names (Croix, Rond, L1, Share…) win
  // over the Xbox-flavored labels passed by the side panels.
  let displayLabel = $derived.by(() => {
    if ($profile.controllerType === "ds4") {
      const key = `btn.ds4.${button}` as MessageKey;
      const name = $t(key);
      if (name !== key) return name;
    }
    return label ?? button;
  });
</script>

<div class="flex items-center justify-between gap-2 rounded-md border border-border px-2.5 py-1.5 hover:bg-muted/50 transition-colors">
  <span class="text-xs font-medium">{displayLabel}</span>
  <KeyAssignPopover {button} />
</div>
