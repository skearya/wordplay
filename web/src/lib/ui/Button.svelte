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
		`border-${color} text-${color} active:bg-${color}`
	]}
	{...rest}
>
	{@render children?.()}
</button>
