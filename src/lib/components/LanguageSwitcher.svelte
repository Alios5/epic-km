<script lang="ts">
  import { locale, setLocale, type Locale } from "$lib/stores/i18n";

  const options: { value: Locale; label: string }[] = [
    { value: "fr", label: "FR" },
    { value: "en", label: "EN" },
  ];
</script>

<div class="inline-flex items-center gap-0.5 rounded-md border border-border p-0.5" role="group" aria-label="Language / Langue">
  {#each options as opt}
    <button
      type="button"
      onclick={() => setLocale(opt.value)}
      class="rounded px-2.5 py-1 text-xs font-medium transition-colors
        {$locale === opt.value
          ? 'bg-primary text-primary-foreground'
          : 'text-muted-foreground hover:text-foreground'}"
    >
      {opt.label}
    </button>
  {/each}
</div>
