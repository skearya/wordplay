<script lang="ts">
	import type { GameState } from '@bindings/GameState';
	import type { Props } from '$lib/context';
	import { onMount } from 'svelte';
	import { gameEmitter } from '$lib/events';
	import { unreachable } from '$lib/utils';
	import WordBomb from './WordBomb.svelte';

	const { ctx, initial, sendMsg }: Props<GameState> = $props();

	let state = $state(initial);

	onMount(() =>
		gameEmitter.handle({
			endRequest: () => {}
		})
	);
</script>

{#if state.variant.kind === 'wordBomb'}
	<WordBomb {ctx} initial={state.variant} {sendMsg} />
{:else if state.variant.kind === 'anagrams'}
	anagrams
{:else if state.variant satisfies never}
	{unreachable(state.variant)}
{/if}
