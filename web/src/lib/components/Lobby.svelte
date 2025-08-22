<script lang="ts">
	import type { ClientLobby } from '@bindings/ClientLobby';
	import type { LobbyState } from '@bindings/LobbyState';
	import type { ClientLobbyState, Context } from '$lib/context';
	import Countdown from './Countdown.svelte';

	const {
		uuid,
		settings,
		clients,
		state,
		client,
		send
	}: Context<LobbyState, ClientLobbyState, ClientLobby> = $props();
</script>

<section>
	{#each state.ready as ready (ready)}
		<h1>{clients[ready]?.username}</h1>
	{/each}

	<hr />

	<button onclick={() => send({ kind: state.ready.includes(uuid) ? 'unready' : 'ready' })}>
		{state.ready.includes(uuid) ? 'Unready' : 'Ready'}
	</button>

	{#if state.timerStart}
		<Countdown timerStart={state.timerStart} />
	{/if}
</section>
