<script lang="ts">
	import type { PageProps } from './$types';
	import type { ClientMessage } from '@bindings/ClientMessage';
	import type { ServerMessage } from '@bindings/ServerMessage';
	import type { SocketParams } from '@bindings/SocketParams';
	import type { Context } from '$lib/context';
	import { PUBLIC_SERVER_URL } from '$env/static/public';
	import { onDestroy } from 'svelte';
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
	import Join from './Join.svelte';

	const { params }: PageProps = $props();

	type State =
		| { kind: 'awaiting' }
		| { kind: 'connecting'; username: string }
		| { kind: 'connected' }
		| { kind: 'ready'; ctx: Context; sendMsg: (message: ClientMessage) => void }
		| { kind: 'error'; reason: string; code?: number; clean?: boolean };

	let socket: WebSocket | undefined;
	let connection = $state<State>({ kind: 'awaiting' });
	let manuallyClosed = false;

	$effect(() => {
		if (connection.kind === 'connected') {
			const id = setTimeout(() => {
				if (connection.kind === 'connected') {
					socket?.close();
					connection = { kind: 'error', reason: 'Timed out waiting for server response' };
				}
			}, 5000);

			return () => clearTimeout(id);
		}
	});

	onDestroy(() => {
		socket?.close();
		manuallyClosed = true;
	});

	function connectSocket(username: string) {
		if (connection.kind !== 'awaiting') return;

		const socketParams: SocketParams = {
			username,
			rejoinToken: import.meta.env.DEV ? null : localStorage.getItem('rejoinToken')
		};

		const urlParams = new URLSearchParams(
			Object.entries(socketParams).filter((param): param is [string, string] => param[1] !== null)
		);

		socket = new WebSocket(
			`${import.meta.env.DEV ? 'ws' : 'wss'}://${PUBLIC_SERVER_URL}/connect/${params.room}?${urlParams}`
		);

		socket.addEventListener('open', () => {
			connection = { kind: 'connected' };
		});

		socket.addEventListener('message', (e) => {
			const message = JSON.parse(e.data) as ServerMessage;

			if (import.meta.env.DEV) {
				console.log('Message', message);
			}

			switch (message.kind) {
				case 'info':
					localStorage.setItem('rejoinToken', message.data.rejoinToken);

					connection = {
						kind: 'ready',
						ctx: message.data,
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
			console.error('WebSocket error', e);
		});

		socket.addEventListener('close', (e) => {
			if (manuallyClosed) return;

			console.error('WebSocket closed', e);

			connection = {
				kind: 'error',
				reason: e.reason,
				code: e.code,
				clean: e.wasClean
			};
		});
	}
</script>

{#if connection.kind === 'awaiting' || connection.kind === 'connecting' || connection.kind === 'connected'}
	<Join
		room={params.room}
		connection={connection.kind}
		onJoin={(username) => connectSocket(username)}
	/>
{:else if connection.kind === 'ready'}
	<Wordplay bind:ctx={connection.ctx} sendMsg={connection.sendMsg} />
{:else if connection.kind === 'error'}
	<main class="flex h-screen items-center justify-center">
		<div>
			<h1>Connection error, sorry about that!</h1>
			<code>{JSON.stringify(connection, null, 2)}</code>
		</div>
	</main>
{:else if connection satisfies never}
	{unreachable(connection)}
{/if}
