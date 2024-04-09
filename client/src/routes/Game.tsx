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
	const { connection, state } = context[0];
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
				const { uuid, room } = message;

				setConnection({
					uuid: uuid,
					clients: room.clients,
					roomOwner: room.owner
				});

				batch(() => {
					if (room.state.type === 'lobby') {
						setState('lobby');
						setLobby({
							readyPlayers: room.state.ready,
							startingCountdown: room.state.startingCountdown
						});
					} else {
						const usedLetters = room.state.usedLetters ? new Set(room.state.usedLetters) : null;
						const input =
							room.state.players.find((player) => player.uuid === connection.uuid)?.input ?? '';

						setState('game');
						setGame({
							players: room.state.players,
							currentTurn: room.state.turn,
							prompt: room.state.prompt,
							usedLetters,
							input
						});
					}
				});
			})
			.with({ type: 'gameSettings' }, (message) => {
				setConnection('public', message.public);
			})
			.with({ type: P.union('serverMessage', 'chatMessage') }, (message) => {
				const messageContent =
					message.type === 'serverMessage'
						? `server: ${message.content}`
						: `${connection.clients.find((client) => client.uuid === message.author)!.username}: ${message.content}`;

				setConnection('chatMessages', connection.chatMessages.length, messageContent);
			})
			.with({ type: 'connectionUpdate' }, (message) => {
				const messageContent =
					message.state.type === 'connected' || message.state.type === 'reconnected'
						? `${message.state.username} has joined`
						: `${connection.clients.find((client) => client.uuid === message.uuid)!.username} has left`;

				setConnection('chatMessages', connection.chatMessages.length, messageContent);

				if (message.state.type === 'connected' || message.state.type === 'reconnected') {
					setConnection('clients', (clients) => [
						...clients.filter((client) => client.uuid !== message.uuid)
					]);
					setConnection('clients', connection.clients.length, {
						uuid: message.uuid,
						username: message.state.username
					});
				} else {
					if (message.state.newRoomOwner) {
						setConnection('roomOwner', message.state.newRoomOwner);
					}
					setConnection('clients', (clients) =>
						clients.filter((client) => client.uuid != message.uuid)
					);
				}

				if (message.state.type === 'reconnected') {
					setGame('players', (player) => player.uuid === message.uuid, {
						username: message.state.username,
						disconnected: false
					});
				} else if (message.state.type === 'disconnected') {
					setGame('players', (player) => player.uuid === message.uuid, 'disconnected', true);
				}
			})
			.with({ type: 'readyPlayers' }, (message) => {
				setLobby('readyPlayers', message.ready);

				if (message.countdownUpdate) {
					if (message.countdownUpdate.type === 'inProgress') {
						setLobby('startingCountdown', message.countdownUpdate.timeLeft);
					} else {
						setLobby('startingCountdown', null);
					}
				}
			})
			.with({ type: 'startingCountdown' }, (message) => {
				setLobby('startingCountdown', message.timeLeft);
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
					if (message.newRoomOwner) {
						setConnection('roomOwner', message.newRoomOwner);
					}

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
