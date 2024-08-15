import { Room, State } from "./types/game";
import { RoomStateInfo, Uuid } from "./types/messages";

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

  if (rejoinTokensString) {
    const rejoinTokens = JSON.parse(rejoinTokensString) as Record<string, string | undefined>;

    localStorage.setItem(
      "rejoinTokens",
      JSON.stringify({
        ...rejoinTokens,
        roomName: token,
      }),
    );
  }
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

export function getUsername(room: Room, uuid: Uuid): string | undefined {
  return room.clients.find((client) => client.uuid === uuid)?.username;
}
