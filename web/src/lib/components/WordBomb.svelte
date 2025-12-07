<script lang="ts">
	import type { WordBombState } from '@bindings/WordBombState';
	import type { Props } from '$lib/context';
	import { getAbortSignal, onMount } from 'svelte';
	import { wordBombEmitter } from '$lib/events';
	import DownArrow from '$lib/icons/DownArrow.svelte';
	import GameBomb from '$lib/icons/GameBomb.svelte';
	import GameBombWire from '$lib/icons/GameBombWire.svelte';
	import GreenSettings from '$lib/icons/GreenSettings.svelte';
	import HeartIcon from '$lib/icons/HeartIcon.svelte';
	import Logo from '$lib/icons/Logo.svelte';
	import Star from '$lib/icons/Star.svelte';
	import { lerp } from '$lib/utils';

	const { ctx, initial, sendMsg }: Props<WordBombState> = $props();

	let activeOutlineElement: HTMLElement;
	let playersContainer: HTMLElement;
	let bombElement: HTMLElement;
	let playerElements: Record<string, HTMLElement> = {};
	let arrowElements: Record<string, HTMLElement> = {};

	let wordBomb = $state(initial);

	const players = $derived(Object.entries(wordBomb.players));

	onMount(() =>
		wordBombEmitter.handle({
			input: ({ input }) => {
				const player = wordBomb.players[wordBomb.turn]!;

				player.input = input;
			},
			valid: ({ guess, prompt, life, turn }) => {
				const player = wordBomb.players[wordBomb.turn]!;

				player.letters = [...new Set([...player.letters, ...guess])];

				if (life) {
					player.lives += 1;
					player.letters = [];
				}

				wordBomb.prompt = prompt;
				wordBomb.turn = turn;
			},
			invalid: () => {},
			exploded: ({ prompt, turn }) => {
				const player = wordBomb.players[wordBomb.turn]!;

				player.lives -= 1;

				wordBomb.prompt = prompt;
				wordBomb.turn = turn;
			}
		})
	);

	$effect(() => {
		const playerUUID = wordBomb.turn;
		const playerBBox = playerElements[playerUUID].getBoundingClientRect();
		const outlineBBox = activeOutlineElement.getBoundingClientRect();

		const playerX = playerBBox.left + playerBBox.width / 2;
		const playerY = playerBBox.top + playerBBox.height / 2;

		const outlineX = outlineBBox.left + outlineBBox.width / 2;
		const outlineY = outlineBBox.top + outlineBBox.height / 2;

		const playerDistanceX = playerX - window.innerWidth / 2;
		const playerDistanceY = playerY - window.innerHeight / 2;

		const [prevContainerTransformX, prevContainerTransformY] = (
			playersContainer.style.translate || '0px 0px'
		)
			.split(' ')
			.map((value) => value.substring(0, value.length - 2))
			.map((value) => parseFloat(value));

		const containerTransformX = -playerDistanceX * 0.006;
		const containerTransformY = -playerDistanceY * 0.006;

		const bombTransformX = -playerDistanceX * 0.003;
		const bombTransformY = -playerDistanceY * 0.003;

		playersContainer.style.translate = `${containerTransformX}px ${containerTransformY}px`;
		bombElement.style.translate = `calc(-50% + ${bombTransformX}px) calc(-50% + ${bombTransformY}px)`;

		const newPlayerX = playerX + containerTransformX - prevContainerTransformX;
		const newPlayerY = playerY + containerTransformY - prevContainerTransformY;

		const toX1 = lerp(outlineX, playerX, 0.1);
		const toY1 = lerp(outlineY, playerY, 0.1);

		activeOutlineElement
			.animate(
				{
					transform: `translateX(calc(-50% + ${toX1}px)) translateY(calc(-50% + ${toY1}px))`,
					filter: 'blur(2px)',
					opacity: '0%'
				},
				{
					fill: 'forwards',
					easing: 'cubic-bezier(0.25, 1, 0.5, 1)',
					duration: 100
				}
			)
			.finished.then(() => {
				const fromX2 = lerp(outlineX, newPlayerX, 0.9);
				const fromY2 = lerp(outlineY, newPlayerY, 0.9);

				const toX3 = newPlayerX;
				const toY3 = newPlayerY;

				activeOutlineElement.animate(
					{
						transform: [
							`translateX(calc(-50% + ${fromX2}px)) translateY(calc(-50% + ${fromY2}px))`,
							`translateX(calc(-50% + ${toX3}px)) translateY(calc(-50% + ${toY3}px))`
						],
						filter: 'blur(0px)',
						opacity: '100%'
					},
					{
						fill: 'forwards',
						easing: 'cubic-bezier(0.61, 1, 0.88, 1)',
						duration: 150
					}
				);
			});

		const arrowElement = arrowElements[playerUUID];
		const arrowElementChild = arrowElement.firstChild as HTMLElement;

		arrowElementChild.animate(
			{
				translate: `0px 32px`,
				opacity: ['100%', '0%']
			},
			{
				easing: 'ease-out',
				duration: 400
			}
		);
	});

	function wrappingMod(i: number, iMax: number) {
		return ((i % iMax) + iMax) % iMax;
	}
