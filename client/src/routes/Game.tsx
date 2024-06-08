import {
	Match,
	Show,
	Switch,
	batch,
	createSignal,
	onCleanup,
	onMount,
	useContext,
	type Component
} from 'solid-js';
import { produce } from 'solid-js/store';
import { P, match } from 'ts-pattern';
import { Context } from '../lib/context';
import { createAnagrams } from '../lib/game/Anagrams';
import { ChatMessages } from '../lib/game/ChatMessages';
import { Connecting } from '../lib/game/Connecting';
import { Errored } from '../lib/game/Errored';
import { Lobby } from '../lib/game/Lobby';
import { Nav } from '../lib/game/Nav';
import { createWordBomb } from '../lib/game/WordBomb';
import type { ClientMessage, ServerMessage } from '../lib/types/messages';

const JoinGame: Component = () => {
	let usernameInputRef!: HTMLInputElement;

	const [username, setUsername] = createSignal(localStorage.getItem('username') ?? '');
	const [ready, setReady] = createSignal(false);
	const canJoin = () => username().length <= 20 && username() !== '';

	function join() {
		localStorage.setItem('username', username());
		setReady(true);
	}

	onMount(() => {
		usernameInputRef.focus();
	});

	return (
		<>
			<Nav />
			<Show
				when={ready()}
				fallback={
					<section class="flex min-h-screen flex-col items-center justify-center gap-3">
						<h1>choose a username</h1>
						<div class="flex gap-3">
							<input
								ref={usernameInputRef}
								type="text"
								class="rounded-md border bg-background px-3 py-2 text-sm"
								maxlength="20"
								placeholder="name"
								value={username()}
								onInput={(event) => setUsername(event.target.value)}
								onKeyDown={(event) => {
									if (event.key === 'Enter' && canJoin()) {
										join();
									}
								}}
							/>
							<button
								class="rounded-md border bg-secondary px-3 transition-opacity disabled:opacity-50"
								disabled={!canJoin()}
								onClick={join}
							>
								join
							</button>
						</div>
					</section>
				}
			>
				<Game username={username()} />
			</Show>
		</>
	);
};

