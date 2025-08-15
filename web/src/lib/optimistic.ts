import type { ClientGame } from '@bindings/ClientGame';
import type { ClientLobby } from '@bindings/ClientLobby';
import type { ClientMessage } from '@bindings/ClientMessage';
import type { GameState } from '@bindings/GameState';
import type { Info } from '@bindings/Info';
import type { LobbyState } from '@bindings/LobbyState';
import type { RoomSettings } from '@bindings/RoomSettings';
import type { ServerState } from '@bindings/ServerState';

type Context<State> = Omit<Info, 'state'> & { state: State };

/// Extra state that can be also be rolled-back.
type ExtraState = {};

function optimisticApply(
	info: Info,
	message: ClientMessage
): { settings: RoomSettings; state: ServerState } | null {
	let state: ServerState;

	switch (message.kind) {
		case 'general':
			switch (message.data.kind) {
				case 'ping':
					break;
				case 'chat':
					break;
				case 'settings':
					break;
			}

			state = structuredClone(info.state);
			break;
		case 'lobby':
			if (info.state.kind !== 'lobby') throw new Error('Invalid state');

			const lobby = optimisticApplyLobby(info as Context<LobbyState>, message.data);
			if (!lobby) return null;

			state = { kind: 'lobby', ...lobby };
			break;
		case 'game':
			if (info.state.kind !== 'game') throw new Error('Invalid state');

			const game = optimisticApplyGame(info as Context<GameState>, message.data);
			if (!game) return null;

			state = { kind: 'game', ...game };
			break;
	}

	return { settings: info.settings, state };
}

function optimisticApplyLobby(
	{ uuid, state }: Context<LobbyState>,
	message: ClientLobby
): LobbyState | null {
	switch (message.kind) {
		case 'ready':
			return { ...state, ready: [...state.ready, uuid] };
		case 'startEarly':
			return null;
		case 'unready':
			return { ...state, ready: state.ready.filter((client) => client !== uuid) };
		case 'practiceRequest':
			return null;
		case 'practiceSubmission':
			return null;
	}
}

function optimisticApplyGame(context: Context<GameState>, message: ClientGame): GameState | null {
	switch (message.kind) {
		case 'wordBomb':
			switch (message.data.kind) {
				case 'input':
					break;
				case 'guess':
					break;
			}
			break;
		case 'anagrams':
			switch (message.data.kind) {
				case 'guess':
					break;
			}
			break;
		case 'endRequest':
			break;
		case 'forceEnd':
			break;
	}

	return null;
}
