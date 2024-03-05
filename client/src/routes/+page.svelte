<script lang="ts">
	import { PUBLIC_SERVER } from '$env/static/public';
	import { inGameOrLobby, type AppState, type ServerMessage, type ClientMessage } from '$lib/types';
	import { tick } from 'svelte';
	import { writable } from 'svelte/store';
	import { match } from 'ts-pattern';

	const alphabet = [...'abcdefghijklmnopqrstuvwy'];

	let chatInput: string = '';
	let gameInputNode: HTMLInputElement;
	let gameInput = writable('');

	let countdownId: number;
	let timeLeft: number | undefined = undefined;

	let state: AppState = {
		type: 'connecting'
	};

	let winner: string;

	const room = window.prompt('room', 'one');
	const username = window.prompt('username', 'skeary');

	const params = new URLSearchParams({ username: username! }).toString();
	const socket = new WebSocket(`${PUBLIC_SERVER}/rooms/${room}?${params}`);

	socket.addEventListener('message', (event) => {
		const message: ServerMessage = JSON.parse(event.data);

		state = match([state, message])
			.returnType<AppState>()
			.with(
				[{ type: 'connecting' }, { type: 'roomInfo', state: { type: 'lobby' } }],
				([_state, { state: roomState, uuid }]) => {
					return {
						type: 'lobby',
						uuid: uuid,
						readyPlayers: roomState.readyPlayers,
						chatMessages: []
					};
				}
			)
			.with(
				[{ type: 'connecting' }, { type: 'roomInfo', state: { type: 'inGame' } }],
				([_state, { state: roomState, uuid }]) => {
					// for reconnecting prob wanna do this
					// gameInput = roomState.players.find((player) => player.uuid = uuid)!.input
					// same for usedLetters
					// this data wont be found if the player joins a game and they weren't in it

					return {
						type: 'game',
						players: roomState.players,
						currentTurn: roomState.players.find((player) => player.uuid == roomState.turn)!,
						prompt: roomState.prompt,
						usedLetters: new Set(),
						uuid,
						chatMessages: []
					};
				}
			)
			.with(
				[{ type: inGameOrLobby }, { type: 'serverMessage' }],
				[{ type: inGameOrLobby }, { type: 'chatMessage' }],
				([state, message]) => {
					const messageContent =
						message.type === 'serverMessage'
							? `server: ${message.content}`
							: `${message.author}: ${message.content}`;

					return {
						...state,
						chatMessages: [...state.chatMessages, messageContent]
					};
				}
			)
			.with([{ type: 'lobby' }, { type: 'readyPlayers' }], ([state, message]) => {
				if (message.countdown) {
					timeLeft = 10;

					clearInterval(countdownId);
					countdownId = setInterval(() => {
						timeLeft! -= 1;

						if (timeLeft == 0) {
							clearInterval(countdownId);
						}
					}, 1000);
				}

				return {
					...state,
					readyPlayers: message.players
				};
			})
			.with([{ type: 'lobby' }, { type: 'gameStarted' }], ([state, message]) => {
				$gameInput = '';

				return {
					type: 'game',
					players: message.players,
					prompt: message.prompt,
					currentTurn: message.players.find((player) => player.uuid == message.turn)!,
					usedLetters: new Set(),
					chatMessages: state.chatMessages,
					uuid: state.uuid
				};
			})
			.with([{ type: 'game' }, { type: 'inputUpdate' }], ([state, message]) => {
				state.players.find((player) => player.uuid === message.uuid)!.input = message.input;

				return {
					...state
				};
			})
			// .with([{ type: 'game' }, { type: 'invalidWord' }], ([state, message]) => {})
			.with([{ type: 'game' }, { type: 'newPrompt' }], ([state, message]) => {
				let oldTurn = state.players.find((player) => player.uuid === state.currentTurn.uuid)!;
				oldTurn.lives += message.lifeChange;

				if (oldTurn.uuid === state.uuid && message.lifeChange >= 0) {
					state.usedLetters = new Set([...state.usedLetters, ...$gameInput]);
				}

				return {
					...state,
					prompt: message.prompt,
					currentTurn: state.players.find((player) => player.uuid == message.turn)!
				};
			})
			.with([{ type: 'game' }, { type: 'gameEnded' }], ([state, message]) => {
				chatInput = '';
				timeLeft = undefined;
				winner = state.players.find((player) => player.uuid === message.winner)!.username;

				return {
					type: 'lobby',
					readyPlayers: [],
					chatMessages: state.chatMessages,
					uuid: state.uuid
				};
			})
			.otherwise(() => state);
	});

	socket.addEventListener('close', (event) => {
		state = {
			type: 'error',
			message: event.reason
		};
	});

	function sendMessage(message: ClientMessage) {
		socket.send(JSON.stringify(message));
	}

	gameInput.subscribe((input) => {
		if (state.type === 'game') sendMessage({ type: 'input', input });
	});

	$: hasTurn = match(state)
		.with({ type: 'game' }, (state) => state.uuid === state.currentTurn.uuid)
		.otherwise(() => false);

	$: if (hasTurn) {
		$gameInput = '';

		tick().then(() => {
			gameInputNode.focus();
		});
	}

	$: unusedLetters = match(state)
		.with({ type: 'game' }, (state) => alphabet.filter((letter) => !state.usedLetters.has(letter)))
		.otherwise(() => []);
