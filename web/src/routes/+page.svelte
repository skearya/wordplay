<script lang="ts">
	import type { SocketParams } from '@bindings/SocketParams';
	import type { ServerMessage } from '@bindings/ServerMessage';
	import { rootEmitter } from '$lib/events';
	import { unreachable } from '$lib/utils';
	import { onMount, type ComponentProps } from 'svelte';
	import Wordplay from '$lib/components/Wordplay.svelte';

	type State =
		| { kind: 'loading' }
		| { kind: 'connected' }
		| { kind: 'ready'; props: ComponentProps<typeof Wordplay> }
		| { kind: 'error' };

	let state = $state<State>({ kind: 'loading' });
	let socket: WebSocket | undefined;

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
			state = { kind: 'connected' };
		});

		socket.addEventListener('message', (e) => {
			const message = JSON.parse(e.data) as ServerMessage;

			if (import.meta.env.DEV) {
				console.log('Message', message, e.data);
			}

			if (message.kind === 'general' && message.data.kind === 'info') {
				state = {
					kind: 'ready',
					props: {
						info: message.data,
						sendMsg: (message) => socket!.send(JSON.stringify(message))
					}
				};

				return;
			}

			rootEmitter.emit(message);
		});

		socket.addEventListener('error', (e) => {
			console.log('WebSocket error', e);
		});

		socket.addEventListener('close', (e) => {
			console.log('WebSocket closed', e);
			state = { kind: 'error' };
		});

		return () => socket?.close();
	});

	$effect(() => {
		if (state.kind === 'connected') {
			const id = setTimeout(() => {
				if (state.kind === 'connected') {
					socket?.close();
					state = { kind: 'error' };
				}
			}, 5000);

			return () => clearTimeout(id);
		}
	});
</script>

{#if state.kind === 'loading'}
	<h1>Loading</h1>
{:else if state.kind === 'connected'}
	<h1>Loading (established connection)</h1>
{:else if state.kind === 'ready'}
	<Wordplay {...state.props} />
{:else if state.kind === 'error'}
	<h1>Error</h1>
{:else if state satisfies never}
	{unreachable(state)}
{/if}
