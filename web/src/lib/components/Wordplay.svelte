<script lang="ts">
	import type { ClientMessage } from '@bindings/ClientMessage';
	import type { ServerGeneral } from '@bindings/ServerGeneral';
	import { unreachable, type Variant } from '$lib/utils';
	import { gameEmitter, generalEmitter, lobbyEmitter, rootEmitter } from '$lib/events';
	import { onMount } from 'svelte';
	import Lobby from './Lobby.svelte';

	const {
		info,
		sendMsg
	}: { info: Variant<ServerGeneral, 'info'>; sendMsg: (message: ClientMessage) => void } = $props();

	const uuid = info.uuid;
	let settings = $state(info.settings);
	let clients = $state(info.clients);
	// Can't name this variable "state"
	let sstate = $state(info.state);

	onMount(() =>
		rootEmitter.on({
			general: (message) => {
				switch (message.data.kind) {
					case 'info':
						unreachable('Info message got sent twice');
						break;
					case 'join':
						clients[message.data.uuid] = message.data.client;
						break;
					case 'leave':
						if (message.data.newOwner) {
							settings.owner = message.data.newOwner;
						}

						delete clients[message.data.kind];
						break;
					case 'pong':
						break;
					case 'chat':
						break;
					case 'settings':
						settings = message.data;
						break;
					case 'error':
						alert(message.data.message);
						break;
					default:
						message.data satisfies never;
				}

				generalEmitter.emit(message.data);
			},
			lobby: (message) => lobbyEmitter.emit(message.data),
			game: (message) => gameEmitter.emit(message.data)
		})
	);
</script>

{#if sstate.kind === 'lobby'}
	<Lobby
		{uuid}
		{clients}
		{settings}
		initialState={sstate}
		setRootState={(state) => (sstate = state)}
		sendMsg={(message) => sendMsg({ kind: 'lobby', data: message })}
	/>
{:else if sstate.kind === 'game'}
	<h1>game</h1>
{:else if sstate satisfies never}
	{unreachable(sstate)}
{/if}
