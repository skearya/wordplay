import type { Component } from 'solid-js';
import type { ClientMessage, ServerMessage } from './types/messages';
import { createSignal, Switch, Match, For, onCleanup } from 'solid-js';
import { match } from 'ts-pattern';

const App: Component = () => {
	const [state, setState] = createSignal<'connecting' | 'error' | 'lobby' | 'game'>('connecting');
	const [messages, setMessages] = createSignal<Array<string>>([]);
	const [input, setInput] = createSignal<string>('');

	const room = prompt('room', 'one');
	const username = prompt('username', 'skeary');

	const params = new URLSearchParams({ username: username });
	const socket = new WebSocket(
		`${import.meta.env.PUBLIC_SERVER}/rooms/${room}?${params.toString()}`
	);

	socket.addEventListener('message', (event) => {
		const message: ServerMessage = JSON.parse(event.data);
	});

	function sendMessage(data: ClientMessage) {
		socket.send(JSON.stringify(data));
	}

	onCleanup(() => {
		socket.close();
	});

	return (
		<Switch>
			<Match when={state() === 'connecting'}>
				<h1>connecting</h1>
			</Match>
			<Match when={state() === 'error'}>
				<h1>we errored</h1>
			</Match>
			<Match when={state() === 'lobby'}>
				<h1>ready players: </h1>

				<input
					type="text"
					onKeyDown={(event) => {
						if (event.key === 'Enter') {
							sendMessage({
								type: 'chatMessage',
								content: event.currentTarget.value
							});
						}
					}}
				/>

				<ul class="list-item">
					<For each={messages()}>{(message, _) => <li>{message}</li>}</For>
				</ul>
			</Match>
			<Match when={state() === 'game'}>
				<h1>game</h1>
			</Match>
		</Switch>
	);
};

export { App };
