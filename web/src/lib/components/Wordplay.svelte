<script lang="ts">
	import type { ClientMessage } from '@bindings/ClientMessage';
	import type { Context } from '@bindings/Context';
	import { unreachable } from '$lib/utils';
	import Lobby from './Lobby.svelte';

	const { context, send }: { context: Context; send: (message: ClientMessage) => void } = $props();
</script>

{#if context.state.kind === 'lobby'}
	<Lobby
		context={{ ...context, state: context.state }}
		send={(message) => send({ kind: 'lobby', data: message })}
	/>
{:else if context.state.kind === 'game'}
	<pre>(game)</pre>
{:else if context.state satisfies never}
	{unreachable(context.state)}
{/if}
