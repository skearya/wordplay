import { onCleanup } from "solid-js";
import { ServerMessage } from "~/lib/types/messages";

type ServerMessageTypes = ServerMessage["type"];

type ServerMessageData<EventT> = Extract<ServerMessage, { type: EventT }>;

const listeners: {
  [EventT in ServerMessageTypes]: Array<(data: ServerMessageData<EventT>) => void>;
} = {
  Info: [],
  Error: [],
  RoomSettings: [],
  ChatMessage: [],
  ConnectionUpdate: [],
  ReadyPlayers: [],
  StartingCountdown: [],
  GameStarted: [],
  GameEnded: [],
  WordBombInput: [],
  WordBombInvalidGuess: [],
  WordBombPrompt: [],
  AnagramsInvalidGuess: [],
  AnagramsCorrectGuess: [],
};

export function callEventListeners<EventT extends ServerMessageTypes>(
  event: ServerMessageData<EventT>,
) {
  listeners[event.type].forEach((listener) => listener(event));
}

export function useEvent<EventT extends ServerMessageTypes>(
  type: EventT,
  f: (data: ServerMessageData<EventT>) => void,
) {
  listeners[type].push(f);

  onCleanup(() => {
    const index = listeners[type].indexOf(f);

    if (index > -1) {
      listeners[type].splice(index, 1);
    }
  });
}
