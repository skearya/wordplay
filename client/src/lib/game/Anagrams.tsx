import { useContext, For, Show, createSignal } from 'solid-js';
import { Context } from '../context';
import { ClientMessage } from '../types/messages';

export const createAnagrams = (props: { sender: (message: ClientMessage) => void }) => {
	const context = useContext(Context);
	if (!context) throw new Error('Not called inside context provider?');
	const { connection, anagrams: game } = context[0];

	let gameInputRef!: HTMLInputElement;
	let errorTextRef!: HTMLHeadingElement;

	const [input, setInput] = createSignal('');
	const [guessError, setGuessError] = createSignal('');

	const onAnagramsGuess = (guess: { type: 'correct' } | { type: 'invalid'; reason: string }) => {
		if (guess.type === 'correct') {
			setInput('');
		} else {
			setGuessError(guess.reason);
			gameInputRef.animate([{ backgroundColor: '#FF0000' }, { backgroundColor: '#FFF' }], 1000);
			errorTextRef.animate([{ opacity: 100 }, { opacity: 0 }], 1000);
		}
	};

	const Anagrams = () => (
		<section class="flex min-h-screen flex-col items-center justify-center gap-y-3">
			<h1 class="text-xl">{game.anagram}</h1>
			<input
				ref={gameInputRef}
				class="border text-black"
				type="text"
				maxlength="6"
				disabled={!game.players.some((player) => connection.uuid === player.uuid)}
				value={input()}
				onInput={(event) => setInput(event.target.value.substring(0, 6))}
				onKeyDown={(event) => {
					if (event.key === 'Enter' && event.currentTarget.value.length <= 6) {
						props.sender({
							type: 'AnagramsGuess',
							word: event.currentTarget.value
						});
					}
				}}
			/>
			<div class="flex gap-4">
				<For each={game.players}>
					{(player) => (
						<div class="flex flex-col items-center">
							<h1>{connection.clients.find((client) => player.uuid === client.uuid)!.username}</h1>
							<div>
								<For each={player.used_words}>{(word) => <h1>{word}</h1>}</For>
							</div>
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
			<h1 ref={errorTextRef} class="text-red-400 opacity-0">
				{guessError()}
			</h1>
		</section>
	);

	return {
		onAnagramsGuess,
		Anagrams
	};
};
