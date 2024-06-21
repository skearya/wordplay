import type {
  AnagramsPlayerData,
  ClientInfo,
  Games,
  PostGameInfo,
  RoomSettings,
  Uuid,
  WordBombPlayerData,
} from "@game/types/messages";
import type { Emitter } from "@solid-primitives/event-bus";

export type State = "Connecting" | "Error" | "Lobby" | Games;

export type ConnectionData = {
  room: string;
  settings: RoomSettings;
  uuid: Uuid;
  roomOwner: Uuid;
  clients: Array<ClientInfo>;
  chatMessages: Array<[string, string] | string>;
  events: Emitter<{
    wordBombGuess: {
      uuid: string;
      guess: { type: "correct" } | { type: "invalid"; reason: string };
    };
    anagramsGuess: { type: "correct" } | { type: "invalid"; reason: string };
  }>;
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
