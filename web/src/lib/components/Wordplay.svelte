<script lang="ts">
	import type { ClientMessage } from '@bindings/ClientMessage';
	import type { Context } from '$lib/context';
	import { onMount } from 'svelte';
	import { coreEmitter } from '$lib/events';
	import GreenSettings from '$lib/icons/GreenSettings.svelte';
	import Logo from '$lib/icons/Logo.svelte';
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

<main
	style="background: linear-gradient(180deg, rgba(0, 0, 0, 0) 0%, rgba(246, 245, 180, 0.08) 100%), var(--color-background)"
	class="flex h-screen flex-col overflow-y-hidden"
>
	{#if ctx.state.kind === 'lobby'}
		<nav class="flex items-center justify-between px-5 py-4">
			<Logo />
			<div class="flex items-center gap-x-4">
				<div class="flex -space-x-6">
					{#each Object.entries(ctx.clients) as [uuid, client]}
						<img
							src={`https://avatar.vercel.sh/${client!.username}`}
							alt={client!.username}
							title={`${client!.username} (${uuid})`}
							width="120"
							height="120"
							class="size-12 rounded-full border border-black"
						/>
					{/each}
					<div class="border-green size-12 content-center rounded-full border bg-black text-center">
						+2
					</div>
				</div>
				<GreenSettings />
			</div>
		</nav>
	{/if}
	{#if ctx.state.kind === 'lobby'}
		<Lobby {ctx} initial={ctx.state} {sendMsg} />
	{:else if ctx.state.kind === 'game'}
		<Game {ctx} initial={ctx.state} {sendMsg} />
	{:else if ctx.state satisfies never}
		{unreachable(ctx.state)}
	{/if}
</main>
