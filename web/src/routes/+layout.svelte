<script lang="ts">
	import '../app.css';
	// @ts-expect-error
	import '@fontsource-variable/inter';
	import { onMount } from 'svelte';
	import favicon from '$lib/assets/favicon.svg';
	import Error from '$lib/components/Error.svelte';

	let { children } = $props();

	onMount(() => {
		history.scrollRestoration = 'manual';
		window.scrollTo(0, 0);
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<svelte:boundary onerror={(error) => console.error(error)}>
	{@render children?.()}

	{#snippet failed(error, _reset)}
		<Error message={`${error}`} fatal />
	{/snippet}
</svelte:boundary>
