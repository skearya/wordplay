<script lang="ts">
	import type { PageProps } from './$types';
	import type { RoomInfo } from '@bindings/RoomInfo';
	import type { RoomsInfoResponse } from '@bindings/RoomsInfoResponse';
	import { PUBLIC_SERVER_URL } from '$env/static/public';
	import { onMount } from 'svelte';
	import { fade, slide } from 'svelte/transition';
	import gridSvg from '$lib/assets/grid.svg';
	import Bomb from '$lib/icons/Bomb.svelte';
	import Github from '$lib/icons/Github.svelte';
	import Me from '$lib/icons/Me.svelte';
	import Search from '$lib/icons/Search.svelte';
	import Settings from '$lib/icons/Settings.svelte';
	import { createLetterCanvas, lightStyle } from '$lib/letters';
	import { animateText } from '$lib/typewriter';
	import Avatar from '$lib/ui/Avatar.svelte';
	import { unreachable } from '$lib/utils';

	const { data }: PageProps = $props();

	let backgroundCanvasElement: HTMLCanvasElement;
	let headerTextElement: HTMLElement;
	let contentElement: HTMLElement;
	let roomsMessageElement: HTMLElement | undefined = $state();

	let roomsInfo = $state<
		| { kind: 'pending' }
		| { kind: 'fulfilled'; data: RoomsInfoResponse }
		| { kind: 'rejected'; error: any }
	>({ kind: 'pending' });

	onMount(() => {
		fetchRoomsInfo().then(
			(data) => (roomsInfo = { kind: 'fulfilled', data }),
			(error) => (roomsInfo = { kind: 'rejected', error })
		);

		const animation = contentElement.animate(
			{ translate: ['0px 50vh', '0px 0px'] },
			{
				fill: 'forwards',
				delay: 500,
				duration: 1000,
				easing: 'cubic-bezier(0.87, 0, 0.13, 1)'
			}
		);

		headerTextElement.animate(
			{ opacity: '100%' },
			{
				fill: 'forwards',
				delay: 1500,
				duration: 750,
				easing: 'ease-in'
			}
		);

		const cleanupCanvas = createLetterCanvas(backgroundCanvasElement, {
			style: lightStyle,
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

		backgroundCanvasElement.animate(
			{ opacity: '100%' },
			{ fill: 'forwards', duration: 150, easing: 'ease-in' }
		);

		return () => cleanupCanvas();
	});

	async function fetchRoomsInfo(): Promise<RoomsInfoResponse> {
		return (await (await fetch(`${PUBLIC_SERVER_URL}/info`)).json()) as RoomsInfoResponse;
	}

	let roomsMessage = $derived.by(() => {
		if (roomsInfo.kind === 'pending') return 'Loading...';
		if (roomsInfo.kind === 'rejected') return 'Something went wrong, please try refreshing...?';
		if (Object.keys(roomsInfo.data.rooms).length === 0)
			return "There aren't any public rooms right now, be the first?";

		return null;
	});

	let searchQuery = $state('');

	let roomEntries = $derived.by(() => {
		if (roomsInfo.kind !== 'fulfilled') return [];

		return Object.entries(roomsInfo.data.rooms).filter(
			([name]) => name === '' || name.includes(searchQuery)
		);
	});

	$effect(() => {
		if (!roomsMessage || !roomsMessageElement) return;

		return animateText(roomsMessageElement, roomsMessage);
	});
</script>

<header
	style={`background: linear-gradient(45deg, rgba(255, 250, 226, 1), rgba(229, 228, 158, 0.8)), url("data:image/svg+xml,%3Csvg viewBox='0 0 250 250' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noiseFilter'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='1.91' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noiseFilter)'/%3E%3C/svg%3E");`}
	class="absolute top-0 left-0 -z-10 h-full w-full"
>
	<canvas bind:this={backgroundCanvasElement} class="h-full w-full opacity-0"></canvas>
	<h1
		bind:this={headerTextElement}
		class="pointer-events-none absolute bottom-[50vh] left-0 p-4 font-serif text-8xl text-foreground opacity-0 mix-blend-exclusion"
	>
		Wordplay
	</h1>
</header>

<section
	bind:this={contentElement}
	style={`background-image: url("${gridSvg}");`}
	class="background-scroll mt-[50vh] flex min-h-[50vh] translate-y-[50vh] items-start gap-2.5 bg-background bg-repeat p-4 inset-shadow-[0_20px_20px] inset-shadow-black"
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
				<input
					type="text"
					placeholder="Search..."
					class="text-xl"
					disabled={roomsInfo.kind !== 'fulfilled'}
					bind:value={searchQuery}
				/>
				<Search />
			</div>
		</div>
		{#if roomsMessage}
			<div
				bind:this={roomsMessageElement}
				transition:slide={{ duration: 200 }}
				class={[
					'h-16 content-center text-center font-serif text-xl text-black transition-colors duration-1000',
					roomsInfo.kind === 'rejected' ? 'bg-red' : 'bg-pink'
				]}
			>
				{roomsMessage}
			</div>
		{/if}
		{#if roomEntries.length !== 0}
			<div transition:fade class="relative grid grid-cols-3 gap-2.5">
				{#each roomEntries as [name, info]}
					{@render room({ name, info })}
				{/each}
			</div>
		{:else}
			<div transition:fade={{ duration: 200 }} class="relative grid grid-cols-3 gap-2.5">
				{#each { length: 15 }, i}
					{@render room({
						name: 'Your Room Here...',
						style: `animation: pulse 2s ${Math.floor(i / 3) * 300}ms cubic-bezier(0.4, 0, 0.6, 1) infinite;`
					})}
				{/each}
			</div>
		{/if}
	</div>
</section>

{#snippet room({ name, info, style }: { name: string; info?: RoomInfo; style?: string })}
	<svelte:element
		this={info ? 'a' : 'div'}
		href={info ? `/game/${name}` : null}
		class="relative flex h-32 flex-col justify-between border border-faded-green bg-background p-2.5"
		{style}
	>
		<p class="text-lg">{name}</p>
		<div class="flex -space-x-2">
			{#if info}
				{#each info.clients as client}
					<Avatar size="sm" username={client.username} avatarUrl={client.avatarUrl ?? undefined} />
				{/each}
			{:else}
				<div class="size-9.5 rounded-full border border-pastel-red"></div>
				<div class="size-9.5 rounded-full border border-pastel-green"></div>
				<div class="size-9.5 rounded-full border border-pastel-blue"></div>
			{/if}
		</div>
		{#if info === undefined || info.settings.game === 'wordBomb'}
			<Bomb class="absolute right-3 bottom-3" />
		{:else if info.settings.game === 'anagrams'}
			{unreachable(info.settings.game)}
		{:else if info.settings.game satisfies never}
			{unreachable(info.settings.game)}
		{/if}
	</svelte:element>
{/snippet}
