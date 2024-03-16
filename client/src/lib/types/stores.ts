import { PlayerData, PlayerInfo } from './messages';

export type ConnectionData = {
	room: string;
	username: string;
	uuid: string;
	chatMessages: Array<string>;
};

export type LobbyData = {
	readyPlayers: Array<PlayerInfo>;
	previousWinner: string | null;
	startingCountdown: number | null;
};

export type GameData = {
	players: Array<PlayerData>;
	currentTurn: string;
	prompt: string;
	usedLetters: Set<string> | null;
	input: string;
};
