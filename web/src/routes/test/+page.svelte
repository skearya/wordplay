<script lang="ts">
	import type { PageProps } from './$types';
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
					class="border-dark-green size-12 content-center rounded-full border-2 bg-black text-center"
				>
					+2
				</div>
			</div>
			<GreenSettings />
		</div>
	</nav>
	<div class="flex flex-1 items-stretch gap-4 overflow-y-hidden p-4 pt-0">
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
						style="background: linear-gradient(180deg, rgba(246, 245, 180, 0.08) 0%, rgba(243, 246, 245, 0) 100%);"
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
		<section
			class="border-green bg-green/15 relative flex flex-[70%] flex-col overflow-clip border"
		>
			<div class="pointer-events-none absolute -bottom-6 -right-6 opacity-50">
				<WordBomb class="aspect-[901/916] w-[calc(min(45vw,60vh))] mix-blend-color-dodge" />
			</div>
			<div class="bg-green absolute left-0 top-0 w-min text-nowrap rounded-br-2xl px-3 py-1">
				<p>Ready Players</p>
			</div>
			<div class="text-bright-green absolute right-3 top-3 w-min text-nowrap">
				<p>97 slots left</p>
			</div>
			<div class="flex flex-1 items-center justify-center gap-x-20 px-12">
				<div class="relative size-[304px]">
					<div
						in:fly={{ x: 48, y: 48, duration: 400, delay: 150 }}
						out:fly={{ x: 48, y: -48, duration: 400 }}
						class={[
							readyPlayers.length === 0 ? 'delay-400 opacity-100' : 'opacity-0',
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
							class="timing-function-0 absolute left-1/2 top-1/2 flex flex-col items-center gap-y-2 transition-transform duration-[400ms]"
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
						<h1 class="text-yellow font-serif text-4xl">Word Bomb</h1>
						<DownArrow />
					</div>
					<div class="bg-green h-[1px] w-full"></div>
					<div class="grid grid-cols-2 justify-between text-sm">
						<p class="text-light-green text-left font-medium">Difficulty</p>
						<p class="text-light-green text-right">Easy</p>
						<p class="text-light-green text-left font-medium">Starting Lives</p>
						<p class="text-light-green text-right">2</p>
					</div>
				</button>
			</div>
			<div class="z-10 flex gap-x-4 p-4">
				<button
					style="box-shadow: 0px 4px 4px rgba(0, 0, 0, 0.25)"
					class="border-pastel-pink bg-dark-dark-green text-pastel-pink active:bg-pastel-pink active:text-dark-dark-green flex-1 border p-4 text-xl transition-all active:translate-y-1.5"
					onclick={() => readyPlayers.push(crypto.randomUUID())}
				>
					Ready
				</button>
				<button
					style="box-shadow: 0px 4px 4px rgba(0, 0, 0, 0.25)"
					class="bg-dark-dark-green active:text-dark-dark-green flex-1 border border-[#C0E8FF] p-4 text-xl text-[#C0E8FF] transition-all active:translate-y-1.5 active:bg-[#C0E8FF]"
				>
					Start Early
				</button>
			</div>
		</section>
	</div>
</main>
