<script lang="ts">
	import { inGameOrLobby, type AppState, type ServerMessage, type ClientMessage } from '$lib/types';
	import { PUBLIC_SERVER } from '$env/static/public';
	import { match } from 'ts-pattern';

	let chatInput: string = '';
	let gameInput: string = '';

	let state: AppState = {
		type: 'connecting'
	};

	let winner: string;

	const room = window.prompt('room', 'one');
	const username = window.prompt('username', 'name');

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
					return {
						type: 'game',
						uuid,
						players: roomState.players,
						prompt: roomState.prompt,
						currentTurn: roomState.players.find((player) => player.uuid == roomState.turn)!,
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
				return {
					...state,
					readyPlayers: message.players
				};
			})
			.with([{ type: 'lobby' }, { type: 'gameStarted' }], ([state, message]) => {
				gameInput = '';

				return {
					type: 'game',
					players: message.players,
					prompt: message.prompt,
					currentTurn: message.players.find((player) => player.uuid == message.turn)!,
					chatMessages: state.chatMessages,
					uuid: state.uuid
				};
			})
			.with([{ type: 'game' }, { type: 'newPrompt' }], ([state, message]) => {
				if (message.timedOut) {
					state.players.find((player) => player.uuid == state.currentTurn.uuid)!.lives -= 1;
				}

				return {
					...state,
					prompt: message.prompt,
					currentTurn: state.players.find((player) => player.uuid == message.turn)!
				};
			})
			// .with([{ type: 'game' }, { type: 'invalidWord' }], ([state, message]) => {})
			.with([{ type: 'game' }, { type: 'gameEnded' }], ([state, message]) => {
				chatInput = '';
				winner = state.players.find((player) => player.uuid == message.winner)!.username;

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
		<ul class="list-item">
			{#each state.chatMessages as message}
				<li>{message}</li>
			{/each}
		</ul>
		<input
			class="border"
			type="text"
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
					<h1 class="text-red-400">lives: {player.lives}</h1>
				</div>
			{/each}
		</div>
		<input
			class="border"
			type="text"
			bind:value={gameInput}
			on:keydown={(event) => {
				if (event.key === 'Enter') {
					sendMessage({
						type: 'guess',
						word: gameInput
					});

					gameInput = '';
				}
			}}
		/>
		<ul>
			{#each state.chatMessages as message}
				<li>{message}</li>
			{/each}
		</ul>
	{/if}
</section>
