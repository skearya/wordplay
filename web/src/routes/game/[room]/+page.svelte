<script lang="ts">
	import type { PageProps } from './$types';
	import Socket from './Socket.svelte';

	const { params }: PageProps = $props();

	let ready = $state(false);
	let username = $state('');
</script>

<svelte:boundary>
	{#if ready}
		<Socket room={params.room} {username} />
	{:else}
		<form
			onsubmit={(e) => {
				e.preventDefault();

				if (username.length > 0 && username.length <= 32) {
					ready = true;
				}
			}}
		>
			<label>Username <input bind:value={username} /></label>
			<button>Join</button>
		</form>
	{/if}

	{#snippet failed(_error, _reset)}
		<button onclick={() => window.location.reload()}>oops! try again</button>
	{/snippet}
</svelte:boundary>
