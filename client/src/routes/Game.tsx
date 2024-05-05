import type { Component } from 'solid-js';
import type { ClientMessage, ServerMessage } from '../lib/types/messages';
import { Switch, Match, onCleanup, batch, useContext, createSignal } from 'solid-js';
import { produce, reconcile } from 'solid-js/store';
import { P, match } from 'ts-pattern';
import { Context } from '../lib/context';
import { Nav } from '../lib/game/Nav';
import { Connecting } from '../lib/game/Connecting';
import { Errored } from '../lib/game/Errored';
import { ChatMessages } from '../lib/game/ChatMessages';
import { Lobby } from '../lib/game/Lobby';
import { WordBomb } from '../lib/game/WordBomb';
import { Anagrams } from '../lib/game/Anagrams';

const Game: Component = () => {
	const context = useContext(Context);
	if (!context) throw new Error('Not called inside context provider?');
	const { connection, state } = context[0];
	const { setConnection, setState, setLobby, setWordBomb, setAnagrams } = context[1];

	const [connectionError, setConnectionError] = createSignal<string | null>(null);

	const rejoinToken: string | undefined = JSON.parse(localStorage.getItem('rejoinTokens') ?? '{}')[
		connection.room
	];

	const params = new URLSearchParams({
		username: connection.username,
		...(rejoinToken && { rejoinToken })
	});

	const socket = new WebSocket(
		`${(import.meta.env.PUBLIC_SERVER as string).replace('http', 'ws')}/rooms/${connection.room}?${params.toString()}`
	);

	const sender = (data: ClientMessage) => socket.send(JSON.stringify(data));

	socket.addEventListener('message', (event) => {
		const message: ServerMessage = JSON.parse(event.data);

		match(message)
			.with({ type: 'info' }, (message) => {
				const { uuid, room } = message;

				setConnection({
					uuid: uuid,
					clients: room.clients,
					roomOwner: room.owner,
					settings: room.settings
				});

				batch(() => {
					setState(room.state.type);

					if (room.state.type === 'lobby') {
						setLobby({
							readyPlayers: room.state.ready,
							startingCountdown: room.state.startingCountdown
						});
					} else if (room.state.type === 'wordBomb') {
						const usedLetters = room.state.usedLetters ? new Set(room.state.usedLetters) : null;
						const input =
							room.state.players.find((player) => player.uuid === connection.uuid)?.input ?? '';

						setWordBomb({
							players: room.state.players,
							currentTurn: room.state.turn,
							prompt: room.state.prompt,
							usedLetters,
							input
						});
					} else {
						setAnagrams({
							players: room.state.players,
							prompt: room.state.prompt
						});
					}
				});
			})
			.with({ type: 'roomSettings' }, (message) => {
				const { type, ...settings } = message;

				setConnection('settings', reconcile(settings));
			})
			.with({ type: P.union('chatMessage', 'error') }, (message) => {
				const messageContent =
					message.type === 'error'
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
					setWordBomb('players', (player) => player.uuid === message.uuid, {
						username: message.state.username,
						disconnected: false
					});
				} else if (message.state.type === 'disconnected') {
					setWordBomb('players', (player) => player.uuid === message.uuid, 'disconnected', true);
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
					setState(message.game.type);

					if (message.game.type === 'wordBomb') {
						setWordBomb({
							players: message.game.players,
							currentTurn: message.game.turn,
							prompt: message.game.prompt,
							guessError: ['', ''],
							usedLetters: new Set(),
							input: ''
						});
					} else {
						setAnagrams({
							players: message.game.players,
							prompt: message.game.prompt
						});
					}
				});
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
					setAnagrams('guessError', ['']);
				});
			})
			.with({ type: 'wordBombInput' }, (message) => {
				setWordBomb('players', (player) => player.uuid === message.uuid, 'input', message.input);
			})
			.with({ type: 'wordBombInvalidGuess' }, (message) => {
				setWordBomb('guessError', [message.uuid, message.reason.type]);
			})
			.with({ type: 'wordBombPrompt' }, (message) => {
				setWordBomb(
					produce((game) => {
						let turn = game.players.find((player) => player.uuid === game.currentTurn)!;
						turn.lives += message.lifeChange;

						if (turn.uuid === connection.uuid && message.correctGuess) {
							game.usedLetters = new Set([...game.usedLetters!, ...message.correctGuess]);
						}
						if (message.turn === connection.uuid) {
							game.input = '';
						}

						game.prompt = message.prompt;
						game.currentTurn = message.turn;
					})
				);
			})
			.with({ type: 'anagramsCorrectGuess' }, (message) => {
				if (message.uuid == connection.uuid) {
					setAnagrams('input', '');
				}

				setAnagrams(
					'players',
					(player) => player.uuid === message.uuid,
					'usedWords',
					(words) => [...words, message.guess]
				);
			})
			.with({ type: 'anagramsInvalidGuess' }, (message) => {
				setAnagrams('guessError', [message.reason.type]);
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
				<Match when={state() !== 'connecting' || state() !== 'error'}>
					<Switch>
						<Match when={state() === 'lobby'}>
							<Lobby sender={sender} />
						</Match>
						<Match when={state() === 'wordBomb'}>
							<WordBomb sender={sender} />
						</Match>
						<Match when={state() === 'anagrams'}>
							<Anagrams sender={sender} />
						</Match>
					</Switch>
					<ChatMessages sender={sender} />
				</Match>
			</Switch>
		</>
	);
};

export default Game;
