<script lang="ts" generics="Item">
	import type { Snippet } from 'svelte';
	import type { SvelteHTMLElements } from 'svelte/elements';

	type Props = {
		items: Item[];
		radius?: string;
		offset?: boolean;
		children: Snippet<[item: Item, angle: number]>;
	};

	const {
		items,
		radius = 'min(100vw, 100vh) * 0.35',
		offset,
		children,
		...rest
	}: Props & SvelteHTMLElements['div'] = $props();
</script>

<div class="relative" {...rest}>
	{#each items as item, i}
		{@const angleBetween = (2 * Math.PI) / items.length}
		{@const angle = i * angleBetween + (offset ? angleBetween / 2 : 0)}
		<div
			style={`translate: calc(-50% + cos(${angle}rad) * (${radius})) calc(-50% - sin(${angle}rad) * (${radius}));`}
			class="absolute top-1/2 left-1/2"
		>
			{@render children(item, angle)}
		</div>
	{/each}
</div>
