<script lang="ts">
	import type { AnagramsState } from '@bindings/AnagramsState';
	import type { Props } from '$lib/context';
	import { onMount, tick } from 'svelte';
	import { flip } from 'svelte/animate';
	import { anagramsEmitter } from '$lib/events';
	import Avatar from '../Avatar.svelte';

	let {
		ctx = $bindable(),
		state: anagrams = $bindable(),
		sendMsg
	}: Props<AnagramsState> = $props();

	let playerElements: Record<string, HTMLElement> = $state({});
	let letterElements: HTMLElement[] = $state([]);
	let playerInputElement: HTMLElement;
	let invalidReasonElement: HTMLElement | undefined = $state();

	let invalidReason = $state<string | null>(null);

	onMount(() =>
		anagramsEmitter.handle({
			valid: ({ uuid, word, points }) => {
				const prevRank = players.findIndex(([playerUuid]) => uuid === playerUuid);
				anagrams.players[uuid].points += points;

				if (prevRank !== players.findIndex(([playerUuid]) => uuid === playerUuid)) {
					animateRankJump(uuid);
				}

				if (uuid === ctx.uuid) {
					invalidReason = null;
					animateCorrect(word);
				}
			},
			invalid: ({ uuid, reason }) => {
				if (uuid === ctx.uuid) {
					invalidReason = reason;
					animateIncorrect();
				}
			}
		})
	);

	let players = $derived(
		Object.entries(anagrams.players).toSorted((a, b) => b[1].points - a[1].points)
	);

	function animateRankJump(uuid: string) {
		playerElements[uuid].animate(
			{
				boxShadow: [
					'0px 0px 24px 8px color-mix(in oklab, var(--color-green) 100%, transparent)',
					'0px 0px 96px 8px color-mix(in oklab, var(--color-green) 0%, transparent)'
				],
				backgroundColor: [
					'color-mix(in oklab, var(--color-green) 100%, transparent)',
					'color-mix(in oklab, var(--color-green) 0%, transparent)'
				]
			},
			{ easing: 'ease-out', duration: 1000 }
		);
	}

	function animateCorrect(word: string) {
		let used = anagrams.anagram;

		for (const letter of word) {
			const i = used.indexOf(letter);
			used = used.replace(letter, ' ');

			const { top, left } = letterElements[i].getBoundingClientRect();
			const clone = letterElements[i].cloneNode(true) as HTMLElement;

			clone.style.position = 'absolute';
			clone.style.top = `${top}px`;
			clone.style.left = `${left}px`;
			clone.style.translate = `0px 0px`;
			clone.style.pointerEvents = 'none';
			clone.classList.remove('border-r-0');

			document.body.appendChild(clone);

			clone
				.animate(
					{
						translate: `${(i - 2.5) * 24}px 96px`,
						rotate: `${Math.random() - 0.5}rad`,
						scale: `75%`,
						opacity: '0%'
					},
					{ easing: 'cubic-bezier(0.1, 1.0, 0.9, 1)', duration: 1000 }
				)
				.addEventListener('finish', () => clone.remove());

			letterElements[i].animate(
				{
					backgroundColor: [
						'var(--color-bright-green)',
						'color-mix(in oklab, var(--color-green) 15%, transparent)'
					]
				},
				{ easing: 'ease-out', duration: 300 }
			);
		}
	}

	function animateIncorrect() {
		playerInputElement.animate(
			{
				borderColor: [
					'var(--color-red)',
					'color-mix(in oklab, var(--color-green) 25%, transparent)'
				]
			},
			{ easing: 'ease-out', duration: 300 }
		);

		tick().then(() => {
			if (!invalidReasonElement) return;

			for (const animation of invalidReasonElement.getAnimations()) {
				animation.cancel();
			}

			invalidReasonElement.animate(
				{ opacity: ['100%', '0%'] },
				{ easing: 'ease-out', delay: 1000, duration: 3000 }
			);
		});
	}
</script>

<div class="flex flex-1 items-center justify-center gap-x-16">
	<div class="divide-gradient flex w-72 flex-col divide-y">
		{#each players as [uuid, player], i (uuid)}
			<div
				bind:this={playerElements[uuid]}
				animate:flip={{ duration: (d) => Math.sqrt(d) * 60 }}
				class="flex items-center justify-start gap-x-2 p-2.5 text-lg"
			>
				<span class="font-mono text-sm">#{i + 1}</span>
				<Avatar {ctx} {uuid} size="sm" class="flex-none" />
				<p class="truncate">{ctx.clients[uuid].username}</p>
				<p class="ml-auto font-medium">{player.points}</p>
			</div>
		{/each}
	</div>
	<div class="h-full w-[1px] rotate-3 bg-green"></div>
	<div class="relative flex flex-col gap-y-4">
		<div class="flex">
			{#each anagrams.anagram as letter, i}
				<p
					bind:this={letterElements[i]}
					class="size-20 content-center border border-r-0 border-green bg-green/15 text-center font-mono text-4xl last:border-r"
				>
					{letter}
				</p>
			{/each}
		</div>
		<input
			{@attach (input) => input.focus()}
			bind:this={playerInputElement}
			type="text"
			disabled={!(ctx.uuid in anagrams.players)}
			placeholder="answer"
			class="border border-green/25 bg-dark-dark-green px-2 py-1.5 text-center text-lg focus:ring-0 focus:outline-none disabled:opacity-50"
			onkeyup={(e) => {
				if (e.key === 'Enter') {
					sendMsg({ kind: 'anagrams', data: { kind: 'guess', word: e.currentTarget.value } });
				}
			}}
		/>
		{#if invalidReason}
			<p
				bind:this={invalidReasonElement}
				class="absolute -bottom-4 left-1/2 -translate-x-1/2 translate-y-full text-lg whitespace-nowrap text-pastel-red"
			>
				{invalidReason}
			</p>
		{/if}
	</div>
</div>

<style>
	.divide-gradient {
		:where(& > :not(:last-child)) {
			border-image: linear-gradient(to right, transparent, var(--color-green)) 1;
		}
	}
</style>
