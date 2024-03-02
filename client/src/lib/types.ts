import { P } from 'ts-pattern';

export type AppState =
	| {
			type: 'connecting';
	  }
	| {
			type: 'error';
			message: string;
	  }
	| {
			type: 'lobby';
			readyPlayers: Array<PlayerInfo>;

			uuid: string;
			chatMessages: Array<string>;
	  }
	| {
			type: 'game';
			players: Array<PlayerData>;
			currentTurn: PlayerData;
			prompt: string;

			uuid: string;
			chatMessages: Array<string>;
	  };

export type ClientMessage =
	| { type: 'ready' }
	| { type: 'unready' }
	| { type: 'chatMessage'; content: string }
	| { type: 'guess'; word: string };

export type ServerMessage =
	| {
			type: 'roomInfo';
			uuid: string;
			state: RoomState;
	  }
	| {
			type: 'serverMessage';
			content: string;
	  }
	| {
			type: 'chatMessage';
			author: string;
			content: string;
	  }
	| {
			type: 'readyPlayers';
			players: Array<PlayerInfo>;
	  }
	| {
			type: 'gameStarted';
			prompt: string;
			turn: string;
			players: Array<PlayerData>;
	  }
	| {
			type: 'newPrompt';
			timedOut: boolean;
			prompt: string;
			turn: string;
	  }
	| {
			type: 'invalidWord';
			uuid: string;
	  }
	| {
			type: 'gameEnded';
			winner: string;
	  };

type RoomState =
	| {
			type: 'lobby';
			readyPlayers: Array<PlayerInfo>;
	  }
	| {
			type: 'inGame';
			prompt: string;
			turn: string;
			players: Array<PlayerData>;
	  };

export type PlayerInfo = {
	uuid: string;
	username: string;
};

export type PlayerData = {
	uuid: string;
	username: string;
	lives: number;
};

export const inGameOrLobby = P.when(
	(type): type is 'lobby' | 'game' => type === 'lobby' || type === 'game'
);
