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
	// lobby / generic
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
			game: Exclude<RoomStateInfo, { type: 'lobby' }>;
	  }
	| {
			type: 'gameEnded';
			newRoomOwner?: Uuid;
			info: PostGameInfo;
	  }

	// word bomb
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

	// anagrams
	| {
			type: 'anagramsInvalidGuess';
			reason: AnagramsGuessInfo;
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

export type RoomSettings = {
	public: boolean;
	game: Games;
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
			anagram: string;
	  };

export type PostGameInfo =
	| {
			type: 'wordBomb';
			winner: Uuid;
			minsElapsed: number;
			wordsUsed: number;
			lettersTyped: number;
			fastestGuesses: Array<[Uuid, number, string]>;
			longestWords: Array<[Uuid, string]>;
			avgWpms: Array<[Uuid, number]>;
			avgWordLengths: Array<[Uuid, number]>;
	  }
	| {
			type: 'anagrams';
			originalWord: string;
			leaderboard: Array<[Uuid, number]>;
	  };

export type ClientInfo = {
	uuid: Uuid;
	username: string;
	disconnected: boolean;
};

export type WordBombPlayerData = {
	uuid: Uuid;
	input: string;
	lives: number;
};

export type AnagramsPlayerData = {
	uuid: Uuid;
	usedWords: Array<string>;
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

type AnagramsGuessInfo =
	| {
			type: 'notLongEnough';
	  }
	| {
			type: 'promptMismatch';
	  }
	| {
			type: 'notEnglish';
	  }
	| {
			type: 'alreadyUsed';
	  };