const Game: Component<{ username: string }> = (props) => {
	const context = useContext(Context);
	if (!context) throw new Error('Not called inside context provider?');
	const { connection, state } = context[0];
	const { setConnection, setState, setLobby, setWordBomb, setAnagrams } = context[1];

	const [connectionError, setConnectionError] = createSignal<string | null>(null);

	const rejoinToken: string | undefined = JSON.parse(localStorage.getItem('rejoinTokens') ?? '{}')[
		connection.room
	];

	const params = new URLSearchParams({
		username: props.username,
		...(rejoinToken && { rejoin_token: rejoinToken })
	});

	const socket = new WebSocket(
		`${(import.meta.env.PUBLIC_SERVER as string).replace('http', 'ws')}/rooms/${connection.room}?${params.toString()}`
	);

	const sender = (data: ClientMessage) => socket.send(JSON.stringify(data));

	const { onWordBombGuess, WordBomb } = createWordBomb({ sender });
	const { onAnagramsGuess, Anagrams } = createAnagrams({ sender });

	socket.addEventListener('message', (event) => {
		const message: ServerMessage = JSON.parse(event.data);

		match(message)
			.with({ type: 'Info' }, (message) => {
				const { uuid, room } = message;

				setConnection({
					uuid: uuid,
					clients: room.clients,
					roomOwner: room.owner,
					settings: room.settings
				});

				batch(() => {
					setState(room.state.type);

					if (room.state.type === 'Lobby') {
						const { ready, starting_countdown } = room.state;

						setLobby({
							ready,
							startingCountdown: starting_countdown
						});
					} else if (room.state.type === 'WordBomb') {
						const { players, turn, prompt, used_letters } = room.state;

						const usedLetters = used_letters ? new Set(used_letters) : null;
						const input = players.find((player) => player.uuid === connection.uuid)?.input ?? '';

						setWordBomb({
							players,
							turn,
							prompt,
							usedLetters,
							input
						});
					} else {
						const { players, anagram } = room.state;

						setAnagrams({
							players,
							anagram
						});
					}
				});
			})
			.with({ type: 'RoomSettings' }, (message) => {
				const { type, ...settings } = message;

				setConnection('settings', settings);
			})
			.with({ type: P.union('ChatMessage', 'Error') }, (message) => {
				if (message.type === 'Error') {
					setConnection('chatMessages', connection.chatMessages.length, message.content);
				} else {
					setConnection('chatMessages', connection.chatMessages.length, [
						connection.clients.find((client) => client.uuid === message.author)!.username,
						message.content
					]);
				}
			})
			.with({ type: 'ConnectionUpdate' }, (message) => {
				const messageContent =
					message.state.type === 'Connected' || message.state.type === 'Reconnected'
						? `${message.state.username} has joined`
						: `${connection.clients.find((client) => client.uuid === message.uuid)!.username} has left`;

				setConnection('chatMessages', connection.chatMessages.length, messageContent);

				if (message.state.type === 'Connected' || message.state.type === 'Reconnected') {
					const newClient = {
						uuid: message.uuid,
						username: message.state.username,
						disconnected: false
					};

					setConnection('clients', (clients) => [
						...clients.filter((client) => client.uuid !== message.uuid),
						newClient
					]);
				} else {
					if (message.state.new_room_owner) {
						setConnection('roomOwner', message.state.new_room_owner);
					}

					if (state() !== 'Lobby') {
						setConnection('clients', (client) => client.uuid == message.uuid, 'disconnected', true);
					} else {
						setConnection('clients', (clients) =>
							clients.filter((client) => client.uuid !== message.uuid)
						);
					}
				}
			})
			.with({ type: 'ReadyPlayers' }, (message) => {
				setLobby('ready', message.ready);

				if (message.countdown_update) {
					if (message.countdown_update.type === 'InProgress') {
						setLobby('startingCountdown', message.countdown_update.time_left);
					} else {
						setLobby('startingCountdown', null);
					}
				}
			})
			.with({ type: 'StartingCountdown' }, (message) => {
				setLobby('startingCountdown', message.time_left);
			})
			.with({ type: 'GameStarted' }, (message) => {
				if (message.rejoin_token) {
					let tokens = JSON.parse(localStorage.getItem('rejoinTokens') ?? '{}');
					tokens[connection.room] = message.rejoin_token;
					localStorage.setItem('rejoinTokens', JSON.stringify(tokens));
				}

				batch(() => {
					setState(message.game.type);

					if (message.game.type === 'WordBomb') {
						const { players, turn, prompt } = message.game;

						setWordBomb({
							players,
							turn,
							prompt,
							usedLetters: new Set(),
							input: ''
						});
					} else {
						const { players, anagram } = message.game;

						setAnagrams({
							players,
							anagram
						});
					}
				});
			})
			.with({ type: 'GameEnded' }, (message) => {
				batch(() => {
					if (message.new_room_owner) {
						setConnection('roomOwner', message.new_room_owner);
					}

					setState('Lobby');
					setLobby({
						ready: [],
						startingCountdown: null,
						postGame: message.info
					});
					setConnection('clients', (clients) => clients.filter((client) => !client.disconnected));
				});
			})
			.with({ type: 'WordBombInput' }, (message) => {
				setWordBomb('players', (player) => player.uuid === message.uuid, 'input', message.input);
			})
			.with({ type: 'WordBombInvalidGuess' }, (message) => {
				onWordBombGuess(message.uuid, {
					type: 'invalid',
					reason: message.reason.type
				});
			})
			.with({ type: 'WordBombPrompt' }, (message) => {
				setWordBomb(
					produce((game) => {
						onWordBombGuess(
							game.turn,
							message.correct_guess ? { type: 'correct' } : { type: 'invalid', reason: 'timed out' }
						);

						const turn = game.players.find((player) => player.uuid === game.turn)!;
						turn.lives += message.life_change;

						if (turn.uuid === connection.uuid && message.correct_guess) {
							game.usedLetters = new Set([...game.usedLetters!, ...message.correct_guess]);
						}
						if (message.turn === connection.uuid) {
							game.input = '';
						}

						game.prompt = message.prompt;
						game.turn = message.turn;
					})
				);
			})
			.with({ type: 'AnagramsCorrectGuess' }, (message) => {
				if (message.uuid == connection.uuid) {
					onAnagramsGuess({ type: 'correct' });
				}

				setAnagrams(
					'players',
					(player) => player.uuid === message.uuid,
					'used_words',
					(words) => [...words, message.guess]
				);
			})
			.with({ type: 'AnagramsInvalidGuess' }, (message) => {
				const reason = match(message.reason.type)
					.with('NotLongEnough', () => 'not long enough')
					.with('PromptMismatch', () => 'prompt mismatch')
					.with('NotEnglish', () => 'not english')
					.with('AlreadyUsed', () => 'already used')
					.exhaustive();

				onAnagramsGuess({ type: 'invalid', reason });
			})
			.exhaustive();
	});

	socket.addEventListener('close', (event) => {
		batch(() => {
			setState('Error');
			setConnectionError(event.reason);
		});
	});

	onCleanup(() => {
		socket.close();
	});

	return (
		<>
			<Switch>
				<Match when={state() === 'Connecting'}>
					<Connecting />
				</Match>
				<Match when={state() === 'Error'}>
					<Errored errorMessage={connectionError()} />
				</Match>
				<Match when={state() === 'Lobby'}>
					<Lobby sender={sender} />
				</Match>
				<Match when={state() === 'WordBomb'}>
					<WordBomb />
				</Match>
				<Match when={state() === 'Anagrams'}>
					<Anagrams />
				</Match>
			</Switch>
			<Show when={state() !== 'Connecting' || state() !== 'Error'}>
				<ChatMessages sender={sender} />
			</Show>
		</>
	);
};

export default JoinGame;
