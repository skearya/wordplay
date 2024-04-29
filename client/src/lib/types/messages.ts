export type Uuid = string;

type Games = 'wordBomb' | 'anagrams';

export type ClientMessage =
	| { type: 'ready' }
	| { type: 'startEarly' }
	| { type: 'unready' }
	| ({ type: 'roomSettings' } & RoomSettings)
	| { type: 'chatMessage'; content: string }
	| { type: 'wordBombInput'; input: string }
	| { type: 'wordBombGuess'; word: string }
	| { type: 'anagramsGuess'; word: string };

export type ServerMessage =
	| {
			type: 'info';
			uuid: Uuid;
			room: RoomInfo;
	  }
	| {
			type: 'error';
			content: string;
	  }
	| ({
			type: 'roomSettings';
	  } & RoomSettings)
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
			game: RoomStateInfo;
	  }
	| {
			type: 'gameEnded';
			winner: Uuid;
			newRoomOwner?: Uuid;
	  }
	| {
			type: 'wordBombInput';
			uuid: Uuid;
			input: string;
	  }
	| {
			type: 'wordBombInvalidGuess';
			uuid: Uuid;
			reason: WordBombGuessInfo;
	  }
	| {
			type: 'wordBombPrompt';
			correctGuess?: string;
			lifeChange: number;
			prompt: string;
			turn: Uuid;
	  }
	| {
			type: 'anagramsCorrectGuess';
			uuid: Uuid;
			guess: string;
	  };

type RoomInfo = {
	owner: Uuid;
	settings: RoomSettings;
	clients: Array<ClientInfo>;
	state: RoomStateInfo;
};

type RoomStateInfo =
	| {
			type: 'lobby';
			ready: Array<Uuid>;
			startingCountdown?: number;
	  }
	| {
			type: 'wordBomb';
			players: Array<WordBombPlayerData>;
			turn: Uuid;
			prompt: string;
			usedLetters?: Array<string>;
	  }
	| {
			type: 'anagrams';
			players: Array<AnagramsPlayerData>;
			prompt: string;
			usedWords?: Array<String>;
	  };

export type RoomSettings = {
	public: boolean;
	game: Games;
};

export type ClientInfo = {
	uuid: Uuid;
	username: string;
};

export type WordBombPlayerData = {
	uuid: Uuid;
	username: string;
	disconnected: boolean;
	input: string;
	lives: number;
};

export type AnagramsPlayerData = {
	puuid: Uuid;
	username: string;
	disconnected: boolean;
	used_words: Array<string>;
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

type WordBombGuessInfo =
	| {
			type: 'promptNotIn';
	  }
	| {
			type: 'notEnglish';
	  }
	| {
			type: 'alreadyUsed';
	  };
