export type Uuid = string;

export type ClientMessage =
	| { type: 'gameSettings'; public: boolean }
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
			room: RoomInfo;
	  }
	| {
			type: 'gameSettings';
			public: boolean;
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
			countdownUpdate?: CountdownState;
	  }
	| {
			type: 'startingCountdown';
			timeLeft: number;
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

type RoomInfo = {
	public: boolean;
	owner: Uuid;
	clients: Array<ClientInfo>;
	state: RoomState;
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
