<script lang="ts">
	import type { Context } from '$lib/context';
	import type { ClientLobby } from '@bindings/ClientLobby';
	import type { LobbyState } from '@bindings/LobbyState';
	import Countdown from './Countdown.svelte';
	import { lobbyEmitter } from '$lib/events';
	import { onMount } from 'svelte';

	const {
		uuid,
		clients,
		settings,
		initialState,
		setRootState,
		sendMsg
	}: Context<LobbyState, ClientLobby> = $props();

	let state = $state(initialState);

	onMount(() =>
		lobbyEmitter.on({
			ready: (message) => state.ready.push(message.uuid),
			unready: (message) => state.ready.splice(state.ready.indexOf(message.uuid), 1),
			practice: (message) => {},
			practiceResult: (message) => {},
			gameStarted: (message) => setRootState({ kind: 'game', ...message.state })
		})
	);
</script>

<section>
	<h1>ready players</h1>
	{#each state.ready as ready (ready)}
		<h1>{clients[ready]?.username}</h1>
	{/each}

	<button onclick={() => sendMsg({ kind: state.ready.includes(uuid) ? 'unready' : 'ready' })}>
		{state.ready.includes(uuid) ? 'Unready' : 'Ready'}
	</button>

	{#if state.timerStart}
		<Countdown timerStart={state.timerStart} />
	{/if}
</section>
