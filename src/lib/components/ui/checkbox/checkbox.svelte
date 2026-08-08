<script lang="ts">
	import { Checkbox as CheckboxPrimitive } from "bits-ui";
	import { cn, type WithoutChildrenOrChild } from "$lib/utils.js";
	import CheckIcon from "@lucide/svelte/icons/check";

	let {
		ref = $bindable(null),
		checked = $bindable(false),
		class: className,
		...restProps
	}: WithoutChildrenOrChild<CheckboxPrimitive.RootProps> = $props();
</script>

<CheckboxPrimitive.Root
	bind:ref
	bind:checked
	data-slot="checkbox"
	class={cn(
		"peer size-5 shrink-0 rounded-[4px] border-2 border-muted-foreground/60 bg-transparent transition-colors outline-none",
		"hover:border-primary focus-visible:ring-2 focus-visible:ring-ring/50",
		"data-[state=checked]:border-primary data-[state=checked]:bg-primary data-[state=checked]:text-primary-foreground",
		"flex items-center justify-center cursor-pointer data-disabled:cursor-not-allowed data-disabled:opacity-50",
		className
	)}
	{...restProps}
>
	{#snippet children({ checked })}
		{#if checked}
			<CheckIcon class="size-3.5" strokeWidth={3.5} />
		{/if}
	{/snippet}
</CheckboxPrimitive.Root>
