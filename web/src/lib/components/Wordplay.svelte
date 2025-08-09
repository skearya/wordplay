<script lang="ts">
	import { unreachable } from '$lib/utils';
	import type { ServerState } from '@bindings/ServerState';

	const { initalState }: { initalState: ServerState } = $props();

	let state = $state(initalState);
</script>

{#if state.kind === 'lobby'}
	<h1>Lobby</h1>
{:else if state.kind === 'game'}
	<h1>game</h1>
{:else if state satisfies never}
	{unreachable(state)}
{/if}
