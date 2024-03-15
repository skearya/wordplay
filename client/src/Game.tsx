import type { Component } from 'solid-js';
import type { ClientMessage, ServerMessage } from './types/messages';
import { Switch, Match, For, onCleanup, createEffect, Show, batch, useContext } from 'solid-js';
import { produce } from 'solid-js/store';
import { P, match } from 'ts-pattern';
import { Context } from './context';

const Game: Component = () => {
	let gameInputRef!: HTMLInputElement;

	const [context, setContext] = useContext(Context);
	const { connection, game, lobby, state } = context;
	const { setConnection, setGame, setLobby, setState } = setContext;

	const unusedLetters = () =>
		[...'abcdefghijklmnopqrstuvwy'].filter((letter) => !game.usedLetters?.has(letter));

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

	socket.addEventListener('message', (event) => {
		const message: ServerMessage = JSON.parse(event.data);

		match(message)
			.with({ type: 'roomInfo' }, (message) => {
				setConnection('uuid', message.uuid);

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
							: undefined;
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
						: `${message.author}: ${message.content}`;

				setConnection('chatMessages', connection.chatMessages.length, messageContent);
			})
			.with({ type: 'readyPlayers' }, (message) => {
				setLobby({
					readyPlayers: message.players,
					startingCountdown: message.players.length >= 2 ? 10 : null
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
						usedLetters: new Set(),
						input: ''
					});
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
				setGame(
					produce((game) => {
						let turn = game.players.find((player) => player.uuid === game.currentTurn)!;
						turn.lives += message.lifeChange;

						if (turn.uuid === connection.uuid && message.lifeChange >= 0) {
							// technically game.input could be modified after word submission so this would be incorrect
							// probably just send used letters from server
							game.usedLetters = new Set([...game.usedLetters!, ...game.input]);
						}

						if (message.turn == connection.uuid) {
							game.input = '';
							gameInputRef.focus();
						}

						game.prompt = message.prompt;
						game.currentTurn = message.turn;
					})
				);
			})
			.with({ type: 'gameEnded' }, (message) => {
				batch(() => {
					setState('lobby');
					setLobby({
						previousWinner: game.players.find((player) => player.uuid === message.winner)!.username,
						readyPlayers: [],
						startingCountdown: null
					});
				});
			})
			.exhaustive();
	});

	socket.addEventListener('close', (event) => {
		setState('error');
	});

	function sendMessage(data: ClientMessage) {
		socket.send(JSON.stringify(data));
	}

	createEffect(() => {
		if (state() === 'game' && game.players.map((player) => player.uuid).includes(connection.uuid)) {
			sendMessage({
				type: 'input',
				input: game.input
			});
		}
	});

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
				{lobby.startingCountdown && <h1>starting soon: {lobby.startingCountdown}</h1>}
				{lobby.previousWinner && <h1>winner: {lobby.previousWinner}</h1>}
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
				<div class="flex gap-2">
					<h1>turn</h1>
					<h1 class="text-green-300">
						{game.players.find((player) => player.uuid === game.currentTurn)!.username}
					</h1>
				</div>
				<h1>{game.prompt}</h1>
				<div class="flex gap-2">
					<h1>players:</h1>
					<For each={game.players}>
						{(player, _) => (
							<div>
								<h1>{player.username}</h1>
								<h1>{player.uuid}</h1>
								<h1 class="min-w-16">input: {player.input}</h1>
								<h1 class="text-red-400">lives: {player.lives}</h1>
								<Show when={player.disconnected}>
									<h1>i disconnected...</h1>
								</Show>
							</div>
						)}
					</For>
				</div>
				<div class="flex gap-1">
					<h1>unused letters</h1>
					<For each={unusedLetters()}>{(letter, _) => <h1>{letter}</h1>}</For>
				</div>
				<input
					ref={gameInputRef}
					class="border"
					type="text"
					disabled={game.currentTurn !== connection.uuid}
					value={game.input}
					onInput={(event) => setGame('input', event.target.value)}
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

export { Game };
