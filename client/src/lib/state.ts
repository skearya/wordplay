import { isMatching } from 'ts-pattern';
import type { AppState } from './types';
import { derived, writable } from 'svelte/store';

export const gameState = writable<AppState>({
	type: 'connecting'
});

const isInGame = isMatching({ type: 'game' });

export const gameInfo = derived(gameState, (state) => {
	if (isInGame(state)) {
		return {
			currentTurn: state.players.find((player) => state.currentTurn == player.uuid)!,
			hasTurn: state.uuid === state.currentTurn,
			unusedLetters: [...'abcdefghijklmnopqrstuvwy'].filter(
				(letter) => !state.usedLetters.has(letter)
			)
		};
	}

	return undefined;
});
