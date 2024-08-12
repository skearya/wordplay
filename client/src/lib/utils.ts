import type { RoomStateInfo } from "./types/messages";

export function cloneElement(element: HTMLElement) {
  const clone = element.cloneNode(true) as HTMLElement;
  clone.style.position = "absolute";

  const rect = element.getBoundingClientRect();
  clone.style.top = rect.top + "px";
  clone.style.left = rect.left + "px";
  clone.style.width = rect.width + "px";

  return clone;
}

export function roomStateToCamelCase(state: RoomStateInfo) {
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
        usedLetters: used_letters,
      };
    }
    case "Anagrams": {
      return state;
    }
  }
}
