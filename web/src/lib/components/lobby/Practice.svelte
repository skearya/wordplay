<script lang="ts">
	import type { Props } from '$lib/context';

	let { ctx = $bindable(), sendMsg }: Omit<Props<never>, 'state'> = $props();
</script>

<div
	class="relative flex flex-1 flex-col items-center justify-center border border-pastel-blue bg-pastel-blue/10"
>
	<div
		class="absolute top-0 left-0 w-min rounded-br-2xl bg-pastel-blue px-3 py-1 text-nowrap text-background"
	>
		<p>Practice</p>
	</div>
	<p>...</p>
</div>
