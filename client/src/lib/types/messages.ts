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
			rejoinToken?: string;
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
			word?: string;
			lifeChange: number;
			newPrompt: string;
			newTurn: string;
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
