import { type Component, useContext, For, Show, createEffect } from 'solid-js';
import { Context } from '../context';
import { ClientMessage } from '../types/messages';

const Anagrams: Component<{ sender: (message: ClientMessage) => void }> = (props) => {
	const context = useContext(Context);
	if (!context) throw new Error('Not called inside context provider?');
	const { anagrams: game } = context[0];
	const { setAnagrams } = context[1];

	let gameInputRef!: HTMLInputElement;

	createEffect(() => {
		const [reason] = game.guessError;
		if (reason == '') return;

		gameInputRef.animate([{ backgroundColor: '#FF0000' }, { backgroundColor: '#FFF' }], 1000);
	});

	return (
		<section class="flex min-h-screen flex-col items-center justify-center gap-y-3">
			<h1 class="text-xl">{game.prompt}</h1>
			<input
				ref={gameInputRef}
				class="border text-black"
				type="text"
				maxlength="6"
				value={game.input}
				onInput={(event) => setAnagrams('input', event.target.value.substring(0, 6))}
				onKeyDown={(event) => {
					if (event.key === 'Enter' && event.currentTarget.value.length <= 6) {
						props.sender({
							type: 'anagramsGuess',
							word: event.currentTarget.value
						});
					}
				}}
			/>
			<div class="flex gap-4">
				<For each={game.players}>
					{(player) => (
						<div class="flex flex-col items-center">
							<h1>{player.username}</h1>
							<div>
								{player.usedWords.map((word) => (
									<h1>{word}</h1>
								))}
							</div>
							<Show when={player.disconnected}>
								<h1>i disconnected...</h1>
							</Show>
						</div>
					)}
				</For>
			</div>
		</section>
	);
};

export { Anagrams };
