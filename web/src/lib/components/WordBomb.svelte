<script lang="ts">
	import type { WordBombState } from '@bindings/WordBombState';
	import type { Props } from '$lib/context';
	import { onMount } from 'svelte';
	import { wordBombEmitter } from '$lib/events';

	const { ctx, initial, sendMsg }: Props<WordBombState> = $props();

	let state = $state(initial);

	onMount(() =>
		wordBombEmitter.handle({
			input: ({ input }) => {
				const player = state.players[state.turn]!;

				player.input = input;
			},
			valid: ({ guess, prompt, life, turn }) => {
				const player = state.players[state.turn]!;

				player.letters = [...new Set([...player.letters, ...guess])];

				if (life) {
					player.lives += 1;
					player.letters = [];
				}

				state.prompt = prompt;
				state.turn = turn;
			},
			invalid: () => {},
			exploded: ({ prompt, turn }) => {
				const player = state.players[state.turn]!;

				player.lives -= 1;

				state.prompt = prompt;
				state.turn = turn;
			}
		})
	);
</script>

<h1>prompt: {state.prompt}</h1>
<input
	type="text"
	oninput={(e) => {
		sendMsg({ kind: 'wordBomb', data: { kind: 'input', input: e.currentTarget.value } });
	}}
	onkeyup={(e) => {
		if (e.key === 'Enter') {
			sendMsg({ kind: 'wordBomb', data: { kind: 'guess', word: e.currentTarget.value } });
		}
	}}
/>

<h1>turn: {ctx.clients[state.turn]!.username}</h1>

<h1>players</h1>
{#each Object.entries(state.players) as [uuid, data]}
	<p>{ctx.clients[uuid]!.username}: {data!.input}, {data!.lives}</p>
{/each}
