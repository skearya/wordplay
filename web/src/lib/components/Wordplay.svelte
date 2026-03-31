<script lang="ts">
	import type { ClientMessage } from '@bindings/ClientMessage';
	import type { Context } from '$lib/context';
	import { onMount } from 'svelte';
	import { scale } from 'svelte/transition';
	import { coreEmitter } from '$lib/events';
	import { transitionState } from '$lib/stores/transition.svelte';
	import { objectAssign, unreachable } from '$lib/utils';
	import Game from './Game.svelte';
	import Lobby from './Lobby.svelte';
	import Nav from './Nav.svelte';
	import Transition from './Transition.svelte';

	let {
		ctx = $bindable(),
		sendMsg
	}: {
		ctx: Context;
		sendMsg: (message: ClientMessage) => void;
	} = $props();

	onMount(() =>
		coreEmitter.handle({
			join: ({ uuid, client }) => {
				ctx.clients[uuid] = client;
			},
			rejoin: ({ uuid }) => {
				ctx.clients[uuid]!.connected = true;
			},
			leave: ({ uuid }) => {
				ctx.clients[uuid]!.connected = false;
			},
			gameStart: ({ state }) => {
				objectAssign(transitionState, {
					kind: 'transitioning',
					update: { kind: 'game', ...state }
				});
			},
			gameEnd: ({ postGameInfo }) => {
				objectAssign(transitionState, {
					kind: 'transitioning',
					update: {
						kind: 'lobby',
						ready: [],
						timerStart: null,
						prevGame: postGameInfo
					}
				});
			}
		})
	);
</script>

<main
	in:scale={{ start: 0.9 }}
	style="background: linear-gradient(180deg, rgba(0, 0, 0, 0) 0%, rgba(246, 245, 180, 0.08) 100%), var(--color-background)"
	class="flex h-screen flex-col overflow-hidden"
>
	<Transition bind:ctx />
	<Nav bind:ctx />
	{#if ctx.state.kind === 'lobby'}
		<Lobby bind:ctx bind:state={ctx.state} {sendMsg} />
	{:else if ctx.state.kind === 'game'}
		<Game bind:ctx bind:state={ctx.state} {sendMsg} />
	{:else if ctx.state satisfies never}
		{unreachable(ctx.state)}
	{/if}
</main>
