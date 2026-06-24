<script lang="ts">
	import type { WordBombState } from '@bindings/WordBombState';
	import type { Props } from '$lib/context';
	import type { AnimationConfig } from 'svelte/animate';
	import type { TransitionConfig } from 'svelte/transition';
	import { onMount } from 'svelte';
	import { cubicOut } from 'svelte/easing';
	import Avatar from '$lib/components/Avatar.svelte';
	import { wordBombEmitter } from '$lib/events';
	import { explode } from '$lib/explode';
	import GameBomb from '$lib/icons/GameBomb.svelte';
	import GameBombWire from '$lib/icons/GameBombWire.svelte';
	import HeartIcon from '$lib/icons/HeartIcon.svelte';
	import { debounce, lerp } from '$lib/utils';

	let {
		ctx = $bindable(),
		state: wordBomb = $bindable(),
		sendMsg
	}: Props<WordBombState> = $props();

	let activeOutlineContainer: HTMLElement;
	let incorrectOutlineElement: HTMLElement;
	let playersContainer: HTMLElement;
	let bombElement: HTMLElement;
	let playerInputElement = $state<HTMLInputElement>();
	let playerElements: Record<string, HTMLElement> = $state({});

	let windowWidth = $state(0);
	let windowHeight = $state(0);

	onMount(() => {
		animateTurnChange({ kind: 'first-run' });

		return wordBombEmitter.handle({
			input: ({ input }) => {
				const player = wordBomb.players[wordBomb.turn]!;

				player.input = input;
			},
			valid: ({ guess, prompt, life, turn }) => {
				const prevTurn = wordBomb.turn;
				const player = wordBomb.players[wordBomb.turn]!;

				player.letters = [...new Set([...player.letters, ...guess])];

				if (life) {
					player.lives += 1;
					player.letters = [];
				}

				wordBomb.prompt = prompt;
				wordBomb.turn = turn;

				life ? animateTurnChange({ kind: 'gained-life', on: prevTurn }) : animateTurnChange();
			},
			invalid: () => {
				animateIncorrect();
			},
			exploded: ({ prompt, turn }) => {
				const prevTurn = wordBomb.turn;
				const player = wordBomb.players[wordBomb.turn]!;

				player.lives -= 1;

				wordBomb.prompt = prompt;
				wordBomb.turn = turn;

				animateTurnChange({ kind: 'exploded', on: prevTurn });
			}
		});
	});

	$effect(() => {
		if (ctx.uuid === wordBomb.turn) {
			if (playerInputElement) {
				playerInputElement.value = '';
				playerInputElement.focus();
				playerInputLength = 0;
			}
		}
	});

	let playerInput = $state('');
	let playerInputLength = $derived(playerInput.length);

	const players = $derived(
		Object.entries(wordBomb.players).filter(([_uuid, player]) => player!.lives > 0)
	);

	const playerScale = $derived(`${100 - Math.log2(players.length) * 8}%`);

	const activeOutlineScale = $derived(
		ctx.uuid === wordBomb.turn
			? 100 + playerInputLength * 2
			: 100 - Math.log2(players.length + 1) * 8
	);

	const unusedLetters = $derived(
		[...'abcdefghijklmnopqrstuvwxyz'].filter(
			(c) => !wordBomb.players[ctx.uuid]?.letters.includes(c)
		)
	);

	const screenPull = 0.003;

	async function animateTurnChange(
		opt?:
			| { kind: 'first-run' }
			| { kind: 'exploded'; on: string }
			| { kind: 'gained-life'; on: string }
	) {
		const playerUUID = wordBomb.turn;
		const playerIndex = players.findIndex(([uuid]) => uuid === playerUUID);

		const playerBBox = getPlayerFinalPosition(playerIndex, players.length);
		const outlineBBox = activeOutlineContainer.getBoundingClientRect();

		const playerX = playerBBox.x;
		const playerY = playerBBox.y;

		const outlineX = outlineBBox.left + outlineBBox.width / 2;
		const outlineY = outlineBBox.top + outlineBBox.height / 2;

		const playerDistanceX = playerX - windowWidth / 2;
		const playerDistanceY = playerY - windowHeight / 2;

		const containerShiftX = -playerDistanceX * screenPull * 2;
		const containerShiftY = -playerDistanceY * screenPull * 2;

		const bombShiftX = -playerDistanceX * screenPull;
		const bombShiftY = -playerDistanceY * screenPull;

		playersContainer.style.translate = `${containerShiftX}px ${containerShiftY}px`;
		bombElement.style.translate = `calc(-50% + ${bombShiftX}px) calc(-50% + ${bombShiftY}px)`;

		const newPlayerX = playerX + containerShiftX;
		const newPlayerY = playerY + containerShiftY;

		if (opt?.kind === 'first-run') {
			activeOutlineContainer.style.translate = `${newPlayerX}px ${newPlayerY}px`;

			activeOutlineContainer.animate(
				{ opacity: ['0%', '100%'] },
				{ easing: 'cubic-bezier(0.61, 1, 0.88, 1)', duration: 400 }
			);

			return;
		}

		if (opt?.kind === 'exploded') {
			const badPlayer = playerElements[opt.on];

			explode(badPlayer, {
				distMultiplier: 5,
				elementModifications: (element) => (element.style.border = '1px solid var(--color-red)')
			});

			badPlayer.animate({ opacity: ['0%', '100%'] }, { easing: 'ease-in', duration: 3500 });
		} else if (opt?.kind === 'gained-life') {
			const player = playerElements[opt.on];

			player.animate({ rotate: '360deg' }, { easing: 'ease-in', duration: 1500 });
		}

		const toX1 = lerp(outlineX, playerX, 0.1);
		const toY1 = lerp(outlineY, playerY, 0.1);

		activeOutlineContainer.animate(
			{ translate: `${toX1}px ${toY1}px`, opacity: '0%' },
			{ fill: 'forwards', easing: 'cubic-bezier(0.25, 1, 0.5, 1)', duration: 100 }
		);

		const fromX2 = lerp(outlineX, newPlayerX, 0.9);
		const fromY2 = lerp(outlineY, newPlayerY, 0.9);

		const toX3 = newPlayerX;
		const toY3 = newPlayerY;

		activeOutlineContainer.animate(
			{ translate: [`${fromX2}px ${fromY2}px`, `${toX3}px ${toY3}px`], opacity: '100%' },
			{ fill: 'forwards', easing: 'cubic-bezier(0.61, 1, 0.88, 1)', duration: 150, delay: 100 }
		);
	}

	function animateIncorrect() {
		for (const text of playerElements[wordBomb.turn]!.querySelectorAll('p')) {
			text.animate(
				{ color: ['var(--color-red)', text.style.color] },
				{ easing: 'cubic-bezier(0.61, 1, 0.88, 1)', duration: 300 }
			);
		}

		incorrectOutlineElement.animate(
			{ opacity: ['100%', '0%'] },
			{ easing: 'cubic-bezier(0.61, 1, 0.88, 1)', duration: 300 }
		);
	}

	function getPlayerFinalPosition(index: number, players: number) {
		const angleBetween = (2 * Math.PI) / players;
		const angle = index * angleBetween;

		const containerRect = playersContainer.getBoundingClientRect();
		const centerX = containerRect.left + containerRect.width / 2;
		const centerY = containerRect.top + containerRect.height / 2;
		const dist = Math.min(windowWidth, windowHeight) * 0.4;

		return {
			x: centerX + Math.cos(angle) * dist,
			y: centerY - Math.sin(angle) * dist
		};
	}

	// Modified version of `svelte/animate/flip`.
	function flipLetter(
		_node: HTMLElement,
		{ from, to }: { from: DOMRect; to: DOMRect }
	): AnimationConfig {
		const delay = 300;
		const duration = (d: number) => Math.sqrt(d) * 20;
		const easing = cubicOut;

		const ox = 0.5;
		const oy = 0.5;

		const fx = from.left + from.width * ox;
		const fy = from.top + from.height * oy;

		const tx = to.left + to.width * ox;
		const ty = to.top + to.height * oy;

		const dx = fx - tx;
		const dy = fy - ty;

		return {
			delay,
			duration: duration(Math.sqrt(dx * dx + dy * dy)),
			easing,
			css: (_t, u) => `translate: ${u * dx}px ${u * dy}px`
		};
	}

	function correctLetterOut(_node: HTMLElement): TransitionConfig {
		const delay = 0;
		const duration = 1500;
		const easing = cubicOut;

		return {
			delay,
			duration,
			easing,
			css: (t, u) =>
				`background-color: var(--color-green);` + `translate: ${u * 200}px 0px;` + `opacity: ${t};`
		};
	}