</script>

<!-- 
<h1>prompt: {wordBomb.prompt}</h1>

<h1>turn: {ctx.clients[wordBomb.turn]!.username}</h1>

<h1>players</h1>
{#each Object.entries(wordBomb.players) as [uuid, data]}
	<p>{ctx.clients[uuid]!.username}: {data!.input}, {data!.lives}</p>
{/each} -->

<div class="relative flex flex-1 items-center justify-center gap-4 overflow-hidden p-4 pt-0">
	<div
		bind:this={bombElement}
		class="timing-function-0 absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 transition-transform duration-[400ms]"
	>
		<GameBomb />
		<GameBombWire class="absolute top-0 right-0 translate-x-[4.8rem] -translate-y-[4.8rem]" />
		<div
			class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-[40%] font-serif text-7xl uppercase"
		>
			{wordBomb.prompt}
		</div>
	</div>
	<div class="absolute top-0 left-0 flex max-h-full flex-col flex-wrap gap-1 p-2">
		{#each { length: 26 }, i}
			<div class="size-12 content-center border border-green text-center">
				{String.fromCharCode(i + 'A'.charCodeAt(0))}
			</div>
		{/each}
	</div>
	<div bind:this={activeOutlineElement} class="absolute top-0 left-0">
		<div
			style={`scale: ${100 - Math.log2(players.length) * 8}%;`}
			class="rotating-border size-48"
		></div>
	</div>
	<div
		bind:this={playersContainer}
		class="timing-function-0 relative size-full transition-transform duration-[400ms]"
	>
		{#each players as [uuid, player], i (uuid)}
			<div
				bind:this={playerElements[uuid]}
				style={`--angle-between: 2 * pi / ${players.length};
						--angle: ${i} * var(--angle-between);
						--dist: min(100vw, 100vh) * 0.35;
						translate: calc(-50% + cos(var(--angle)) * var(--dist)) calc(-50% - sin(var(--angle)) * var(--dist));
						scale: ${100 - Math.log2(players.length) * 8}%;`}
				class="timing-function-0 absolute top-1/2 left-1/2 flex flex-col items-center p-2 transition-transform duration-[400ms]"
			>
				<div class="relative mb-2">
					<img
						src={`https://avatar.vercel.sh/${ctx.clients[uuid]!.username}`}
						alt="avatar"
						width="120"
						height="120"
						class="size-24 rounded-full"
					/>
					<div class="absolute bottom-0 left-0 flex flex-col gap-y-1.5 mix-blend-plus-lighter">
						{#each { length: player!.lives }}
							<HeartIcon />
						{/each}
					</div>
					{#if i === 0}
						<div class="absolute top-0 right-0 translate-x-1/2 -translate-y-1/2">
							<Star />
							<span
								class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-[40%] tracking-tight text-black"
							>
								x<span class="text-2xl font-semibold">10</span>
							</span>
						</div>
					{/if}
				</div>
				<p>{ctx.clients[uuid]!.username}</p>
				{#if ctx.uuid === uuid}
					<input
						type="text"
						disabled={ctx.uuid !== wordBomb.turn}
						oninput={(e) => {
							sendMsg({ kind: 'wordBomb', data: { kind: 'input', input: e.currentTarget.value } });
						}}
						onkeyup={(e) => {
							if (e.key === 'Enter') {
								sendMsg({ kind: 'wordBomb', data: { kind: 'guess', word: e.currentTarget.value } });
							}
						}}
					/>
				{:else}
					<p>{player!.input}</p>
				{/if}
			</div>
		{/each}
		{#each players as [uuid], i (uuid)}
			{@const angleBetween = (2 * Math.PI) / players.length}
			{@const angle = i * angleBetween - angleBetween / 2}
			<div
				bind:this={arrowElements[uuid]}
				style={`--angle-between: 2 * pi / ${players.length};
						--angle: ${i} * var(--angle-between) + var(--angle-between) / 2;
						--dist: min(100vw, 100vh) * 0.35;
						translate: calc(-50% + cos(var(--angle)) * var(--dist)) calc(-50% - sin(var(--angle)) * var(--dist));
						scale: ${100 - Math.log2(players.length) * 8}%;
						rotate: ${Math.PI - angle}rad;`}
				class="timing-function-0 absolute top-1/2 left-1/2 text-transparent transition-transform duration-[400ms]"
			>
				<div class="opacity-0">
					{#each { length: 3 }}
						<DownArrow />
					{/each}
				</div>
			</div>
		{/each}
	</div>
</div>
