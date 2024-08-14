import { onCleanup } from "solid-js";
import { ServerMessage } from "~/lib/types/messages";

type ServerMessageTypes = ServerMessage["type"];

export type ServerMessageData<EventT> = Extract<ServerMessage, { type: EventT }>;

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

export function callEventListeners(event: ServerMessage) {
  (listeners[event.type] as Array<(data: ServerMessage) => void>).forEach((listener) =>
    listener(event),
  );
}

/*
  TODO: make sure events recieved before a listener subscribes will get sent to it
  look into https://www.npmjs.com/package/mitt
*/

export function useEvent<EventT extends ServerMessageTypes>(
  type: EventT,
  f: (data: ServerMessageData<EventT>) => void,
  cleanup = true,
) {
  listeners[type].push(f);

  const cleanupFn = () => {
    const index = listeners[type].indexOf(f);

    if (index > -1) {
      listeners[type].splice(index, 1);
    }
  };

  if (cleanup) {
    onCleanup(cleanupFn);
  }

  return cleanupFn;
}

export function useEvents(events: {
  [EventT in ServerMessageTypes]?: (data: ServerMessageData<EventT>) => void;
}) {
  for (const [type, f] of Object.entries(events)) {
    useEvent(
      type as ServerMessageTypes,
      f as (data: ServerMessageData<ServerMessageTypes>) => void,
    );
  }
}