</script>

<svelte:window
	bind:innerWidth={windowWidth}
	bind:innerHeight={windowHeight}
	on:resize={debounce(() => animateTurnChange(), 150)}
/>

<div class="relative flex flex-1 items-center justify-center gap-4 overflow-hidden p-4 pt-0">
	<div
		bind:this={bombElement}
		class="ease-out-cubic absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 transition-transform duration-[400ms]"
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
		{#each unusedLetters as letter (letter)}
			<div
				animate:flipLetter
				out:correctLetterOut
				class="size-12 content-center border border-green bg-green/15 text-center font-mono uppercase"
			>
				{letter}
			</div>
		{/each}
	</div>
	<div bind:this={activeOutlineContainer} class="absolute top-0 left-0">
		<div
			style={`scale: ${activeOutlineScale}%;`}
			class="rotating-border rotating absolute top-1/2 left-1/2 size-56 -translate-x-1/2 -translate-y-1/2 duration-300"
		></div>
		<div
			bind:this={incorrectOutlineElement}
			style={`scale: ${activeOutlineScale}%;`}
			class="rotating ease-out-cubic absolute top-1/2 left-1/2 z-10 size-56 -translate-x-1/2 -translate-y-1/2 border-4 border-red opacity-0 transition-transform duration-300"
		></div>
	</div>
	<div
		bind:this={playersContainer}
		class="ease-out-cubic relative size-full transition-transform duration-[400ms]"
	>
		{#each players as [uuid, player], i (uuid)}
			{@const angleBetween = (2 * Math.PI) / players.length}
			{@const angle = i * angleBetween}
			{@const dist = Math.min(windowWidth, windowHeight) * 0.4}
			<div
				bind:this={playerElements[uuid]}
				style={`translate: calc(-50% + cos(${angle}rad) * ${dist}px) calc(-50% - sin(${angle}rad) * ${dist}px);` +
					`scale: ${playerScale}%;`}
				class="ease-out-cubic absolute top-1/2 left-1/2 flex flex-col items-center p-2 transition-transform duration-[400ms]"
			>
				<div class="relative mb-2">
					<Avatar {ctx} {uuid} />
					<div class="absolute bottom-0 left-0 flex flex-col gap-y-1.5 mix-blend-plus-lighter">
						{#each { length: player!.lives }}
							<HeartIcon />
						{/each}
					</div>
				</div>
				<p class="font-medium">{ctx.clients[uuid]!.username}</p>
				{#if ctx.uuid === uuid}
					<input
						bind:this={playerInputElement}
						bind:value={playerInput}
						type="text"
						disabled={ctx.uuid !== wordBomb.turn}
						placeholder="answer"
						class="mt-2.5 w-24 border border-green px-2 py-1.5 text-center text-lg focus:border-pastel-green focus:ring-0 focus:outline-none disabled:opacity-50"
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
	</div>
</div>
