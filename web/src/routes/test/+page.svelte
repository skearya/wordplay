<script lang="ts">
	import type { PageProps } from './$types';
	import type { LobbyState } from '@bindings/LobbyState';
	import type { ServerLobby } from '@bindings/ServerLobby';
	import { getAbortSignal } from 'svelte';
	import { fly } from 'svelte/transition';
	import grid2Svg from '$lib/assets/grid2.svg';
	import Crown from '$lib/icons/Crown.svelte';
	import DoubleRightArrow from '$lib/icons/DoubleRightArrow.svelte';
	import DownArrow from '$lib/icons/DownArrow.svelte';
	import GreenSettings from '$lib/icons/GreenSettings.svelte';
	import Logo from '$lib/icons/Logo.svelte';
	import WordBomb from '$lib/icons/WordBomb.svelte';

	const { data }: PageProps = $props();

	const lobby: LobbyState = {
		ready: [],
		timerStart: null,
		prevGame: null
	};

	let unreadMessages = $state(0);
	let readyPlayers = $state<string[]>([]);

	$effect(() => {
		const intervalId = setInterval(() => {
			unreadMessages += 1;
		}, 1500);

		document.addEventListener(
			'keydown',
			(e) => {
				if (e.key === '=') {
					readyPlayers.push(crypto.randomUUID());
				} else if (e.key === '-') {
					readyPlayers.pop();
				}
			},
			{ signal: getAbortSignal() }
		);

		return () => clearInterval(intervalId);
	});
</script>

<main
	style="background: linear-gradient(180deg, rgba(0, 0, 0, 0) 0%, rgba(246, 245, 180, 0.08) 100%), var(--color-background)"
	class="flex h-screen flex-col overflow-y-hidden"
>
	<nav class="flex items-center justify-between px-5 py-4">
		<Logo />
		<div class="flex items-center gap-x-4">
			<div class="flex -space-x-6">
				{#each { length: 3 }}
					<img
						src="https://avatar.vercel.sh/s"
						alt="avatar"
						width="120"
						height="120"
						class="size-12 rounded-full border-2 border-black"
					/>
				{/each}
				<div
					class="size-12 content-center rounded-full border-2 border-dark-green bg-black text-center"
				>
					+2
				</div>
			</div>
			<GreenSettings />
		</div>
	</nav>
	<div class="flex flex-1 items-stretch gap-4 overflow-y-hidden p-4 pt-0">
		<section class="flex flex-[30%] flex-col overflow-y-clip border border-pink bg-pink/15">
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
		</section>
		<section
			class="relative flex flex-[70%] flex-col overflow-clip border border-green bg-green/15"
		>
			<div class="pointer-events-none absolute -right-6 -bottom-6 opacity-50">
				<WordBomb class="aspect-[901/916] w-[calc(min(45vw,60vh))] mix-blend-color-dodge" />
			</div>
			<div class="absolute top-0 left-0 w-min rounded-br-2xl bg-green px-3 py-1 text-nowrap">
				<p>Ready Players</p>
			</div>
			<div class="absolute top-3 right-3 w-min text-nowrap text-bright-green">
				<p>97 slots left</p>
			</div>
			<div class="flex flex-1 items-center justify-center gap-x-20 px-12">
				<div class="relative size-[304px]">
					<div
						in:fly={{ x: 48, y: 48, duration: 400, delay: 150 }}
						out:fly={{ x: 48, y: -48, duration: 400 }}
						class={[
							readyPlayers.length === 0 ? 'opacity-100 delay-400' : 'opacity-0',
							'absolute top-1/2 left-1/2 flex -translate-x-1/2 -translate-y-1/2 flex-col items-center gap-y-4 text-[#B1C1AE] transition-opacity'
						]}
					>
						<div
							class="striped aspect-square size-24 content-center rounded-full border border-dashed text-center"
						>
							?
						</div>
						<p class="loading animate-pulse text-nowrap">Waiting for someone to ready</p>
					</div>
					{#each readyPlayers as uuid, i (uuid)}
						{@const angleBetween = (2 * Math.PI) / readyPlayers.length}
						{@const angle = i * angleBetween + Math.PI / 2}
						{@const distanceFromCenter = 80 + Math.log2(readyPlayers.length) * 20}
						{@const x = Math.cos(angle) * distanceFromCenter}
						{@const y = Math.sin(angle) * distanceFromCenter - 32}
						<div
							in:fly={{ x: 48, y: 48, duration: 400, delay: 150 }}
							out:fly={{ x: 48, y: -48, duration: 400 }}
							style={readyPlayers.length === 1
								? `translate: -50% -50%;`
								: `translate: calc(-50% + ${x}px) calc(-50% + ${-y}px); scale: ${100 - Math.log2(readyPlayers.length) * 8}%;`}
							class="timing-function-0 absolute top-1/2 left-1/2 flex flex-col items-center gap-y-2 transition-transform duration-[400ms]"
						>
							<img
								src={`https://avatar.vercel.sh/${i}`}
								alt="avatar"
								width="120"
								height="120"
								class="size-24 rounded-full"
							/>
							<p>skeary</p>
						</div>
					{/each}
				</div>
				<DoubleRightArrow />
				<button
					style="border-image: linear-gradient(to right, #475D50, #95C3A8) 1;"
					class="z-10 w-72 space-y-2.5 border p-4 pt-5 backdrop-blur-sm backdrop-brightness-90 transition-all"
				>
					<div class="flex items-center justify-between">
						<h1 class="font-serif text-4xl text-yellow">Word Bomb</h1>
						<DownArrow />
					</div>
					<div class="h-[1px] w-full bg-green"></div>
					<div class="grid grid-cols-2 justify-between text-sm">
						<p class="text-left font-medium text-light-green">Difficulty</p>
						<p class="text-right text-light-green">Easy</p>
						<p class="text-left font-medium text-light-green">Starting Lives</p>
						<p class="text-right text-light-green">2</p>
					</div>
				</button>
			</div>
			<div class="z-10 flex gap-x-4 p-4">
				<button
					style="box-shadow: 0px 4px 4px rgba(0, 0, 0, 0.25)"
					class="flex-1 border border-pastel-pink bg-dark-dark-green p-4 text-xl text-pastel-pink transition-all active:translate-y-1.5 active:bg-pastel-pink active:text-dark-dark-green"
					onclick={() => readyPlayers.push(crypto.randomUUID())}
				>
					Ready
				</button>
				<button
					style="box-shadow: 0px 4px 4px rgba(0, 0, 0, 0.25)"
					class="flex-1 border border-[#C0E8FF] bg-dark-dark-green p-4 text-xl text-[#C0E8FF] transition-all active:translate-y-1.5 active:bg-[#C0E8FF] active:text-dark-dark-green"
				>
					Start Early
				</button>
			</div>
		</section>
	</div>
</main>
