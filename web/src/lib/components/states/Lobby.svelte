<script lang="ts">
	import type { LobbyState } from '@bindings/LobbyState';
	import type { PostGameInfo } from '@bindings/PostGameInfo';
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

	type Panel =
		| { kind: 'stats'; info: PostGameInfo }
		| { kind: 'join' }
		| { kind: 'chat' }
		| { kind: 'practice' };

	let panels: Panel[] = $state(
		lobby.prevGame
			? [{ kind: 'stats', info: lobby.prevGame }]
			: [{ kind: 'join' }, { kind: 'chat' }, { kind: 'practice' }]
	);

	const handleTimer = (action: TimerAction) => {
		switch (action) {
			case 'start':
				lobby.timerStart = Date.now();
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
			setTimeout(() => panels.push({ kind: 'join' }, { kind: 'chat' }, { kind: 'practice' }), 2500);
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
	onwheel={(e) => {
		e.preventDefault();
		e.currentTarget.scrollLeft += e.deltaY;
	}}
>
	{#each panels as panel (panel.kind)}
		<div
			animate:flip={{ duration: 400 }}
			in:fly={{ delay: 400 }}
			class={[panel.kind === 'join' ? 'min-w-[calc(70%_-_var(--spacing)_*_8)]' : 'min-w-[30%]']}
		>
			{#if panel.kind === 'stats'}
				<Stats {ctx} info={panel.info} />
			{:else if panel.kind === 'join'}
				<Ready {ctx} state={lobby} {sendMsg} />
			{:else if panel.kind === 'chat'}
				<Chat {ctx} {sendMsg} />
			{:else if panel.kind === 'practice'}
				<Practice {ctx} {sendMsg} />
			{:else if panel satisfies never}
				{unreachable(panel)}
			{/if}
		</div>
	{/each}
</div>
