<script lang="ts">
	import type { HTMLButtonAttributes } from 'svelte/elements';

	type Props = {
		color: 'pastel-blue' | 'pastel-pink';
	};

	let { children, color, ...rest }: HTMLButtonAttributes & Props = $props();
</script>

<button
	class={[
		'shadow-0 bg-dark-dark-green active:text-dark-dark-green  flex-1 border p-4 text-xl transition-all active:translate-y-1.5 disabled:pointer-events-none disabled:opacity-50',
		color === 'pastel-blue'
			? `border-pastel-blue text-pastel-blue active:bg-pastel-blue`
			: `border-pastel-pink text-pastel-pink active:bg-pastel-pink`
	]}
	{...rest}
>
	{@render children?.()}
</button>
