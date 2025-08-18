<script lang="ts">
	import type { PageProps } from './$types';
	import type { Context } from '@bindings/Context';
	import type { ServerMessage } from '@bindings/ServerMessage';
	import type { SocketParams } from '@bindings/SocketParams';
	import { onMount } from 'svelte';
	import Wordplay from '$lib/components/Wordplay.svelte';
	import { handleServerMessage } from '$lib/handlers';
	import { unreachable } from '$lib/utils';

	const { data }: PageProps = $props();

	type State = { kind: 'loading' } | { kind: 'connected' } | { kind: 'ready' } | { kind: 'error' };

	let socket: WebSocket | undefined;
	// Can't name this variable "state"
	let connection = $state<State>({ kind: 'loading' });
	let context = $state<Context>();

	onMount(() => {
		const room = 'one';

		const params: SocketParams = {
			username: 'Client',
			rejoinToken: null
		};

		const urlParams = new URLSearchParams(
			Object.entries(params).filter((param): param is [string, string] => param[1] !== null)
		);

		socket = new WebSocket(`ws://localhost:3000/${room}?${urlParams}`);

		socket.addEventListener('open', () => {
			console.log('Connected');
			connection = { kind: 'connected' };
		});

		socket.addEventListener('message', (e) => {
			const message = JSON.parse(e.data) as ServerMessage;

			if (import.meta.env.DEV) {
				console.log('Message', message, e.data);
			}

			if (connection.kind === 'ready') {
				handleServerMessage(context!, message);
			} else if (message.kind === 'general' && message.data.kind === 'info') {
				context = message.data;
				connection = { kind: 'ready' };
			}
		});

		socket.addEventListener('error', (e) => {
			console.log('WebSocket error', e);
		});

		socket.addEventListener('close', (e) => {
			console.log('WebSocket closed', e);
			connection = { kind: 'error' };
		});

		return () => socket?.close();
	});

	$effect(() => {
		if (connection.kind === 'connected') {
			const id = setTimeout(() => {
				if (connection.kind === 'connected') {
					socket?.close();
					connection = { kind: 'error' };
				}
			}, 5000);

			return () => clearTimeout(id);
		}
	});
</script>

{#if connection.kind === 'loading'}
	<h1>Loading</h1>
{:else if connection.kind === 'connected'}
	<h1>Loading (established connection)</h1>
{:else if connection.kind === 'ready'}
	<Wordplay context={context!} send={(message) => socket!.send(JSON.stringify(message))} />
{:else if connection.kind === 'error'}
	<h1>Error</h1>
{:else if connection satisfies never}
	{unreachable(connection)}
{/if}
