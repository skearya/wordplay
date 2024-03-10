<script lang="ts">
	import { PUBLIC_SERVER } from '$env/static/public';
	import { gameInfo, gameState } from '$lib/state';
	import { inGameOrLobby, type AppState, type ServerMessage, type ClientMessage } from '$lib/types';
	import { tick } from 'svelte';
	import { writable } from 'svelte/store';
	import { match } from 'ts-pattern';

	let chatInput: string = '';
	let gameInputNode: HTMLInputElement;
	let gameInput = writable('');

	const room = window.prompt('room', 'one');
	const username = window.prompt('username', 'skeary');

	let params = new URLSearchParams({ username: username! });

	let rejoinToken = localStorage.getItem(room!);
	if (rejoinToken !== null) {
		params.append('rejoinToken', rejoinToken);
	}

	const socket = new WebSocket(`${PUBLIC_SERVER}/rooms/${room}?${params.toString()}`);

	socket.addEventListener('message', (event) => {
		const message: ServerMessage = JSON.parse(event.data);

		$gameState = match([$gameState, message])
			.returnType<AppState>()
			.with(
				[{ type: 'connecting' }, { type: 'roomInfo', state: { type: 'lobby' } }],
				([_state, message]) => {
					return {
						type: 'lobby',
						uuid: message.uuid,
						readyPlayers: message.state.ready,
						chatMessages: [],
						previousWinner: null,
						countdown: null
					};
				}
			)
			.with(
				[{ type: 'connecting' }, { type: 'roomInfo', state: { type: 'inGame' } }],
				([_state, message]) => {
					const playerData = message.state.players.find((player) => message.uuid === player.uuid);

					if (playerData !== undefined) {
						$gameInput = playerData.input;
					}

					return {
						type: 'game',
						players: message.state.players,
						currentTurn: message.state.turn,
						prompt: message.state.prompt,
						usedLetters: new Set(message.state.usedLetters || []),
						uuid: message.uuid,
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
				if (message.players.length >= 2) {
					state.countdown = 10;
				}

				return {
					...state,
					readyPlayers: message.players
				};
			})
			.with([{ type: 'lobby' }, { type: 'startingCountdown' }], ([state, message]) => {
				return {
					...state,
					countdown: message.state.type === 'inProgress' ? message.state.timeLeft : null
				};
			})
			.with([{ type: 'lobby' }, { type: 'gameStarted' }], ([state, message]) => {
				$gameInput = '';
				localStorage.setItem(room!, message.rejoinToken);

				return {
					type: 'game',
					players: message.players,
					prompt: message.prompt,
					currentTurn: message.turn,
					usedLetters: new Set(),
					chatMessages: state.chatMessages,
					uuid: state.uuid
				};
			})
			.with([{ type: 'game' }, { type: 'playerUpdate' }], ([state, message]) => {
				let player = state.players.find((player) => player.uuid === message.uuid)!;

				if (message.state.type === 'reconnected') {
					player.username = message.state.username;
					player.disconnected = false;
				} else {
					player.disconnected = true;
				}

				return state;
			})
			.with([{ type: 'game' }, { type: 'inputUpdate' }], ([state, message]) => {
				state.players.find((player) => player.uuid === message.uuid)!.input = message.input;

				return state;
			})
			.with([{ type: 'game' }, { type: 'invalidWord' }], ([state, message]) => {
				return state;
			})
			.with([{ type: 'game' }, { type: 'newPrompt' }], ([state, message]) => {
				let oldTurn = state.players.find((player) => player.uuid === state.currentTurn)!;
				oldTurn.lives += message.lifeChange;

				if (oldTurn.uuid === state.uuid && message.lifeChange >= 0) {
					state.usedLetters = new Set([...state.usedLetters, ...$gameInput]);
				}

				if (state.uuid === message.turn) {
					$gameInput = '';

					tick().then(() => {
						gameInputNode.focus();
					});
				}

				return {
					...state,
					prompt: message.prompt,
					currentTurn: message.turn
				};
			})
			.with([{ type: 'game' }, { type: 'gameEnded' }], ([state, message]) => {
				chatInput = '';

				return {
					type: 'lobby',
					readyPlayers: [],
					chatMessages: state.chatMessages,
					uuid: state.uuid,
					previousWinner: state.players.find((player) => player.uuid === message.winner)!.username,
					countdown: null
				};
			})
			.otherwise(() => $gameState);
	});

	socket.addEventListener('close', (event) => {
		$gameState = {
			type: 'error',
			message: event.reason
		};
	});

	function sendMessage(message: ClientMessage) {
		socket.send(JSON.stringify(message));
	}

	gameInput.subscribe((input) => {
		if ($gameState.type === 'game') sendMessage({ type: 'input', input });
	});
</script>

<section>
	{#if $gameState.type === 'connecting'}
		<h1>connecting!</h1>
	{:else if $gameState.type === 'error'}
		<h1>we errored: {$gameState.message}</h1>
	{:else if $gameState.type === 'lobby'}
		{#if $gameState.previousWinner !== null}
			<h1 class="text-green-400">winner!!: {$gameState.previousWinner}</h1>
		{/if}
		<h1 class="text-2xl">room: {room}</h1>
		<div class="flex gap-2">
			<h1>ready players:</h1>
			{#each $gameState.readyPlayers as player}
				<h1>
					{player.username}
				</h1>
			{/each}
		</div>
		{#if $gameState.countdown !== null}
			<h1 class="text-red-300">starting in {$gameState.countdown} !!</h1>
		{/if}
		<ul class="list-item">
			{#each $gameState.chatMessages as message}
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
				{$gameInfo?.currentTurn.username}
			</h1>
		</div>
		<h1>{$gameState.prompt}</h1>
		<div class="flex gap-2">
			<h1>players:</h1>
			{#each $gameState.players as player}
				<div class:animate-pulse={player.disconnected}>
					<h1>{player.username}</h1>
					<h1>{player.uuid}</h1>
					<h1 id={`input-${player.uuid}`} class="min-w-16">input: {player.input}</h1>
					<h1 class="text-red-400">lives: {player.lives}</h1>
					{#if player.disconnected}
						<h1>i disconnected..</h1>
					{/if}
				</div>
			{/each}
		</div>
		<div class="flex gap-1">
			<h1>unused letters</h1>
			{#each $gameInfo?.unusedLetters || '' as letter}
				<h1>{letter}</h1>
			{/each}
		</div>
		<input
			class="border bg-green-200 disabled:bg-red-400"
			type="text"
			placeholder="answer"
			disabled={!$gameInfo?.hasTurn}
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
			{#each $gameState.chatMessages as message}
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
