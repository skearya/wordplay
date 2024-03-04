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
			usedLetters: Set<string>;

			uuid: string;
			chatMessages: Array<string>;
	  };

export type ClientMessage =
	| { type: 'ready' }
	| { type: 'unready' }
	| { type: 'chatMessage'; content: string }
	| { type: 'input'; input: string }
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
			countdown: boolean;
	  }
	| {
			type: 'gameStarted';
			prompt: string;
			turn: string;
			players: Array<PlayerData>;
	  }
	| {
			type: 'inputUpdate';
			uuid: string;
			input: string;
	  }
	| {
			type: 'invalidWord';
			uuid: string;
	  }
	| {
			type: 'newPrompt';
			lifeChange: number;
			prompt: string;
			turn: string;
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
	input: string;
	lives: number;
};

export const inGameOrLobby = P.when(
	(type): type is 'lobby' | 'game' => type === 'lobby' || type === 'game'
);
