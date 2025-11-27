<script lang="ts">
	import type { LobbyState } from '@bindings/LobbyState';
	import type { TimerAction } from '@bindings/TimerAction';
	import type { Props } from '$lib/context';
	import { onMount } from 'svelte';
	import { fly } from 'svelte/transition';
	import grid2Svg from '$lib/assets/grid2.svg';
	import { lobbyEmitter } from '$lib/events';
	import Crown from '$lib/icons/Crown.svelte';
	import DoubleRightArrow from '$lib/icons/DoubleRightArrow.svelte';
	import DownArrow from '$lib/icons/DownArrow.svelte';
	import WordBomb from '$lib/icons/WordBomb.svelte';

	const { ctx, initial, sendMsg }: Props<LobbyState> = $props();

	let lobby = $state(initial);

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

	onMount(() =>
		lobbyEmitter.handle({
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
		})
	);
</script>

<!-- 
<h1>lobby</h1>
<button
	onclick={() => {
		sendMsg({
			kind: 'lobby',
			data: { kind: lobby.ready.includes(ctx.uuid) ? 'unready' : 'ready' }
		});
	}}
>
	ready
</button>

{#each lobby.ready as uuid}
	<p>{ctx.clients[uuid]!.username}</p>
{/each}

{#if lobby.timerStart}
	<Countdown timerStart={lobby.timerStart} />
{/if} -->

<div class="flex flex-1 items-stretch gap-4 overflow-y-hidden p-4 pt-0">
	{#if lobby.prevGame}
		<section class="border-pink bg-pink/15 flex flex-[30%] flex-col overflow-y-clip border">
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
					<Crown class="absolute -right-4 -top-4" />
				</div>
				<h1 class="text-xl"><b>skeary</b> won the game!</h1>
			</div>
			<div class="flex flex-1 flex-col overflow-y-hidden p-4">
				<div class="mb-4 flex items-center justify-between gap-x-8 px-8">
					<div class="text-center">
						<h1 class="text-yellow font-serif text-4xl">21</h1>
						<p>Minutes Elapsed</p>
					</div>
					<div class="bg-pastel-pink w-[1px] rotate-12 self-stretch"></div>
					<div class="text-center">
						<h1 class="text-yellow font-serif text-4xl">21</h1>
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
						class="border-pastel-pink flex-1 divide-y overflow-y-auto border"
					>
						{#each { length: 99 }, i}
							<div class="border-pastel-pink relative flex items-center space-x-2.5 p-2.5">
								<div
									class="bg-pastel-pink absolute left-0 top-0 content-center px-1 py-0.5 text-center text-xs text-black"
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
		</section>
	{/if}
	<section class="border-green bg-green/15 relative flex flex-[70%] flex-col overflow-clip border">
		<div class="pointer-events-none absolute -bottom-6 -right-6 opacity-50">
			<WordBomb class="aspect-[901/916] w-[calc(min(45vw,60vh))] mix-blend-color-dodge" />
		</div>
		<div class="bg-green absolute left-0 top-0 w-min text-nowrap rounded-br-2xl px-3 py-1">
			<p>Ready Players</p>
		</div>
		<div class="text-bright-green absolute right-3 top-3 w-min text-nowrap">
			<p>{ctx.settings.size - lobby.ready.length} slots left</p>
		</div>
		<div class="flex flex-1 items-center justify-center gap-x-20 px-12">
			<div class="relative size-[304px]">
				<div
					in:fly={{ x: 48, y: 48, duration: 400, delay: 150 }}
					out:fly={{ x: 48, y: -48, duration: 400 }}
					class={[
						lobby.ready.length === 0 ? 'delay-400 opacity-100' : 'opacity-0',
						'absolute left-1/2 top-1/2 flex -translate-x-1/2 -translate-y-1/2 flex-col items-center gap-y-4 text-[#B1C1AE] transition-opacity'
					]}
				>
					<div
						class="striped aspect-square size-24 content-center rounded-full border border-dashed text-center"
					>
						?
					</div>
					<p class="loading animate-pulse text-nowrap">Waiting for someone to ready</p>
				</div>
				{#each lobby.ready as uuid, i (uuid)}
					{@const angleBetween = (2 * Math.PI) / lobby.ready.length}
					{@const angle = i * angleBetween + Math.PI / 2}
					{@const distanceFromCenter = 80 + Math.log2(lobby.ready.length) * 20}
					{@const x = Math.cos(angle) * distanceFromCenter}
					{@const y = Math.sin(angle) * distanceFromCenter - 32}
					<div
						in:fly={{ x: 48, y: 48, duration: 400, delay: 150 }}
						out:fly={{ x: 48, y: -48, duration: 400 }}
						style={lobby.ready.length === 1
							? `translate: -50% -50%;`
							: `translate: calc(-50% + ${x}px) calc(-50% + ${-y}px); scale: ${100 - Math.log2(lobby.ready.length) * 8}%;`}
						class="timing-function-0 absolute left-1/2 top-1/2 flex flex-col items-center gap-y-2 transition-transform duration-[400ms]"
					>
						<img
							src={`https://avatar.vercel.sh/${ctx.clients[uuid]!.username}`}
							alt="avatar"
							width="120"
							height="120"
							class="size-24 rounded-full"
						/>
						<p>{ctx.clients[uuid]!.username}</p>
					</div>
				{/each}
			</div>
			<DoubleRightArrow />
			<button
				style="border-image: linear-gradient(to right, #475D50, #95C3A8) 1;"
				class="z-10 w-72 space-y-2.5 border p-4 pt-5 backdrop-blur-sm backdrop-brightness-90 transition-all"
			>
				<div class="flex items-center justify-between">
					<h1 class="text-yellow font-serif text-4xl">Word Bomb</h1>
					<DownArrow />
				</div>
				<div class="bg-green h-[1px] w-full"></div>
				<div class="text-light-green grid grid-cols-2 justify-between text-sm">
					<p class="text-left font-medium">Difficulty</p>
					<p class="text-right">Easy</p>
					<p class="text-left font-medium">Starting Lives</p>
					<p class="text-right">2</p>
				</div>
			</button>
		</div>
		<div class="z-10 flex gap-x-4 p-4">
			<button
				class="shadow-0 border-pastel-pink bg-dark-dark-green text-pastel-pink active:bg-pastel-pink active:text-dark-dark-green flex-1 border p-4 text-xl transition-all active:translate-y-1.5"
				onclick={() => {
					sendMsg({
						kind: 'lobby',
						data: { kind: lobby.ready.includes(ctx.uuid) ? 'unready' : 'ready' }
					});
				}}
			>
				{lobby.ready.includes(ctx.uuid) ? 'Unready' : 'Ready'}
			</button>
			<button
				disabled={ctx.settings.owner !== ctx.uuid || lobby.timerStart === null}
				class="shadow-0 bg-dark-dark-green active:text-dark-dark-green flex-1 border border-[#C0E8FF] p-4 text-xl text-[#C0E8FF] transition-all active:translate-y-1.5 active:bg-[#C0E8FF] disabled:pointer-events-none disabled:opacity-50"
				onclick={() => {
					sendMsg({ kind: 'lobby', data: { kind: 'startEarly' } });
				}}
			>
				Start Early
			</button>
		</div>
	</section>
</div>
