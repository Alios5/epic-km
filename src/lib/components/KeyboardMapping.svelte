<script lang="ts">
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Table from "$lib/components/ui/table/index.js";
  import * as Select from "$lib/components/ui/select/index.js";
  import { profile, GAMEPAD_BUTTONS, type KeyboardMapping } from "$lib/stores/profile";
  import KeyCaptureInput from "$lib/components/KeyCaptureInput.svelte";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import TrashIcon from "@lucide/svelte/icons/trash-2";

  function addRow() {
    const newId = crypto.randomUUID();
    profile.update((p) => ({
      ...p,
      keyboardToButton: [...p.keyboardToButton, { id: newId, key: "", button: "A" }],
    }));
  }

  function removeRow(id: string) {
    profile.update((p) => ({
      ...p,
      keyboardToButton: p.keyboardToButton.filter((m) => m.id !== id),
    }));
  }

  function updateKey(id: string, key: string) {
    profile.update((p) => ({
      ...p,
      keyboardToButton: p.keyboardToButton.map((m) =>
        m.id === id ? { ...m, key } : m,
      ),
    }));
  }

  function updateButton(id: string, button: string) {
    profile.update((p) => ({
      ...p,
      keyboardToButton: p.keyboardToButton.map((m) =>
        m.id === id ? { ...m, button } : m,
      ),
    }));
  }
</script>

<section class="space-y-4">
  <div class="flex items-center justify-between">
    <h2 class="text-lg font-semibold tracking-tight">Mapping clavier → boutons</h2>
    <Button variant="outline" size="sm" onclick={addRow}>
      <PlusIcon class="size-4" />
      Ajouter une ligne
    </Button>
  </div>

  <div class="rounded-lg border border-border">
    <Table.Root>
      <Table.Header>
        <Table.Row>
          <Table.Head class="w-[40%]">Touche clavier</Table.Head>
          <Table.Head class="w-[40%]">Bouton manette</Table.Head>
          <Table.Head class="w-[10%] text-right">Actions</Table.Head>
        </Table.Row>
      </Table.Header>
      <Table.Body>
        {#each $profile.keyboardToButton as mapping (mapping.id)}
          <Table.Row>
            <Table.Cell>
              <KeyCaptureInput
                value={mapping.key}
                onchange={(key) => updateKey(mapping.id, key)}
              />
            </Table.Cell>
            <Table.Cell>
              <Select.Root
                type="single"
                value={mapping.button}
                onValueChange={(v) => v && updateButton(mapping.id, v)}
              >
                <Select.Trigger class="w-40">
                  {mapping.button}
                </Select.Trigger>
                <Select.Content>
                  {#each GAMEPAD_BUTTONS as btn}
                    <Select.Item value={btn} label={btn} />
                  {/each}
                </Select.Content>
              </Select.Root>
            </Table.Cell>
            <Table.Cell class="text-right">
              <Button
                variant="ghost"
                size="sm"
                onclick={() => removeRow(mapping.id)}
                aria-label="Supprimer"
              >
                <TrashIcon class="size-4 text-destructive" />
              </Button>
            </Table.Cell>
          </Table.Row>
        {/each}
        {#if $profile.keyboardToButton.length === 0}
          <Table.Row>
            <Table.Cell colspan={3} class="text-center text-muted-foreground py-6">
              Aucun mapping. Cliquez sur « Ajouter une ligne ».
            </Table.Cell>
          </Table.Row>
        {/if}
      </Table.Body>
    </Table.Root>
  </div>
</section>
