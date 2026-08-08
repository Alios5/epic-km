<script lang="ts">
  import KeyAssignPopover from "$lib/components/KeyAssignPopover.svelte";

  interface Props {
    button: string;
    label?: string;
  }

  let { button, label }: Props = $props();
</script>

<div class="flex items-center justify-between gap-2 rounded-md border border-border px-2.5 py-1.5 hover:bg-muted/50 transition-colors">
  <span class="text-xs font-medium">{label ?? button}</span>
  <KeyAssignPopover {button} />
</div>
