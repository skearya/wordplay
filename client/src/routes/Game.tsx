import type { Component } from 'solid-js';
import type { ClientMessage, ServerMessage } from '../lib/types/messages';
import { Switch, Match, onCleanup, batch, useContext, createSignal } from 'solid-js';
import { produce } from 'solid-js/store';
import { P, match } from 'ts-pattern';
import { Context } from '../lib/context';
import { Lobby } from '../lib/game/Lobby';
import { InGame } from '../lib/game/InGame';
import { ChatMessages } from '../lib/game/ChatMessages';
import { Nav } from '../lib/game/Nav';
import { Connecting } from '../lib/game/Connecting';
import { Errored } from '../lib/game/Errored';

const Game: Component = () => {
	const context = useContext(Context);
	if (!context) throw new Error('Not called inside context provider?');
	const { connection, game, state } = context[0];
	const { setConnection, setGame, setLobby, setState } = context[1];

	const [connectionError, setConnectionError] = createSignal<string | null>(null);

	const rejoinToken: string | undefined = JSON.parse(localStorage.getItem('rejoinTokens') ?? '{}')[
		connection.room
	];

	const params = new URLSearchParams({
		username: connection.username,
		...(rejoinToken && { rejoinToken })
	});

	const socket = new WebSocket(
		`${import.meta.env.PUBLIC_SERVER}/rooms/${connection.room}?${params.toString()}`
	);

	const sendMessage = (data: ClientMessage) => socket.send(JSON.stringify(data));

	socket.addEventListener('message', (event) => {
		const message: ServerMessage = JSON.parse(event.data);

		match(message)
			.with({ type: 'roomInfo' }, (message) => {
				setConnection('uuid', message.uuid);
				setConnection('clients', message.clients);

				batch(() => {
					if (message.state.type === 'lobby') {
						setState('lobby');
						setLobby({
							readyPlayers: message.state.ready,
							startingCountdown: message.state.startingCountdown
						});
					} else {
						const usedLetters = message.state.usedLetters
							? new Set(message.state.usedLetters)
							: null;
						const input =
							message.state.players.find((player) => player.uuid === connection.uuid)?.input ?? '';

						setState('game');
						setGame({
							players: message.state.players,
							currentTurn: message.state.turn,
							prompt: message.state.prompt,
							usedLetters,
							input
						});
					}
				});
			})
			.with({ type: P.union('serverMessage', 'chatMessage') }, (message) => {
				const messageContent =
					message.type === 'serverMessage'
						? `server: ${message.content}`
						: `${connection.clients.find((client) => client.uuid === message.author)!.username}: ${message.content}`;

				setConnection('chatMessages', connection.chatMessages.length, messageContent);
			})
			.with({ type: 'connectionUpdate', state: { type: 'connected' } }, (message) => {
				setConnection(
					'chatMessages',
					connection.chatMessages.length,
					`${message.state.username} has joined`
				);
				setConnection('clients', connection.clients.length, {
					uuid: message.uuid,
					username: message.state.username
				});
			})
			.with({ type: 'connectionUpdate', state: { type: 'reconnected' } }, (message) => {
				setConnection(
					'chatMessages',
					connection.chatMessages.length,
					`${message.state.username} has joined`
				);
				setConnection('clients', connection.clients.length, {
					uuid: message.uuid,
					username: message.state.username
				});
				setGame('players', (player) => player.uuid === message.uuid, {
					disconnected: false,
					username: message.state.username
				});
			})
			.with({ type: 'connectionUpdate', state: { type: 'disconnected' } }, (message) => {
				setConnection(
					'chatMessages',
					connection.chatMessages.length,
					`${connection.clients.find((client) => client.uuid === message.uuid)!.username} has left`
				);
				setConnection('clients', (clients) =>
					clients.filter((client) => client.uuid != message.uuid)
				);
				setGame('players', (player) => player.uuid === message.uuid, 'disconnected', true);
			})
			.with({ type: 'readyPlayers' }, (message) => {
				setLobby({
					readyPlayers: message.ready,
					startingCountdown: message.ready.length >= 2 ? 10 : null
				});
			})
			.with({ type: 'startingCountdown' }, (message) => {
				setLobby({
					startingCountdown: message.state.type === 'inProgress' ? message.state.timeLeft : null
				});
			})
			.with({ type: 'gameStarted' }, (message) => {
				if (message.rejoinToken) {
					let tokens = JSON.parse(localStorage.getItem('rejoinTokens') ?? '{}');
					tokens[connection.room] = message.rejoinToken;
					localStorage.setItem('rejoinTokens', JSON.stringify(tokens));
				}

				batch(() => {
					setState('game');
					setGame({
						players: message.players,
						currentTurn: message.turn,
						prompt: message.prompt,
						guessError: ['', ''],
						usedLetters: new Set(),
						input: ''
					});
				});
			})
			.with({ type: 'inputUpdate' }, (message) => {
				setGame('players', (player) => player.uuid === message.uuid, 'input', message.input);
			})
			.with({ type: 'invalidWord' }, (message) => {
				setGame('guessError', [message.uuid, message.reason.type]);
			})
			.with({ type: 'newPrompt' }, (message) => {
				setGame(
					produce((game) => {
						let turn = game.players.find((player) => player.uuid === game.currentTurn)!;
						turn.lives += message.lifeChange;

						if (turn.uuid === connection.uuid && message.word) {
							game.usedLetters = new Set([...game.usedLetters!, ...message.word]);
						}

						if (message.newTurn === connection.uuid) {
							game.input = '';
						}

						game.prompt = message.newPrompt;
						game.currentTurn = message.newTurn;
					})
				);
			})
			.with({ type: 'gameEnded' }, (message) => {
				batch(() => {
					setState('lobby');
					setLobby({
						previousWinner: connection.clients.find((player) => player.uuid === message.winner)!
							.username,
						readyPlayers: [],
						startingCountdown: null
					});
				});
			})
			.exhaustive();
	});

	socket.addEventListener('close', (event) => {
		batch(() => {
			setState('error');
			setConnectionError(event.reason);
		});
	});

	onCleanup(() => {
		socket.close();
	});

	return (
		<>
			<Nav />
			<Switch>
				<Match when={state() === 'connecting'}>
					<Connecting />
				</Match>
				<Match when={state() === 'error'}>
					<Errored errorMessage={connectionError} />
				</Match>
				<Match when={state() === 'lobby' || state() === 'game'}>
					<Switch>
						<Match when={state() === 'lobby'}>
							<Lobby sendMessage={sendMessage} />
						</Match>
						<Match when={state() === 'game'}>
							<InGame sendMessage={sendMessage} />
						</Match>
					</Switch>
					<ChatMessages sendMessage={sendMessage} />
				</Match>
			</Switch>
		</>
	);
};

export default Game;
