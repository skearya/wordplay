export type Uuid = string;

export type ClientMessage =
	| { type: 'ready' }
	| { type: 'startEarly' }
	| { type: 'unready' }
	| { type: 'chatMessage'; content: string }
	| { type: 'input'; input: string }
	| { type: 'guess'; word: string };

export type ServerMessage =
	| {
			type: 'roomInfo';
			uuid: Uuid;
			roomOwner: Uuid;
			clients: Array<ClientInfo>;
			state: RoomState;
	  }
	| {
			type: 'serverMessage';
			content: string;
	  }
	| {
			type: 'chatMessage';
			author: Uuid;
			content: string;
	  }
	| {
			type: 'connectionUpdate';
			uuid: Uuid;
			state: ConnectionUpdate;
	  }
	| {
			type: 'readyPlayers';
			ready: Array<Uuid>;
	  }
	| {
			type: 'startingCountdown';
			state: CountdownState;
	  }
	| {
			type: 'gameStarted';
			rejoinToken?: string;
			players: Array<PlayerData>;
			prompt: string;
			turn: Uuid;
	  }
	| {
			type: 'inputUpdate';
			uuid: Uuid;
			input: string;
	  }
	| {
			type: 'invalidWord';
			uuid: Uuid;
			reason: InvalidWordReason;
	  }
	| {
			type: 'newPrompt';
			word?: string;
			lifeChange: number;
			newPrompt: string;
			newTurn: Uuid;
	  }
	| {
			type: 'gameEnded';
			winner: Uuid;
			newRoomOwner?: Uuid;
	  };

type RoomState =
	| {
			type: 'lobby';
			ready: Array<Uuid>;
			startingCountdown?: number;
	  }
	| {
			type: 'inGame';
			players: Array<PlayerData>;
			turn: Uuid;
			prompt: string;
			usedLetters?: Array<string>;
	  };

export type ClientInfo = {
	uuid: Uuid;
	username: string;
};

export type PlayerData = {
	uuid: Uuid;
	username: string;
	disconnected: boolean;
	input: string;
	lives: number;
};

type ConnectionUpdate =
	| {
			type: 'connected';
			username: string;
	  }
	| {
			type: 'reconnected';
			username: string;
	  }
	| {
			type: 'disconnected';
			newRoomOwner?: Uuid;
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
