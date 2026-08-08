<script lang="ts">
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Select from "$lib/components/ui/select/index.js";
  import * as Table from "$lib/components/ui/table/index.js";
  import { profile, markDirty, type StickDirection } from "$lib/stores/profile";
  import KeyCaptureInput from "$lib/components/KeyCaptureInput.svelte";
  import { t } from "$lib/stores/i18n";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import TrashIcon from "@lucide/svelte/icons/trash-2";

  const DIRECTIONS: StickDirection[] = ["up", "down", "left", "right"];

  function addRow() {
    const newId = crypto.randomUUID();
    profile.update((p) => ({
      ...p,
      keyboardToLeftStick: [
        ...p.keyboardToLeftStick,
        { id: newId, key: "", direction: "up" },
      ],
    }));
    markDirty();
  }

  function removeRow(id: string) {
    profile.update((p) => ({
      ...p,
      keyboardToLeftStick: p.keyboardToLeftStick.filter((m) => m.id !== id),
    }));
    markDirty();
  }

  function updateKey(id: string, key: string) {
    profile.update((p) => ({
      ...p,
      keyboardToLeftStick: p.keyboardToLeftStick.map((m) =>
        m.id === id ? { ...m, key } : m,
      ),
    }));
    markDirty();
  }

  function updateDirection(id: string, direction: StickDirection) {
    profile.update((p) => ({
      ...p,
      keyboardToLeftStick: p.keyboardToLeftStick.map((m) =>
        m.id === id ? { ...m, direction } : m,
      ),
    }));
    markDirty();
  }
</script>

<section class="space-y-2">
  <div class="flex items-center justify-between">
    <span class="text-xs text-muted-foreground">{$t("lsm.title")}</span>
    <Button variant="ghost" size="sm" class="h-6 px-2" onclick={addRow}>
      <PlusIcon class="size-3" />
    </Button>
  </div>

  {#if $profile.keyboardToLeftStick.length > 0}
    <div class="space-y-1.5">
      {#each $profile.keyboardToLeftStick as mapping (mapping.id)}
        <div class="flex items-center gap-1.5">
          <KeyCaptureInput
            value={mapping.key}
            onchange={(key) => updateKey(mapping.id, key)}
          />
          <Select.Root
            type="single"
            value={mapping.direction}
            onValueChange={(v) => v && updateDirection(mapping.id, v as StickDirection)}
          >
            <Select.Trigger class="h-8 w-24 text-xs">
              {$t(`dir.${mapping.direction}`)}
            </Select.Trigger>
            <Select.Content>
              {#each DIRECTIONS as dir}
                <Select.Item value={dir} label={$t(`dir.${dir}`)} />
              {/each}
            </Select.Content>
          </Select.Root>
          <Button
            variant="ghost"
            size="sm"
            class="h-8 w-8 p-0"
            onclick={() => removeRow(mapping.id)}
            aria-label={$t("common.delete")}
          >
            <TrashIcon class="size-3 text-destructive" />
          </Button>
        </div>
      {/each}
    </div>
  {:else}
    <p class="text-xs text-muted-foreground py-1">{$t("lsm.empty")}</p>
  {/if}
</section>
