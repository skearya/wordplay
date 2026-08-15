<script lang="ts">
	import type { Props } from '$lib/context';
	import Tag from '$lib/ui/Tag.svelte';

	let { ctx = $bindable(), sendMsg }: Omit<Props<never>, 'state'> = $props();
</script>

<div
	class="relative flex h-full flex-col items-center justify-center border border-pastel-blue bg-pastel-blue/10"
>
	<Tag class="bg-pastel-blue">Practice</Tag>
	<p>...</p>
</div>
