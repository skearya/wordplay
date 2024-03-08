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
			uuid: string;
			chatMessages: Array<string>;
			readyPlayers: Array<PlayerInfo>;
			previousWinner: string | null;
			countdown: number | null;
	  }
	| {
			type: 'game';
			uuid: string;
			chatMessages: Array<string>;
			players: Array<PlayerData>;
			currentTurn: string;
			prompt: string;
			usedLetters: Set<string>;
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
	  }
	| {
			type: 'startingCountdown';
			state: CountdownState;
	  }
	| {
			type: 'gameStarted';
			rejoinToken: string;
			prompt: string;
			turn: string;
			players: Array<PlayerData>;
	  }
	| {
			type: 'playerUpdate';
			uuid: string;
			state: PlayerUpdate;
	  }
	| {
			type: 'inputUpdate';
			uuid: string;
			input: string;
	  }
	| {
			type: 'invalidWord';
			uuid: string;
			reason: InvalidWordReason;
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
			ready: Array<PlayerInfo>;
			startingCountdown?: number;
	  }
	| {
			type: 'inGame';
			prompt: string;
			turn: string;
			players: Array<PlayerData>;
			usedLetters?: Array<string>;
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
	disconnected: boolean;
};

type PlayerUpdate =
	| {
			type: 'disconnected';
	  }
	| {
			type: 'reconnected';
			username: string;
	  };

type CountdownState =
	| {
			type: 'inProgress';
			timeLeft: number;
	  }
	| {
			type: 'stopped';
	  };

type InvalidWordReason =
	| {
			type: 'promptNotIn';
	  }
	| {
			type: 'notEnglish';
	  }
	| {
			type: 'alreadyUsed';
	  };

export const inGameOrLobby = P.when(
	(type): type is 'lobby' | 'game' => type === 'lobby' || type === 'game'
);
