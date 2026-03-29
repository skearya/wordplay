<script lang="ts">
	import '../app.css';
	import '@fontsource-variable/inter';
	import favicon from '$lib/assets/favicon.svg';

	let { children } = $props();
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
