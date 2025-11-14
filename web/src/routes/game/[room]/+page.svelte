<script lang="ts">
	import type { PageProps } from './$types';
	import type { ServerMessage } from '@bindings/ServerMessage';
	import type { SocketParams } from '@bindings/SocketParams';
	import { onMount } from 'svelte';
	import { unreachable } from '$lib/utils';

	const { data, params }: PageProps = $props();

	type State =
		| { kind: 'connecting' }
		| { kind: 'connected' }
		| { kind: 'ready' }
		| { kind: 'error'; details: string };

	let socket: WebSocket | undefined;
	let connection = $state<State>({ kind: 'connecting' });

	onMount(() => {
		const socketParams: SocketParams = {
			username: 'Client',
			rejoinToken: null
		};

		const urlParams = new URLSearchParams(
			Object.entries(socketParams).filter((param): param is [string, string] => param[1] !== null)
		);

		socket = new WebSocket(`ws://localhost:3000/${params.room}?${urlParams}`);

		socket.addEventListener('open', () => {
			connection = { kind: 'connected' };
		});

		socket.addEventListener('message', (e) => {
			const message = JSON.parse(e.data) as ServerMessage;

			if (import.meta.env.DEV) {
				console.log('Message', message, e.data);
			}

			if (message.kind === 'info') {
				connection = {
					kind: 'ready'
				};
			}
		});

		socket.addEventListener('error', (e) => {
			console.log('WebSocket error', e);
		});

		socket.addEventListener('close', (e) => {
			console.log('WebSocket closed', e);
			connection = { kind: 'error', details: e.reason };
		});

		return () => socket?.close();
	});

	$effect(() => {
		if (connection.kind === 'connected') {
			const id = setTimeout(() => {
				if (connection.kind === 'connected') {
					socket?.close();
					connection = { kind: 'error', details: 'Timed out waiting for server response' };
				}
			}, 5000);

			return () => clearTimeout(id);
		}
	});
</script>

{#if connection.kind === 'connecting'}
	<h1>Connecting</h1>
{:else if connection.kind === 'connected'}
	<h1>Connected (awaiting details)</h1>
{:else if connection.kind === 'ready'}
	<h1>Ready</h1>
{:else if connection.kind === 'error'}
	<h1>Error {connection.details}</h1>
{:else if connection satisfies never}
	{unreachable(connection)}
{/if}
