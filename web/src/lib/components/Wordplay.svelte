<script lang="ts">
	import type { LobbyState } from '@bindings/LobbyState';
	import type { ServerGame } from '@bindings/ServerGame';
	import type { ServerGeneral } from '@bindings/ServerGeneral';
	import type { ServerLobby } from '@bindings/ServerLobby';
	import type { TimerAction } from '@bindings/TimerAction';
	import type { ClientState, Context } from '$lib/context';
	import { serverMessageEmitter } from '$lib/events';
	import { unreachable } from '$lib/utils';
	import Lobby from './Lobby.svelte';

	const props: Context = $props();

	// Kept in sync with the server.
	let context = $state({
		uuid: props.uuid,
		settings: props.settings,
		clients: props.clients,
		state: props.state,
		send: props.send
	});

	// Not kept in sync.
	let client = $state<ClientState>(props.client);

	$effect(() =>
		serverMessageEmitter.on((message) => {
			switch (message.kind) {
				case 'info':
					throw new Error('Info message sent twice?');
				case 'join':
					context.clients[message.data.uuid] = message.data.client;
					break;
				case 'leave':
					if (message.data.newOwner) {
						context.settings.owner = message.data.newOwner;
					}

					delete context.clients[message.data.uuid];
					break;
				case 'general':
					generalMessage({ ...context, client }, message.data);
					break;
				case 'lobby':
					lobbyMessage({ ...context, client }, message.data);
					break;
				case 'game':
					gameMessage({ ...context, client }, message.data);
					break;
			}
		})
	);

	function generalMessage({ settings, client }: Context, message: ServerGeneral) {
		switch (message.kind) {
			case 'pong':
				client.ping = Date.now() - (message.timestamp as any);
				break;
			case 'chat':
				client.chatMessages.push(message);
				break;
			case 'settings':
				Object.assign(settings, message);
				break;
			case 'error':
				alert(JSON.stringify(message));
				break;
		}
	}

	function lobbyMessage({ state, client }: Context, message: ServerLobby) {
		if (state.kind !== 'lobby') return;

		const timerAction = (state: LobbyState, timer: TimerAction) => {
			switch (timer) {
				case 'start':
					state.timerStart = Date.now() as any;
					break;
				case 'stop':
					state.timerStart = null;
					break;
				case 'none':
					break;
			}
		};

		switch (message.kind) {
			case 'ready':
				state.ready.push(message.uuid);
				timerAction(state, message.timer);
				break;
			case 'unready':
				state.ready = state.ready.filter((client) => client !== message.uuid);
				timerAction(state, message.timer);
				break;
			case 'practice':
				break;
			case 'practiceResult':
				break;
			case 'gameStarted':
				if (message.rejoinToken) {
					localStorage.setItem(`rejoinToken`, message.rejoinToken);
				}

				Object.assign(state, { kind: 'game', ...message.state });
				client.state = { kind: 'game' };
				break;
		}
	}

	function gameMessage({ settings, state }: Context, message: ServerGame) {
		if (state.kind !== 'game') return;

		switch (message.kind) {
			case 'wordBomb':
				break;
			case 'anagrams':
				break;
			case 'endRequest':
				state.requestingEnd.push(message.data.uuid);
				break;
			case 'ended':
				if (message.data.newOwner) {
					settings.owner = message.data.newOwner;
				}

				Object.assign(state, {
					kind: 'lobby',
					ready: [],
					timerStart: null,
					prevGame: message.data.postGameInfo
				});
				client.state = { kind: 'lobby', practicePrompts: [] };
				break;
		}
	}

	$effect(() => {
		const id = setInterval(
			() => context.send({ kind: 'general', data: { kind: 'ping', timestamp: Date.now() as any } }),
			5000
		);

		return () => clearInterval(id);
	});
</script>

<section>
	<textarea
		class="h-64"
		onkeydown={(e) => {
			if (e.key === 'Enter') {
				context.send({
					kind: 'general',
					data: { kind: 'settings', ...JSON.parse(e.currentTarget.value) }
				});
			}
		}}>{JSON.stringify(context.settings, null, 2)}</textarea
	>
	{#each Object.entries(context.clients) as data}
		<pre>{JSON.stringify(data)}</pre>
	{/each}

	<hr />

	<h1>ping: {client.ping}</h1>
	{#each client.chatMessages as { content, author }}
		<h1>{author}: {content}</h1>
	{/each}
	<label for="message">send message</label>
	<input
		type="text"
		id="message"
		onkeydown={(e) => {
			if (e.key === 'Enter') {
				e.preventDefault();
				context.send({ kind: 'general', data: { kind: 'chat', content: e.currentTarget.value } });
			}
		}}
	/>

	<hr />

	{#if context.state.kind === 'lobby' && client.state.kind === 'lobby'}
		<Lobby
			{...{
				...context,
				state: context.state,
				client: client.state,
				send: (data) => context.send({ kind: 'lobby', data })
			}}
		/>
	{:else if context.state.kind === 'game'}
		<pre>(game)</pre>
	{:else}
		{unreachable(context.state)}
	{/if}
</section>
