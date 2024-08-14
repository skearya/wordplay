import {
  AnagramsPlayerData,
  ClientInfo,
  ClientMessage,
  RoomSettings,
  Uuid,
  WordBombPlayerData,
} from "./messages";

export type SendFn = (message: ClientMessage) => void;

export type Room = {
  uuid: Uuid;
  clients: ClientInfo[];
  owner: Uuid;
  settings: RoomSettings;
};

export type LobbyState = {
  type: "Lobby";
  ready: Array<Uuid>;
  startingCountdown: number | undefined;
};

export type WordBombState = {
  type: "WordBomb";
  players: Array<WordBombPlayerData>;
  turn: Uuid;
  prompt: string;
  usedLetters: Array<string> | undefined;
};

export type AnagramsState = {
  type: "Anagrams";
  players: Array<AnagramsPlayerData>;
  anagram: string;
};

export type State = LobbyState | WordBombState | AnagramsState;
