import type {
	ClientInfo,
	WordBombPlayerData,
	Uuid,
	RoomSettings,
	AnagramsPlayerData
} from './messages';

export type State = 'connecting' | 'error' | 'lobby' | 'wordBomb' | 'anagrams';

export type ConnectionData = {
	room: string;
	settings: RoomSettings;
	uuid: Uuid;
	username: string;
	roomOwner: Uuid;
	clients: Array<ClientInfo>;
	chatMessages: Array<string>;
};

export type LobbyData = {
	readyPlayers: Array<Uuid>;
	previousWinner: string | null;
	startingCountdown: number | null;
};

export type WordBombData = {
	players: Array<WordBombPlayerData>;
	currentTurn: Uuid;
	prompt: string;
	input: string;
	guessError: [Uuid, string];
	usedLetters: Set<string> | null;
};

export type AnagramsData = {
	players: Array<AnagramsPlayerData>;
	prompt: string;
	// [string] to push reactive updates with the same content
	guessError: [string];
	input: string;
};
