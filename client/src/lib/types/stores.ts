import type {
	ClientInfo,
	WordBombPlayerData,
	Uuid,
	RoomSettings,
	AnagramsPlayerData,
	PostGameInfo
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
	startingCountdown: number | null;
	postGame: PostGameInfo | null;
};

export type WordBombData = {
	players: Array<WordBombPlayerData>;
	currentTurn: Uuid;
	prompt: string;
	input: string;
	usedLetters: Set<string> | null;
};

export type AnagramsData = {
	players: Array<AnagramsPlayerData>;
	anagram: string;
};
