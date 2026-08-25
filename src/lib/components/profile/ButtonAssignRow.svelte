<script lang="ts">
  import KeyAssignPopover from "$lib/components/KeyAssignPopover.svelte";
  import { profile } from "$lib/stores/profile";
  import { t, type MessageKey } from "$lib/stores/i18n";

  interface Props {
    button: string;
    label?: string;
  }

  let { button, label }: Props = $props();

  // Button labels use the internal Xbox naming (A/B/X/Y, LB/RB, etc.).
  let displayLabel = $derived.by(() => {
    return label ?? button;
  });
</script>

<div class="flex items-center justify-between gap-2 rounded-md border border-border px-2.5 py-1.5 hover:bg-muted/50 transition-colors">
  <span class="text-xs font-medium">{displayLabel}</span>
  <KeyAssignPopover {button} />
</div>
