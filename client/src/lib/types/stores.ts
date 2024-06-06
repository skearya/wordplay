import type {
	ClientInfo,
	WordBombPlayerData,
	Uuid,
	RoomSettings,
	AnagramsPlayerData,
	PostGameInfo,
	Games
} from './messages';

export type State = 'Connecting' | 'Error' | 'Lobby' | Games;

export type ConnectionData = {
	room: string;
	settings: RoomSettings;
	uuid: Uuid;
	username: string;
	roomOwner: Uuid;
	clients: Array<ClientInfo>;
	chatMessages: Array<[string, string]>;
};

export type LobbyData = {
	ready: Array<Uuid>;
	startingCountdown: number | null;
	postGame: PostGameInfo | null;
};

export type WordBombData = {
	players: Array<WordBombPlayerData>;
	turn: Uuid;
	prompt: string;
	input: string;
	usedLetters: Set<string> | null;
};

export type AnagramsData = {
	players: Array<AnagramsPlayerData>;
	anagram: string;
};
