<script lang="ts">
  import { Button } from "$lib/components/ui/button/index.js";
  import { confirmState, answerConfirm } from "$lib/stores/dialogs";
  import { t } from "$lib/stores/i18n";

  function handleKeydown(e: KeyboardEvent) {
    if (!$confirmState.open) return;
    if (e.code === "Escape") {
      e.preventDefault();
      answerConfirm(false);
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if $confirmState.open}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    role="presentation"
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
    onmousedown={(e) => e.target === e.currentTarget && answerConfirm(false)}
  >
    <div role="dialog" aria-modal="true" aria-label={$confirmState.title} class="elevated-panel rounded-xl p-6 max-w-sm w-full mx-4">
      <h2 class="text-lg font-semibold mb-2">{$confirmState.title}</h2>
      <p class="text-sm text-muted-foreground mb-6">{$confirmState.message}</p>
      <div class="flex justify-end gap-2">
        <Button variant="outline" size="sm" onclick={() => answerConfirm(false)}>
          {$confirmState.cancelLabel || $t("common.cancel")}
        </Button>
        <Button
          variant={$confirmState.kind === "warning" ? "destructive" : "default"}
          size="sm"
          onclick={() => answerConfirm(true)}
        >
          {$confirmState.okLabel || $t("common.confirm")}
        </Button>
      </div>
    </div>
  </div>
{/if}
