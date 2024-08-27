import {
  AnagramsPlayerData,
  ClientInfo,
  ClientMessage,
  RoomSettings,
  Uuid,
  WordBombPlayerData,
} from "./messages";

export type SendFn = (message: ClientMessage) => void;

export enum ChatMessageType {
  Info,
  Error,
  Client,
}

export type ChatMessage =
  | { type: ChatMessageType.Info | ChatMessageType.Error; content: string }
  | { type: ChatMessageType.Client; uuid: Uuid; content: string };

export type Room = {
  uuid: Uuid;
  clients: ClientInfo[];
  owner: Uuid;
  settings: RoomSettings;
};

export type LobbyState = {
  type: "Lobby";
  ready: Array<Uuid>;
  startingCountdown: number | null;
};

export type WordBombState = {
  type: "WordBomb";
  players: Array<WordBombPlayerData>;
  turn: Uuid;
  prompt: string;
  usedLetters: Set<string> | null;
};

export type AnagramsState = {
  type: "Anagrams";
  players: Array<AnagramsPlayerData>;
  anagram: string;
};

export type State = LobbyState | WordBombState | AnagramsState;
