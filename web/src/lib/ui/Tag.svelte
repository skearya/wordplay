<script lang="ts">
	import type { SvelteHTMLElements } from 'svelte/elements';

	let { class: className, children, ...rest }: SvelteHTMLElements['span'] = $props();
</script>

<span
	{...rest}
	class={['absolute top-0 left-0 rounded-br-2xl px-3 py-1 text-nowrap text-background', className]}
>
	{@render children?.()}
</span>
