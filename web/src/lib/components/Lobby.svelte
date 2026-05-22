<script lang="ts">
	import type { LobbyState } from '@bindings/LobbyState';
	import type { TimerAction } from '@bindings/TimerAction';
	import type { Props } from '$lib/context';
	import { onMount } from 'svelte';
	import { flip } from 'svelte/animate';
	import { fly } from 'svelte/transition';
	import { lobbyEmitter } from '$lib/events';
	import Chat from './lobby/Chat.svelte';
	import Practice from './lobby/Practice.svelte';
	import Ready from './lobby/Ready.svelte';
	import Stats from './lobby/Stats.svelte';

	let { ctx = $bindable(), state: lobby = $bindable(), sendMsg }: Props<LobbyState> = $props();

	let panels: ('stats' | 'join' | 'pane')[] = $state(
		lobby.prevGame ? ['stats'] : ['stats', 'join', 'pane']
	);

	const handleTimer = (action: TimerAction) => {
		switch (action) {
			case 'start':
				lobby.timerStart = BigInt(Date.now());
				break;
			case 'stop':
				lobby.timerStart = null;
				break;
			case 'none':
				break;
		}
	};

	onMount(() => {
		if (lobby.prevGame) {
			setTimeout(() => (panels = ['stats', 'join', 'pane']), 2500);
		}

		return lobbyEmitter.handle({
			ready: ({ uuid, timer }) => {
				handleTimer(timer);

				lobby.ready.push(uuid);
			},
			unready: ({ uuid, timer }) => {
				handleTimer(timer);

				const index = lobby.ready.indexOf(uuid);
				if (index !== -1) lobby.ready.splice(index, 1);
			},
			practice: () => {},
			practiceResult: () => {}
		});
	});
</script>

<div
	class={[
		'no-scrollbar flex flex-1 items-stretch gap-4 overflow-x-auto p-4 pt-0',
		panels.length === 1 ? 'justify-center' : 'justify-start'
	]}
>
	{#each panels as kind (kind)}
		<div
			animate:flip={{ duration: 400 }}
			in:fly={{ delay: 400 }}
			class={[
				kind === 'stats'
					? 'min-w-[30%]'
					: kind === 'join'
						? 'min-w-[calc(70%_-_var(--spacing)_*_4)]'
						: 'grid min-w-[30%] grid-rows-2 gap-y-4'
			]}
		>
			{#if kind === 'stats'}
				<Stats />
			{:else if kind === 'join'}
				<Ready {ctx} state={lobby} {sendMsg} />
			{:else}
				<Practice {ctx} {sendMsg} />
				<Chat {ctx} {sendMsg} />
			{/if}
		</div>
	{/each}
</div>
