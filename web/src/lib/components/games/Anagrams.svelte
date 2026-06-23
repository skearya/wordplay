<script lang="ts">
	import type { AnagramsState } from '@bindings/AnagramsState';
	import type { Props } from '$lib/context';
	import { onMount } from 'svelte';
	import { anagramsEmitter } from '$lib/events';

	let {
		ctx = $bindable(),
		state: anagrams = $bindable(),
		sendMsg
	}: Props<AnagramsState> = $props();

	let playerInput = $state('');

	onMount(() =>
		anagramsEmitter.handle({
			valid: ({ uuid, points }) => {
				anagrams.players[uuid]!.points += points;
			},
			invalid: ({ uuid, reason }) => {}
		})
	);
</script>

<pre>{JSON.stringify(anagrams, null, 2)}</pre>

<input
	bind:value={playerInput}
	type="text"
	disabled={!(ctx.uuid in anagrams.players)}
	placeholder="answer"
	class="mt-2.5 w-24 border border-green px-2 py-1.5 text-center text-lg focus:border-pastel-green focus:ring-0 focus:outline-none disabled:opacity-50"
	onkeyup={(e) => {
		if (e.key === 'Enter') {
			sendMsg({ kind: 'anagrams', data: { kind: 'guess', word: e.currentTarget.value } });
		}
	}}
/>
