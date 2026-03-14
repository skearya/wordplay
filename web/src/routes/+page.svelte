<script lang="ts">
	import type { PageProps } from './$types';
	import { onMount } from 'svelte';
	import gridSvg from '$lib/assets/grid.svg';
	import Bomb from '$lib/icons/Bomb.svelte';
	import Github from '$lib/icons/Github.svelte';
	import Me from '$lib/icons/Me.svelte';
	import Search from '$lib/icons/Search.svelte';
	import Settings from '$lib/icons/Settings.svelte';
	import { createLetterCanvas } from '$lib/letters';

	const { data }: PageProps = $props();

	let backgroundCanvasElement: HTMLCanvasElement;
	let headerTextElement: HTMLElement;
	let contentElement: HTMLElement;

	onMount(() => {
		const animation = contentElement.animate(
			{
				translate: ['0px 50vh', '0px 0px']
			},
			{
				fill: 'forwards',
				delay: 500,
				duration: 1000,
				easing: 'cubic-bezier(0.87, 0, 0.13, 1)'
			}
		);

		headerTextElement.animate(
			{
				opacity: '100%'
			},
			{
				fill: 'forwards',
				delay: 1500,
				duration: 750,
				easing: 'ease-in'
			}
		);

		const cleanupCanvas = createLetterCanvas(backgroundCanvasElement, {
			style: 'light',
			gravity: 0.01,
			initLetters: (width, height) => {
				const letterWidth = 64;
				const letterHeight = 64;

				return ['wordplay', 'byskeary.me', 'abcdefghijkl'].reverse().flatMap((line, lineIndex) =>
					line.split('').map((letter, letterIndex) => {
						const lineWidth = line.length * (letterWidth + 4);
						const start = width / 2 - lineWidth / 2;

						return {
							letter,
							x: start + letterIndex * (letterWidth + 4) + letterWidth / 2,
							y: height - lineIndex * letterHeight - letterHeight / 2,
							angle: (Math.random() - 0.5) * 0.05
						};
					})
				);
			},
			bottomPosition: () => animation?.effect?.getComputedTiming().progress ?? 0
		});

		return () => cleanupCanvas();
	});
</script>

<header
	style={`background: linear-gradient(45deg, rgba(255,250,226,1), rgba(229,228,158,0.8)), url("data:image/svg+xml,%3Csvg viewBox='0 0 250 250' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noiseFilter'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='1.91' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noiseFilter)'/%3E%3C/svg%3E");`}
	class="absolute top-0 left-0 -z-10 h-full w-full bg-cover"
>
	<canvas bind:this={backgroundCanvasElement} class="h-full w-full"></canvas>
	<h1
		bind:this={headerTextElement}
		class="pointer-events-none absolute bottom-[50vh] left-0 p-4 font-serif text-8xl text-background opacity-0"
	>
		Wordplay
	</h1>
</header>

<section
	bind:this={contentElement}
	style={`background-image: url("${gridSvg}");`}
	class="background-scroll mt-[50vh] flex min-h-[64rem] translate-y-[50vh] items-start gap-2.5 bg-background bg-repeat p-4 inset-shadow-[0_20px_20px] inset-shadow-black"
>
	<div class="sticky top-4 w-[325px] space-y-2.5 text-background">
		<button class="w-full bg-pastel-red py-7 text-2xl font-medium">Join room</button>
		<button class="w-full bg-pastel-light-red py-7 text-2xl font-medium">Create room</button>
		<button class="w-full bg-pastel-green py-7 text-2xl font-medium">Singleplayer</button>
		<div class="flex items-center gap-x-2.5 p-2.5">
			<Me width={42} height={42} />
			<Github />
			<Settings class="ml-auto" />
		</div>
	</div>
	<div class="flex-1 space-y-2.5 p-2.5">
		<div class="flex items-center justify-between font-serif">
			<h1 class="text-2xl">Public rooms</h1>
			<div class="flex items-center justify-between gap-x-4 text-[#B0B0B0]">
				<input type="text" placeholder="Search..." class="text-xl" />
				<Search />
			</div>
		</div>
		<div class="grid grid-cols-3 gap-2.5">
			{#each { length: 12 }}
				<a href="/" class="relative border border-faded-green bg-dark-green p-2.5">
					<h1 class="mb-10 text-lg">Stupid Room Name</h1>
					<div class="flex -space-x-2">
						{#each { length: 3 }, i}
							<img
								src={`https://avatar.vercel.sh/${i}`}
								width="38px"
								height="38px"
								alt="avatar"
								class="aspect-square size-[38px] rounded-full border-2 border-background"
							/>
						{/each}
						<p
							class="flex aspect-square size-[38px] items-center justify-center rounded-full bg-background"
						>
							+21
						</p>
					</div>
					<Bomb class="absolute right-2.5 bottom-2.5" />
				</a>
			{/each}
		</div>
	</div>
</section>
