<script lang="ts">
	import type { PageProps } from './$types';
	import type { ClientMessage } from '@bindings/ClientMessage';
	import type { ServerMessage } from '@bindings/ServerMessage';
	import type { SocketParams } from '@bindings/SocketParams';
	import type { Context } from '$lib/context';
	import { onDestroy } from 'svelte';
	import { fade } from 'svelte/transition';
	import Wordplay from '$lib/components/Wordplay.svelte';
	import {
		anagramsEmitter,
		coreEmitter,
		gameEmitter,
		generalEmitter,
		lobbyEmitter,
		wordBombEmitter
	} from '$lib/events';
	import { unreachable } from '$lib/utils';
	import Background from './Background.svelte';
	import Join from './Join.svelte';

	const { params }: PageProps = $props();

	type State =
		| { kind: 'awaiting' }
		| { kind: 'connecting'; username: string }
		| { kind: 'connected' }
		| { kind: 'ready'; context: Context; sendMsg: (message: ClientMessage) => void }
		| { kind: 'error'; details: string };

	let socket: WebSocket | undefined;
	let connection = $state<State>({ kind: 'awaiting' });

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

	onDestroy(() => {
		socket?.close();
	});

	function connectSocket(username: string) {
		if (connection.kind !== 'awaiting') return;

		const rejoinToken = localStorage.getItem('rejoinToken');

		const socketParams: SocketParams = {
			username,
			rejoinToken
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

			switch (message.kind) {
				case 'info':
					connection = {
						kind: 'ready',
						context: message.data,
						sendMsg: (message) => socket!.send(JSON.stringify(message))
					};
					break;
				case 'core':
					coreEmitter.emit(message.data);
					break;
				case 'general':
					generalEmitter.emit(message.data);
					break;
				case 'lobby':
					lobbyEmitter.emit(message.data);
					break;
				case 'game':
					gameEmitter.emit(message.data);
					break;
				case 'wordBomb':
					wordBombEmitter.emit(message.data);
					break;
				case 'anagrams':
					anagramsEmitter.emit(message.data);
					break;
			}
		});

		socket.addEventListener('error', (e) => {
			console.log('WebSocket error', e);
		});

		socket.addEventListener('close', (e) => {
			console.log('WebSocket closed', e);
			connection = { kind: 'error', details: e.reason };
		});
	}
</script>

{#if connection.kind === 'awaiting' || connection.kind === 'connecting' || connection.kind === 'connected'}
	<main out:fade>
		<Background />
		<Join
			room={params.room}
			connection={connection.kind}
			onJoin={(username) => connectSocket(username)}
		/>
	</main>
{:else if connection.kind === 'ready'}
	<Wordplay initial={connection.context} sendMsg={connection.sendMsg} />
{:else if connection.kind === 'error'}
	<main class="flex h-screen items-center justify-center">
		<h1>Error: {connection.details}</h1>
	</main>
{:else if connection satisfies never}
	{unreachable(connection)}
{/if}
