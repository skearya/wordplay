<script lang="ts">
	import type { LobbyState } from '@bindings/LobbyState';
	import type { TimerAction } from '@bindings/TimerAction';
	import type { Props } from '$lib/context';
	import { onMount } from 'svelte';
	import { flip } from 'svelte/animate';
	import { fly } from 'svelte/transition';
	import Chat from '$lib/components/lobby/Chat.svelte';
	import Practice from '$lib/components/lobby/Practice.svelte';
	import Ready from '$lib/components/lobby/Ready.svelte';
	import Stats from '$lib/components/lobby/Stats.svelte';
	import { lobbyEmitter } from '$lib/events';
	import { unreachable } from '$lib/utils';

	let { ctx = $bindable(), state: lobby = $bindable(), sendMsg }: Props<LobbyState> = $props();

	let panels: ('stats' | 'join' | 'chat' | 'practice')[] = $state(
		lobby.prevGame ? ['stats'] : ['join', 'chat', 'practice']
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
			setTimeout(() => (panels = ['stats', 'join', 'chat', 'practice']), 2500);
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
			class={[kind === 'join' ? 'min-w-[calc(70%_-_var(--spacing)_*_8)]' : 'min-w-[30%]']}
		>
			{#if kind === 'stats'}
				<Stats />
			{:else if kind === 'join'}
				<Ready {ctx} state={lobby} {sendMsg} />
			{:else if kind === 'chat'}
				<Chat {ctx} {sendMsg} />
			{:else if kind === 'practice'}
				<Practice {ctx} {sendMsg} />
			{:else if kind satisfies never}
				{unreachable(kind)}
			{/if}
		</div>
	{/each}
</div>
