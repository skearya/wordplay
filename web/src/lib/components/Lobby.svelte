<script lang="ts">
	import type { LobbyState } from '@bindings/LobbyState';
	import type { TimerAction } from '@bindings/TimerAction';
	import type { Props } from '$lib/context';
	import { onMount } from 'svelte';
	import { flip } from 'svelte/animate';
	import { fly } from 'svelte/transition';
	import grid2Svg from '$lib/assets/grid2.svg';
	import { lobbyEmitter } from '$lib/events';
	import Crown from '$lib/icons/Crown.svelte';
	import DoubleRightArrow from '$lib/icons/DoubleRightArrow.svelte';
	import DownArrow from '$lib/icons/DownArrow.svelte';
	import WordBomb from '$lib/icons/WordBomb.svelte';
	import Button from '$lib/ui/Button.svelte';
	import Avatar from './Avatar.svelte';
	import Countdown from './Countdown.svelte';

	let { ctx = $bindable(), state: lobby = $bindable(), sendMsg }: Props<LobbyState> = $props();

	let panels: ('stats' | 'join')[] = $state(lobby.prevGame ? ['stats'] : ['join']);

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
			setTimeout(() => (panels = ['stats', 'join']), 2500);
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

<div class="flex flex-1 items-stretch justify-center gap-4 overflow-hidden p-4 pt-0">
	{#each panels as kind (kind)}
		<section
			animate:flip={{ duration: 400 }}
			in:fly={{ delay: 400 }}
			class={kind === 'stats'
				? 'flex max-w-md flex-[30%] flex-col overflow-y-hidden border border-pink bg-pink/15'
				: 'relative flex flex-[70%] flex-col overflow-hidden border border-green bg-green/15'}
		>
			{#if kind === 'stats'}
				<div
					style={`background: url("${grid2Svg}"), #F3ACFF;`}
					class="background-scroll p-4 pt-16 text-black"
				>
					<div class="relative mb-2.5 size-20">
						<img
							src="https://avatar.vercel.sh/s"
							alt="avatar"
							width="120"
							height="120"
							class="h-full w-full rounded-full"
						/>
						<Crown class="absolute -top-4 -right-4" />
					</div>
					<h1 class="text-xl"><b>skeary</b> won the game!</h1>
				</div>
				<div class="flex flex-1 flex-col overflow-y-hidden p-4">
					<div class="mb-4 flex items-center justify-between gap-x-8 px-8">
						<div class="text-center">
							<h1 class="font-serif text-4xl text-yellow">21</h1>
							<p>Minutes Elapsed</p>
						</div>
						<div class="w-[1px] rotate-12 self-stretch bg-pastel-pink"></div>
						<div class="text-center">
							<h1 class="font-serif text-4xl text-yellow">21</h1>
							<p>Words Used</p>
						</div>
					</div>
					<div class="flex flex-1 flex-col overflow-y-hidden">
						<div class="flex items-center -space-x-1 text-nowrap">
							{#each ['Fastest Guess', 'Longest Word', 'Word Length', 'WPM'] as text, i}
								<button
									style={`z-index: ${4 - i};`}
									class={[
										i === 0
											? 'border-pastel-pink bg-pastel-pink text-black'
											: 'border-pink bg-[#2B212C]',
										'rounded-t-xl border border-b-0 px-2.5 py-1 text-sm'
									]}
								>
									{text}
								</button>
							{/each}
						</div>
						<div
							style="scrollbar-width: none;"
							class="flex-1 divide-y overflow-y-auto border border-pastel-pink"
						>
							{#each { length: 99 }, i}
								<div class="relative flex items-center space-x-2.5 border-pastel-pink p-2.5">
									<div
										class="absolute top-0 left-0 content-center bg-pastel-pink px-1 py-0.5 text-center text-xs text-black"
									>
										<p>{i + 1}</p>
									</div>
									<img
										src="https://avatar.vercel.sh/s"
										alt="avatar"
										width="120"
										height="120"
										class="size-12 rounded-full"
									/>
									<p>skeary</p>
									<p class="ml-auto">999ms</p>
								</div>
							{/each}
						</div>
					</div>
				</div>
			{:else}
				<div class="pointer-events-none absolute -right-6 -bottom-6 opacity-50">
					<WordBomb class="aspect-[901/916] w-[calc(min(45vw,60vh))] mix-blend-color-dodge" />
				</div>
				<div class="absolute top-0 left-0 w-min rounded-br-2xl bg-green px-3 py-1 text-nowrap">
					{#if lobby.timerStart}
						<Countdown timerStart={lobby.timerStart} />
					{:else}
						<p>Ready Players</p>
					{/if}
				</div>
				<div class="absolute top-3 right-3 w-min text-nowrap text-bright-green">
					<p>{ctx.settings.size - lobby.ready.length} slots left</p>
				</div>
				<div class="flex flex-1 items-center justify-center gap-x-20 px-12">
					<div class="relative size-[304px]">
						<div
							class={[
								'absolute top-1/2 left-1/2 flex -translate-x-1/2 -translate-y-1/2 flex-col items-center gap-y-4 text-[#B1C1AE] transition-opacity',
								lobby.ready.length === 0 ? 'opacity-100 delay-400' : 'opacity-0'
							]}
						>
							<div
								class="aspect-square size-24 content-center rounded-full border border-dashed text-center opacity-80"
							>
								?
							</div>
							<p class="loading animate-pulse text-nowrap">Waiting for someone to ready</p>
						</div>
						{#each lobby.ready as uuid, i (uuid)}
							{@const angleBetween = (2 * Math.PI) / lobby.ready.length}
							{@const angle = i * angleBetween + Math.PI / 2}
							{@const dist = 80 + Math.log2(lobby.ready.length) * 20}
							{@const x = Math.cos(angle) * dist}
							{@const y = Math.sin(angle) * dist - 32}
							<div
								in:fly={{ x: 48, y: 48, duration: 400 }}
								out:fly={{ x: 48, y: -48, duration: 400 }}
								style={lobby.ready.length === 1
									? `translate: -50% -50%;`
									: `translate: calc(-50% + ${x}px) calc(-50% + ${-y}px);` +
										`scale: ${100 - Math.log2(lobby.ready.length) * 8}%;`}
								class="timing-function-0 absolute top-1/2 left-1/2 flex flex-col items-center gap-y-2 transition-transform duration-[400ms]"
							>
								<Avatar {ctx} {uuid} />
								<p>{ctx.clients[uuid]!.username}</p>
							</div>
						{/each}
					</div>
					<DoubleRightArrow />
					<button
						style="border-image: linear-gradient(to right, var(--color-green), #95C3A8) 1;"
						class="z-10 w-72 space-y-2.5 border bg-dark-dark-green/80 p-4 pt-5"
					>
						<div class="flex items-center justify-between">
							<h1 class="font-serif text-4xl text-yellow">Word Bomb</h1>
							<DownArrow />
						</div>
						<div class="h-[1px] w-full bg-green"></div>
						<div class="grid grid-cols-2 justify-between text-sm text-light-green">
							<p class="text-left font-medium">Difficulty</p>
							<p class="text-right">Easy</p>
							<p class="text-left font-medium">Starting Lives</p>
							<p class="text-right">2</p>
						</div>
					</button>
				</div>
				<div class="z-10 flex gap-x-4 p-4">
					<Button
						color="pastel-pink"
						class="flex-1"
						onclick={() => {
							sendMsg({
								kind: 'lobby',
								data: { kind: lobby.ready.includes(ctx.uuid) ? 'unready' : 'ready' }
							});
						}}
					>
						{lobby.ready.includes(ctx.uuid) ? 'Unready' : 'Ready'}
					</Button>
					<Button
						disabled={ctx.settings.owner !== ctx.uuid || lobby.timerStart === null}
						color="pastel-blue"
						class="flex-1"
						onclick={() => {
							sendMsg({ kind: 'lobby', data: { kind: 'startEarly' } });
						}}
					>
						Start Early
					</Button>
				</div>
			{/if}
		</section>
	{/each}
</div>
