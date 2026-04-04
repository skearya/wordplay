<script lang="ts">
	import '../app.css';
	import '@fontsource-variable/inter';
	import { onMount } from 'svelte';
	import favicon from '$lib/assets/favicon.svg';

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

	{#snippet failed(_error, _reset)}
		<button onclick={() => window.location.reload()}>oops, something broke. try again?</button>
		<code>{_error}</code>
	{/snippet}
</svelte:boundary>
