<script lang="ts">
	import type { GameState } from '@bindings/GameState';
	import type { Props } from '$lib/context';
	import { onMount } from 'svelte';
	import { gameEmitter } from '$lib/events';
	import { unreachable } from '$lib/utils';
	import WordBomb from './WordBomb.svelte';

	let { ctx = $bindable(), state: game = $bindable(), sendMsg }: Props<GameState> = $props();

	onMount(() =>
		gameEmitter.handle({
			endRequest: () => {}
		})
	);
</script>

{#if game.variant.kind === 'wordBomb'}
	<WordBomb bind:ctx bind:state={game.variant} {sendMsg} />
{:else if game.variant.kind === 'anagrams'}
	anagrams
{:else if game.variant satisfies never}
	{unreachable(game.variant)}
{/if}
