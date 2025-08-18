import type { Context } from '@bindings/Context';
import type { LobbyState } from '@bindings/LobbyState';
import type { ServerAnagrams } from '@bindings/ServerAnagrams';
import type { ServerGame } from '@bindings/ServerGame';
import type { ServerGeneral } from '@bindings/ServerGeneral';
import type { ServerLobby } from '@bindings/ServerLobby';
import type { ServerMessage } from '@bindings/ServerMessage';
import type { ServerWordBomb } from '@bindings/ServerWordBomb';
import type { TimerAction } from '@bindings/TimerAction';
import { serverMessageEmitter } from './events';

export function handleServerMessage(ctx: Context, message: ServerMessage) {
	switch (message.kind) {
		case 'general':
			handleServerGeneralMessage(ctx, message.data);
			break;
		case 'lobby':
			handleServerLobbyMessage(ctx, message.data);
			break;
		case 'game':
			handleServerGameMessage(ctx, message.data);
			break;
	}

	serverMessageEmitter.emit(message);
}

function handleServerGeneralMessage(ctx: Context, message: ServerGeneral) {
	switch (message.kind) {
		case 'info':
			Object.assign(ctx, message);
			break;
		case 'join':
			ctx.clients[message.uuid] = message.client;
			break;
		case 'leave':
			delete ctx.clients[message.uuid];
			break;
		case 'pong':
			break;
		case 'chat':
			break;
		case 'settings':
			ctx.settings = message;
			break;
		case 'error':
			break;
	}
}

function handleServerLobbyMessage(ctx: Context, message: ServerLobby) {
	if (ctx.state.kind !== 'lobby') return;

	const timerAction = (state: LobbyState, timer: TimerAction) => {
		switch (timer) {
			case 'start':
				state.timerStart = BigInt(Date.now());
				break;
			case 'stop':
				state.timerStart = null;
				break;
			case 'none':
				break;
		}
	};

	switch (message.kind) {
		case 'ready':
			ctx.state.ready.push(message.uuid);
			timerAction(ctx.state, message.timer);
			break;
		case 'unready':
			ctx.state.ready = ctx.state.ready.filter((client) => client !== message.uuid);
			timerAction(ctx.state, message.timer);
			break;
		case 'practice':
			break;
		case 'practiceResult':
			break;
		case 'gameStarted':
			if (message.rejoinToken) {
				localStorage.setItem(`rejoinToken`, message.rejoinToken);
			}

			ctx.state = { kind: 'game', ...message.state };
			break;
	}
}

function handleServerGameMessage(ctx: Context, message: ServerGame) {
	if (ctx.state.kind !== 'game') return;

	switch (message.kind) {
		case 'wordBomb':
			handleServerWordBombMessage(ctx, message.data);
			break;
		case 'anagrams':
			handleServerAnagramsMessage(ctx, message.data);
			break;
		case 'endRequest':
			ctx.state.requestingEnd.push(message.data.uuid);
			break;
		case 'ended':
			if (message.data.newOwner) {
				ctx.settings.owner = message.data.newOwner;
			}

			ctx.state = {
				kind: 'lobby',
				ready: [],
				timerStart: null,
				prevGame: message.data.postGameInfo
			};
			break;
	}
}

function handleServerWordBombMessage(ctx: Context, message: ServerWordBomb) {
	if (ctx.state.kind !== 'game' || ctx.state.state.kind !== 'wordBomb') return;

	const {
		state: { state: game }
	} = ctx;

	switch (message.kind) {
		case 'input':
			game.players[game.turn]!.input = message.input;
			break;
		case 'valid':
			if (message.life) {
				game.players[game.turn]!.lives += 1;
			}

			game.prompt = message.prompt;
			game.turn = message.turn;
			break;
		case 'invalid':
			break;
		case 'exploded':
			game.prompt = message.prompt;
			game.turn = message.turn;
			break;
	}
}

function handleServerAnagramsMessage(ctx: Context, message: ServerAnagrams) {
	if (ctx.state.kind !== 'game' || ctx.state.state.kind !== 'anagrams') return;

	const {
		state: { state: game }
	} = ctx;

	switch (message.kind) {
		case 'valid':
			break;
		case 'invalid':
			break;
	}
}
