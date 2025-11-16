<script lang="ts">
	import type { LobbyState } from '@bindings/LobbyState';
	import type { TimerAction } from '@bindings/TimerAction';
	import type { Props } from '$lib/context';
	import { onMount } from 'svelte';
	import { lobbyEmitter } from '$lib/events';
	import Countdown from './Countdown.svelte';

	const { ctx, initial, sendMsg }: Props<LobbyState> = $props();

	let state = $state(initial);

	const handleTimer = (action: TimerAction) => {
		switch (action) {
			case 'start':
				state.timerStart = BigInt(Date.now());
				break;
			case 'stop':
				state.timerStart = null;
				break;
			case 'none':
				break;
		}
	};

	onMount(() =>
		lobbyEmitter.handle({
			ready: ({ uuid, timer }) => {
				handleTimer(timer);

				state.ready.push(uuid);
			},
			unready: ({ uuid, timer }) => {
				handleTimer(timer);

				const index = state.ready.indexOf(uuid);
				if (index !== -1) state.ready.splice(index, 1);
			},
			practice: () => {},
			practiceResult: () => {}
		})
	);
</script>

<h1>lobby</h1>
<button
	onclick={() => {
		sendMsg({
			kind: 'lobby',
			data: { kind: state.ready.includes(ctx.uuid) ? 'unready' : 'ready' }
		});
	}}
>
	ready
</button>

{#each state.ready as uuid}
	<p>{ctx.clients[uuid]!.username}</p>
{/each}

{#if state.timerStart}
	<Countdown timerStart={state.timerStart} />
{/if}
