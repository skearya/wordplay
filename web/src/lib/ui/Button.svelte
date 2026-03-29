<script lang="ts">
	import type { HTMLButtonAttributes } from 'svelte/elements';

	type Props = {
		color: 'pastel-blue' | 'pastel-pink';
	};

	const { children, color, ...rest }: HTMLButtonAttributes & Props = $props();
</script>

<button
	{...rest}
	class={[
		'border bg-dark-dark-green p-4 text-xl transition-all active:translate-y-1.5 active:text-dark-dark-green disabled:pointer-events-none disabled:opacity-50',
		color === 'pastel-blue'
			? `border-pastel-blue text-pastel-blue active:bg-pastel-blue`
			: `border-pastel-pink text-pastel-pink active:bg-pastel-pink`,
		rest.class
	]}
>
	{@render children?.()}
</button>
