<script lang="ts">
	import type { ClientMessage } from '@bindings/ClientMessage';
	import type { Context } from '$lib/context';
	import { onMount } from 'svelte';
	import { scale } from 'svelte/transition';
	import { coreEmitter } from '$lib/events';
	import GreenSettings from '$lib/icons/GreenSettings.svelte';
	import Logo from '$lib/icons/Logo.svelte';
	import { unreachable } from '$lib/utils';
	import Game from './Game.svelte';
	import Lobby from './Lobby.svelte';
	import Nav from './Nav.svelte';

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

<main
	in:scale={{ start: 0.9 }}
	style="background: linear-gradient(180deg, rgba(0, 0, 0, 0) 0%, rgba(246, 245, 180, 0.08) 100%), var(--color-background)"
	class="flex h-screen flex-col overflow-hidden"
>
	<Nav {ctx} {sendMsg} />
	{#if ctx.state.kind === 'lobby'}
		<Lobby {ctx} initial={ctx.state} {sendMsg} />
	{:else if ctx.state.kind === 'game'}
		<Game {ctx} initial={ctx.state} {sendMsg} />
	{:else if ctx.state satisfies never}
		{unreachable(ctx.state)}
	{/if}
</main>
