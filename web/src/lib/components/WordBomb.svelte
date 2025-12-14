<script lang="ts">
	import type { WordBombState } from '@bindings/WordBombState';
	import type { Props } from '$lib/context';
	import { onMount } from 'svelte';
	import { wordBombEmitter } from '$lib/events';
	import DownArrow from '$lib/icons/DownArrow.svelte';
	import GameBomb from '$lib/icons/GameBomb.svelte';
	import GameBombWire from '$lib/icons/GameBombWire.svelte';
	import HeartIcon from '$lib/icons/HeartIcon.svelte';
	import Star from '$lib/icons/Star.svelte';
	import { lerp } from '$lib/utils';

	const { ctx, initial, sendMsg }: Props<WordBombState> = $props();

	let activeOutlineContainer: HTMLElement;
	let incorrectOutlineElement: HTMLElement;
	let playersContainer: HTMLElement;
	let bombElement: HTMLElement;
	let playerInputElement = $state<HTMLInputElement>();
	let playerElements: Record<string, HTMLElement> = {};
	let arrowElements: Record<string, HTMLElement> = {};

	let wordBomb = $state(initial);

	onMount(() => {
		animateInit();

		return wordBombEmitter.handle({
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

				if (ctx.uuid === turn) {
					playerInputElement?.focus();
				}

				animateBombPass();
			},
			invalid: ({ reason }) => {
				animateIncorrect();
			},
			exploded: ({ prompt, turn }) => {
				const player = wordBomb.players[wordBomb.turn]!;

				player.lives -= 1;

				wordBomb.prompt = prompt;
				wordBomb.turn = turn;

				animateBombPass();
			}
		});
	});

	const screenPull = 0.003;

	function animateInit() {
		const playerUUID = wordBomb.turn;
		const playerBBox = playerElements[playerUUID].getBoundingClientRect();

		const playerX = playerBBox.left + playerBBox.width / 2;
		const playerY = playerBBox.top + playerBBox.height / 2;

		const playerDistanceX = playerX - window.innerWidth / 2;
		const playerDistanceY = playerY - window.innerHeight / 2;

		const containerTransformX = -playerDistanceX * screenPull * 2;
		const containerTransformY = -playerDistanceY * screenPull * 2;

		const bombTransformX = -playerDistanceX * screenPull;
		const bombTransformY = -playerDistanceY * screenPull;

		playersContainer.style.translate = `${containerTransformX}px ${containerTransformY}px`;
		bombElement.style.translate = `calc(-50% + ${bombTransformX}px) calc(-50% + ${bombTransformY}px)`;

		const newPlayerX = playerX + containerTransformX;
		const newPlayerY = playerY + containerTransformY;

		activeOutlineContainer.style.translate = `${newPlayerX}px ${newPlayerY}px`;

		activeOutlineContainer.animate(
			{
				opacity: ['0%', '100%']
			},
			{
				easing: 'cubic-bezier(0.61, 1, 0.88, 1)',
				duration: 400
			}
		);

		const arrowElementsValues = Object.values(arrowElements);

		for (let i = 0; i < arrowElementsValues.length; i++) {
			const arrowElementChild = arrowElementsValues[i].firstChild as HTMLElement;

			arrowElementChild.animate(
				{
					opacity: ['100%', '0%']
				},
				{
					easing: 'ease-out',
					duration: 400,
					delay: 150 + i * 400
				}
			);
		}
	}

	function animateBombPass() {
		const playerUUID = wordBomb.turn;
		const playerBBox = playerElements[playerUUID].getBoundingClientRect();
		const outlineBBox = activeOutlineContainer.getBoundingClientRect();

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

		const containerTransformX = -playerDistanceX * screenPull * 2;
		const containerTransformY = -playerDistanceY * screenPull * 2;

		const bombTransformX = -playerDistanceX * screenPull;
		const bombTransformY = -playerDistanceY * screenPull;

		playersContainer.style.translate = `${containerTransformX}px ${containerTransformY}px`;
		bombElement.style.translate = `calc(-50% + ${bombTransformX}px) calc(-50% + ${bombTransformY}px)`;

		const newPlayerX = playerX + containerTransformX - prevContainerTransformX;
		const newPlayerY = playerY + containerTransformY - prevContainerTransformY;

		const toX1 = lerp(outlineX, playerX, 0.1);
		const toY1 = lerp(outlineY, playerY, 0.1);

		activeOutlineContainer
			.animate(
				{
					translate: `${toX1}px ${toY1}px`,
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

				activeOutlineContainer.animate(
					{
						translate: [`${fromX2}px ${fromY2}px`, `${toX3}px ${toY3}px`],
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
	}

	function animateIncorrect() {
		for (const text of playerElements[wordBomb.turn]!.querySelectorAll('p')) {
			text.animate(
				{
					color: ['var(--color-red)', text.style.color]
				},
				{
					easing: 'cubic-bezier(0.61, 1, 0.88, 1)',
					duration: 300
				}
			);
		}

		incorrectOutlineElement.animate(
			{
				opacity: ['100%', '0%']
			},
			{
				easing: 'cubic-bezier(0.61, 1, 0.88, 1)',
				duration: 300
			}
		);
	}

	const players = $derived(Object.entries(wordBomb.players));

	const unusedLetters = $derived(
		Array.from({ length: 26 })
			.map((_, i) => String.fromCharCode(i + 'A'.charCodeAt(0)))
			.filter((c) => !wordBomb.players[ctx.uuid]!.letters.includes(c))
	);
</script>

<div class="relative flex flex-1 items-center justify-center gap-4 overflow-hidden p-4 pt-0">
	<div
		bind:this={bombElement}
		class="timing-function-0 absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 transition-transform duration-[400ms]"
	>
		<GameBomb />
		<GameBombWire class="absolute right-0 top-0 -translate-y-[4.8rem] translate-x-[4.8rem]" />
		<div
			class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-[40%] font-serif text-7xl uppercase"
		>
			{wordBomb.prompt}
		</div>
	</div>
	<div class="absolute left-0 top-0 flex max-h-full flex-col flex-wrap gap-1 p-2">
		{#each unusedLetters as letter}
			<div class="border-green size-12 content-center border text-center">{letter}</div>
		{/each}
	</div>
	<div bind:this={activeOutlineContainer} class="absolute left-0 top-0">
		<div
			style={`scale: ${ctx.uuid === wordBomb.turn ? 100 : 100 - Math.log2(players.length) * 8}%;`}
			class="rotating-border rotating absolute left-1/2 top-1/2 size-56 -translate-x-1/2 -translate-y-1/2 duration-300"
		></div>
		<div
			bind:this={incorrectOutlineElement}
			style={`scale: ${ctx.uuid === wordBomb.turn ? 100 : 100 - Math.log2(players.length) * 8}%;`}
			class="border-red rotating timing-function-0 absolute left-1/2 top-1/2 z-10 size-56 -translate-x-1/2 -translate-y-1/2 border-4 opacity-0 transition-transform duration-300"
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
				class="timing-function-0 absolute left-1/2 top-1/2 flex flex-col items-center p-2 transition-transform duration-[400ms]"
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
						<div class="absolute right-0 top-0 -translate-y-1/2 translate-x-1/2">
							<Star />
							<span
								class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-[40%] tracking-tight text-black"
							>
								x<span class="text-2xl font-semibold">10</span>
							</span>
						</div>
					{/if}
				</div>
				<p class="font-medium">{ctx.clients[uuid]!.username}</p>
				{#if ctx.uuid === uuid}
					<input
						bind:this={playerInputElement}
						type="text"
						disabled={ctx.uuid !== wordBomb.turn}
						class="border-green focus:ring-pasteborder-green focus:border-green shadow-xs mt-2.5 w-24 rounded-lg border px-2 py-1.5 text-center text-lg disabled:opacity-50"
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
					<p class="mt-0.5 text-lg">{player!.input}</p>
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
				class="timing-function-0 absolute left-1/2 top-1/2 text-transparent transition-transform duration-[400ms]"
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