</script>

<section>
	{#if state.type === 'connecting'}
		<h1>connecting!</h1>
	{:else if state.type === 'error'}
		<h1>we errored: {state.message}</h1>
	{:else if state.type === 'lobby'}
		{#if winner}
			<h1 class="text-green-400">winner!!: {winner}</h1>
		{/if}
		<h1 class="text-2xl">room: {room}</h1>
		<div class="flex gap-2">
			<h1>ready players:</h1>
			{#each state.readyPlayers as player}
				<h1>
					{player.username}
				</h1>
			{/each}
		</div>
		{#if timeLeft !== undefined}
			<h1 class="text-red-300">starting in {timeLeft} !!</h1>
		{/if}
		<ul class="list-item">
			{#each state.chatMessages as message}
				<li>{message}</li>
			{/each}
		</ul>
		<input
			class="border"
			type="text"
			placeholder="chat"
			bind:value={chatInput}
			on:keydown={(event) => {
				if (event.key == 'Enter') {
					sendMessage({
						type: 'chatMessage',
						content: chatInput
					});

					chatInput = '';
				}
			}}
		/>
		<button
			on:click={() => {
				sendMessage({
					type: 'ready'
				});
			}}>ready</button
		>
	{:else}
		<div class="flex gap-2">
			<h1>turn:</h1>
			<h1 class="text-green-500">
				{state.currentTurn.username}
			</h1>
		</div>
		<h1>{state.prompt}</h1>
		<div class="flex gap-2">
			<h1>players:</h1>
			{#each state.players as player}
				<div>
					<h1>{player.username}</h1>
					<h1 class="min-w-16">input: {player.input}</h1>
					<h1 class="text-red-400">lives: {player.lives}</h1>
				</div>
			{/each}
		</div>
		<div class="flex gap-1">
			<h1>unused letters</h1>
			{#each unusedLetters as letter}
				<h1>{letter}</h1>
			{/each}
		</div>
		<input
			class="border bg-green-200 disabled:bg-red-400"
			type="text"
			placeholder="answer"
			disabled={!hasTurn}
			bind:this={gameInputNode}
			bind:value={$gameInput}
			on:keydown={(event) => {
				if (event.key === 'Enter') {
					sendMessage({
						type: 'guess',
						word: $gameInput
					});
				}
			}}
		/>
		<ul>
			{#each state.chatMessages as message}
				<li>{message}</li>
			{/each}
		</ul>
		<input
			class="border"
			type="text"
			placeholder="chat"
			bind:value={chatInput}
			on:keydown={(event) => {
				if (event.key == 'Enter') {
					sendMessage({
						type: 'chatMessage',
						content: chatInput
					});

					chatInput = '';
				}
			}}
		/>
	{/if}
</section>
