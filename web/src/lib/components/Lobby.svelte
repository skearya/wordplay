<script lang="ts">
	import type { ClientLobby } from '@bindings/ClientLobby';
	import type { Context } from '@bindings/Context';
	import type { LobbyState } from '@bindings/LobbyState';
	import Countdown from './Countdown.svelte';

	const {
		context,
		send
	}: {
		context: Omit<Context, 'state'> & { state: LobbyState };
		send: (message: ClientLobby) => void;
	} = $props();
</script>

<section>
	{#each context.state.ready as ready (ready)}
		<h1>{context.clients[ready]?.username}</h1>
	{/each}

	<hr />

	<button
		onclick={() => send({ kind: context.state.ready.includes(context.uuid) ? 'unready' : 'ready' })}
	>
		{context.state.ready.includes(context.uuid) ? 'Unready' : 'Ready'}
	</button>

	{#if context.state.timerStart}
		<Countdown timerStart={context.state.timerStart} />
	{/if}
</section>
