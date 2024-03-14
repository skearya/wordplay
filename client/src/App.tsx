import type { Component } from 'solid-js';
import type { ClientMessage, ServerMessage } from './types/messages';
import { createSignal, Switch, Match, For, onCleanup, createEffect, Show } from 'solid-js';
import { createStore } from 'solid-js/store';
import { P, match } from 'ts-pattern';
import { ConnectionData, GameData, LobbyData } from './types/stores';

const App: Component = () => {
	const [state, setState] = createSignal<'connecting' | 'error' | 'lobby' | 'game'>('connecting');

	const [connection, setConnection] = createStore<ConnectionData>({
		room: prompt('room', 'one')!,
		username: prompt('username', 'skeary')!,
		uuid: '',
		chatMessages: []
	});

	const [lobby, setLobby] = createStore<LobbyData>({
		readyPlayers: [],
		previousWinner: null,
		startingCountdown: null
	});

	const [game, setGame] = createStore<GameData>({
		players: [],
		currentTurn: '',
		prompt: '',
		usedLetters: new Set()
	});

	const params = new URLSearchParams({ username: connection.username });
	const socket = new WebSocket(
		`${import.meta.env.PUBLIC_SERVER}/rooms/${connection.room}?${params.toString()}`
	);

	socket.addEventListener('message', (event) => {
		const message: ServerMessage = JSON.parse(event.data);

		match(message)
			.with({ type: 'roomInfo' }, (message) => {
				setConnection('uuid', message.uuid);

				if (message.state.type === 'lobby') {
					setState('lobby');
					setLobby({
						readyPlayers: message.state.ready,
						startingCountdown: message.state.startingCountdown
					});
				} else {
					setState('game');
					setGame({
						players: message.state.players,
						currentTurn: message.state.turn,
						prompt: message.state.prompt,
						usedLetters:
							message.state.usedLetters !== undefined
								? new Set(message.state.usedLetters)
								: undefined
					});
				}
			})
			.with({ type: P.union('serverMessage', 'chatMessage') }, (message) => {
				const messageContent =
					message.type === 'serverMessage'
						? `server: ${message.content}`
						: `${message.author}: ${message.content}`;

				setConnection('chatMessages', connection.chatMessages.length, messageContent);
			})
			.with({ type: 'readyPlayers' }, (message) => {
				setLobby({ readyPlayers: message.players });
			})
			.with({ type: 'startingCountdown' }, (message) => {
				if (message.state.type === 'inProgress') {
					setLobby({ startingCountdown: message.state.timeLeft });
				} else {
					setLobby({ startingCountdown: null });
				}
			})
			.with({ type: 'gameStarted' }, (message) => {
				if (message.rejoinToken !== undefined) {
					let tokens = JSON.parse(localStorage.getItem('rejoinTokens') || '{}');
					tokens[connection.room] = message.rejoinToken;
					localStorage.setItem('rejoinTokens', JSON.stringify(tokens));
				}

				setState('game');
				setGame({
					players: message.players,
					currentTurn: message.turn,
					prompt: message.prompt,
					usedLetters: new Set()
				});
			})
			.with({ type: 'playerUpdate' }, (message) => {
				setGame(
					'players',
					(player) => player.uuid === message.uuid,
					(player) => ({
						...player,
						disconnected: message.state.type === 'disconnected',
						username:
							message.state.type === 'reconnected' ? message.state.username : player.username
					})
				);
			})
			.with({ type: 'inputUpdate' }, (message) => {
				setGame('players', (player) => player.uuid === message.uuid, 'input', message.input);
			})
			.with({ type: 'invalidWord' }, (message) => {})
			.with({ type: 'newPrompt' }, (message) => {
				setGame('prompt', message.prompt);
			})
			.with({ type: 'gameEnded' }, (message) => {
				setState('lobby');
				setLobby({
					previousWinner: game.players.find((player) => player.uuid === message.winner)!.username,
					readyPlayers: [],
					startingCountdown: null
				});
			})
			.exhaustive();
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
				<Show when={lobby.startingCountdown !== null}>
					<h1>{lobby.startingCountdown}</h1>
				</Show>
				<Show when={lobby.previousWinner !== null}>
					<h1>previous winner: {lobby.previousWinner}</h1>
				</Show>
				<h1>ready players: {lobby.readyPlayers.map((player) => player.username).join(' ')}</h1>
				<div class="flex">
					<input
						class="border"
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
					<button onClick={() => sendMessage({ type: 'ready' })}>ready</button>
				</div>
				<ul class="list-item">
					<For each={connection.chatMessages}>{(message, _) => <li>{message}</li>}</For>
				</ul>
			</Match>
			<Match when={state() === 'game'}>
				<h1>{game.prompt}</h1>
				<h1>{game.players.find((player) => player.uuid === game.currentTurn)?.username}</h1>
				<input
					class="border"
					type="text"
					onKeyDown={(event) => {
						if (event.key === 'Enter') {
							sendMessage({
								type: 'guess',
								word: event.currentTarget.value
							});
						}
					}}
				/>
				<input
					class="border"
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
					<For each={connection.chatMessages}>{(message, _) => <li>{message}</li>}</For>
				</ul>
			</Match>
		</Switch>
	);
};

export { App };
