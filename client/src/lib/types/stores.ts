import type { ClientInfo, PlayerData, Uuid } from './messages';

export type ConnectionData = {
	clients: Array<ClientInfo>;
	uuid: Uuid;
	username: string;
	room: string;
	chatMessages: Array<string>;
};

export type LobbyData = {
	readyPlayers: Array<Uuid>;
	previousWinner: string | null;
	startingCountdown: number | null;
};

export type GameData = {
	players: Array<PlayerData & { username: string }>;
	currentTurn: Uuid;
	prompt: string;
	input: string;
	guessError: [Uuid, string];
	usedLetters: Set<string> | null;
};
