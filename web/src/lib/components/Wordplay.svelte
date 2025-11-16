<script lang="ts">
	import type { ClientMessage } from '@bindings/ClientMessage';
	import type { Context } from '$lib/context';
	import { onMount } from 'svelte';
	import { coreEmitter } from '$lib/events';
	import { unreachable } from '$lib/utils';
	import Game from './Game.svelte';
	import Lobby from './Lobby.svelte';

	const {
		initial,
		sendMsg
	}: {
		initial: Context;
		sendMsg: (message: ClientMessage) => void;
	} = $props();

	let ctx = $state(initial);

	onMount(() =>
		coreEmitter.handle({
			join: ({ uuid, client }) => {
				ctx.clients[uuid] = client;
			},
			leave: ({ uuid, newOwner }) => {
				if (newOwner) {
					ctx.settings.owner = newOwner;
				}

				if (ctx.state.kind === 'lobby') {
					delete ctx.clients[uuid];
				} else {
					ctx.clients[uuid]!.connected = false;
				}
			},
			gameStart: ({ rejoinToken, state }) => {
				if (rejoinToken) {
					localStorage.setItem('rejoinToken', rejoinToken);
				}

				ctx.state = { kind: 'game', ...state };
			},
			gameEnd: ({ newOwner, postGameInfo }) => {
				if (newOwner) {
					ctx.settings.owner = newOwner;
				}

				ctx.state = { kind: 'lobby', ready: [], timerStart: null, prevGame: postGameInfo };

				for (const uuid in ctx.clients) {
					if (!ctx.clients[uuid]!.connected) {
						delete ctx.clients[uuid];
					}
				}
			}
		})
	);
</script>

{#if ctx.state.kind === 'lobby'}
	<Lobby {ctx} initial={ctx.state} {sendMsg} />
{:else if ctx.state.kind === 'game'}
	<Game {ctx} initial={ctx.state} {sendMsg} />
{:else if ctx.state satisfies never}
	{unreachable(ctx.state)}
{/if}
