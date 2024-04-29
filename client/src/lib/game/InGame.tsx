import { type Component, useContext, For, Show, createEffect } from 'solid-js';
import { Context } from '../context';
import { ClientMessage } from '../types/messages';

const InGame: Component<{ sendMessage: (message: ClientMessage) => void }> = (props) => {
	const context = useContext(Context);
	if (!context) throw new Error('Not called inside context provider?');
	const { connection, game } = context[0];
	const { setGame } = context[1];

	let gameInputRef!: HTMLInputElement;

	const ourTurn = () => game.currentTurn == connection.uuid;
	const unusedLetters = () =>
		[...'abcdefghijklmnopqrstuvwy'].filter((letter) => !game.usedLetters?.has(letter));

	createEffect(() => {
		if (ourTurn()) {
			props.sendMessage({
				type: 'wordBombInput',
				input: game.input
			});
		}
	});

	createEffect(() => {
		if (ourTurn()) {
			gameInputRef.focus();
		}
	});

	createEffect(() => {
		const [uuid, _reason] = game.guessError;
		if (uuid == '') return;
		const element = document.getElementById(uuid);

		if (element) {
			element.animate(
				[
					{
						color: '#FF0000'
					},
					{
						color: '#FFF'
					}
				],
				1000
			);
		}
	});

	return (
		<section class="flex min-h-screen flex-col items-center justify-center gap-y-3">
			<div class="flex gap-2">
				<h1>turn</h1>
				<h1 class="text-green-300">
					{game.players.find((player) => player.uuid === game.currentTurn)!.username}
				</h1>
			</div>
			<h1 class="text-xl">{game.prompt}</h1>
			<div class="flex gap-4">
				<For each={game.players}>
					{(player) => (
						<div id={player.uuid} class="flex flex-col items-center">
							<h1>{player.username}</h1>
							<h1>{player.input}</h1>
							<h1 class="text-red-400">lives: {player.lives}</h1>
							<Show when={player.disconnected}>
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
				onInput={(event) => setGame('input', event.target.value.substring(0, 35))}
				onKeyDown={(event) => {
					if (event.key === 'Enter' && event.currentTarget.value.length <= 35) {
						props.sendMessage({
							type: 'wordBombGuess',
							word: event.currentTarget.value
						});
					}
				}}
			/>
		</section>
	);
};

export { InGame };
