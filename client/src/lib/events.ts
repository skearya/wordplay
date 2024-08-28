import { onCleanup } from "solid-js";
import { ServerMessage } from "~/lib/types/messages";
import { Variant } from "~/lib/utils";

export type ServerMessageData<EventT> = Variant<ServerMessage, EventT>;

type ServerMessageTypes = ServerMessage["type"];

const listeners: {
  [EventT in ServerMessageTypes]: Array<(data: ServerMessageData<EventT>) => void>;
} = {
  Pong: [],
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

const unactedMessages: {
  [EventT in ServerMessageTypes]?: Array<ServerMessageData<EventT>>;
} = {};

export function callEventListeners(event: ServerMessage) {
  if (listeners[event.type].length !== 0) {
    (listeners[event.type] as Array<(data: ServerMessage) => void>).forEach((listener) =>
      listener(event),
    );
  } else {
    unactedMessages[event.type] ??= [];
    (unactedMessages[event.type] as Array<ServerMessage>).push(event);
  }
}

export function useEvent<EventT extends ServerMessageTypes>(
  type: EventT,
  f: (data: ServerMessageData<EventT>) => void,
  cleanup = true,
) {
  if (unactedMessages[type]) {
    unactedMessages[type].forEach(f);
    unactedMessages[type] = [];
  }

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
