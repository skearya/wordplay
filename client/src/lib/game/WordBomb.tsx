import { useContext, For, Show, createEffect } from 'solid-js';
import { Context } from '../context';
import { ClientMessage, Uuid } from '../types/messages';

const createWordBomb = (props: { sender: (message: ClientMessage) => void }) => {
	const context = useContext(Context);
	if (!context) throw new Error('Not called inside context provider?');
	const { connection, wordBomb: game } = context[0];
	const { setWordBomb } = context[1];

	let gameInputRef!: HTMLInputElement;

	const ourTurn = () => game.currentTurn === connection.uuid;
	const unusedLetters = () =>
		[...'abcdefghijklmnopqrstuvwy'].filter((letter) => !game.usedLetters?.has(letter));

	createEffect(() => {
		if (ourTurn() && gameInputRef) {
			gameInputRef.focus();
			createEffect(() => {
				props.sender({
					type: 'wordBombInput',
					input: game.input
				});
			});
		}
	});

	const onWordBombGuess = (
		uuid: Uuid,
		guess: { type: 'correct' } | { type: 'invalid'; reason: string }
	) => {
		const element = document.getElementById(uuid);

		if (guess.type === 'correct') {
			element?.animate([{ color: '#00FF44' }, { color: '#FFF' }], 1000);
		} else {
			element?.animate([{ color: '#FF0000' }, { color: '#FFF' }], 1000);
		}
	};

	const WordBomb = () => (
		<section class="flex min-h-screen flex-col items-center justify-center gap-y-3">
			<div class="flex gap-2">
				<h1>turn</h1>
				<h1 class="text-green-300">
					{connection.clients.find((player) => player.uuid === game.currentTurn)!.username}
				</h1>
			</div>
			<h1 class="text-xl">{game.prompt}</h1>
			<div class="flex gap-4">
				<For each={game.players}>
					{(player) => (
						<div id={player.uuid} class="flex flex-col items-center">
							<h1>{connection.clients.find((client) => player.uuid === client.uuid)!.username}</h1>
							<h1>{player.input}</h1>
							<h1 class="text-red-400">lives: {player.lives}</h1>
							<Show
								when={
									connection.clients.find((client) => player.uuid === client.uuid)!.disconnected
								}
							>
								<h1>i disconnected...</h1>
							</Show>
						</div>
					)}
				</For>
			</div>
			<div class="flex flex-col items-center gap-1">
				<h1>unused letters</h1>
				<div class="flex gap-0.5">
					<For each={unusedLetters()}>{(letter) => <kbd>{letter}</kbd>}</For>
				</div>
			</div>
			<input
				ref={gameInputRef}
				class="border text-black"
				type="text"
				maxlength="35"
				disabled={game.currentTurn !== connection.uuid}
				value={game.input}
				onInput={(event) => setWordBomb('input', event.target.value.substring(0, 35))}
				onKeyDown={(event) => {
					if (event.key === 'Enter' && event.currentTarget.value.length <= 35) {
						props.sender({
							type: 'wordBombGuess',
							word: event.currentTarget.value
						});
					}
				}}
			/>
		</section>
	);

	return {
		onWordBombGuess,
		WordBomb
	};
};

export { createWordBomb };
