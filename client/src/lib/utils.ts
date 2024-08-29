import { Room, State } from "~/lib/types/game";
import { AnagramsGuessInfo, ClientInfo, RoomStateInfo, Uuid } from "~/lib/types/messages";

export type Variant<T, U> = Extract<T, { type: U }>;

export const url = (path: string) => `${import.meta.env.DEV ? "http://localhost:3021" : ""}${path}`;

export const cubicEasing = "cubic-bezier(0.33, 1, 0.68, 1)";

export const colors = {
  green: "rgb(98 226 151)",
  darkGreen: "rgb(71 93 80)",
  red: "rgb(220 38 38)",
};

export type ErrorType = "socket closed";

export class GameError extends Error {
  type: ErrorType;

  constructor({ type, message }: { type: ErrorType; message: string }) {
    super(message);
    this.name = "GameError";
    this.type = type;
    this.message = message;
  }
}

export function cloneElement(element: HTMLElement) {
  const clone = element.cloneNode(true) as HTMLElement;
  clone.style.position = "absolute";

  const rect = element.getBoundingClientRect();
  clone.style.top = rect.top + "px";
  clone.style.left = rect.left + "px";
  clone.style.width = rect.width + "px";

  return clone;
}

export function getRejoinToken(roomName: string): string | undefined {
  const rejoinTokensString = localStorage.getItem("rejoinTokens");

  if (rejoinTokensString) {
    return JSON.parse(rejoinTokensString)[roomName];
  }
}

export function saveRejoinToken(roomName: string, token: string) {
  const rejoinTokensString = localStorage.getItem("rejoinTokens");

  const rejoinTokens: Record<string, string | undefined> = rejoinTokensString
    ? JSON.parse(rejoinTokensString)
    : {};

  localStorage.setItem(
    "rejoinTokens",
    JSON.stringify({
      ...rejoinTokens,
      [roomName]: token,
    }),
  );
}

export function convertStateMessage(state: RoomStateInfo): State {
  switch (state.type) {
    case "Lobby": {
      const { type, ready, starting_countdown } = state;

      return {
        type,
        ready,
        startingCountdown: starting_countdown,
      };
    }
    case "WordBomb": {
      const { type, players, turn, prompt, used_letters } = state;

      return {
        type,
        players,
        turn,
        prompt,
        usedLetters: new Set(used_letters),
      };
    }
    case "Anagrams": {
      return state;
    }
  }
}

export function getClient(room: Room, uuid: Uuid): ClientInfo | undefined {
  return room.clients.find((client) => client.uuid === uuid);
}

export function getUsername(room: Room, uuid: Uuid): string | undefined {
  return room.clients.find((client) => client.uuid === uuid)?.username;
}

export const calculateAnagramsPoints = (word: string) => 50 * 2 ** (word.length - 2);

export function translateAnagramsGuessError(guessInfo: AnagramsGuessInfo) {
  switch (guessInfo.type) {
    case "NotLongEnough":
      return "not long enough";
    case "PromptMismatch":
      return "word doesn't contain anagram";
    case "NotEnglish":
      return "word isn't valid english";
    case "AlreadyUsed":
      return "word already used";
  }
}

export function removeNonAlphanumeric(input: string) {
  return input.replace(/[^a-zA-Z0-9]/g, "");
}
