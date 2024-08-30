import { useParams } from "@solidjs/router";
import {
  ComponentProps,
  createSignal,
  ErrorBoundary,
  Match,
  onCleanup,
  Show,
  Switch,
} from "solid-js";
import { createStore } from "solid-js/store";
import { Anagrams } from "~/lib/components/Anagrams";
import { Chat } from "~/lib/components/Chat";
import { error, ErrorDisplay, setError } from "~/lib/components/Error";
import { Lobby } from "~/lib/components/Lobby";
import { GameNav } from "~/lib/components/Nav";
import { Button } from "~/lib/components/ui/Button";
import { Input } from "~/lib/components/ui/Input";
import { WordBomb } from "~/lib/components/WordBomb";
import { callEventListeners, ServerMessageData, useEvent, useEvents } from "~/lib/events";
import { ChatMessage, ChatMessageType, Room, SendFn, State } from "~/lib/types/game";
import { ClientMessage, PostGameInfo } from "~/lib/types/messages";
import {
  convertStateMessage,
  GameError,
  getRejoinToken,
  getUsername,
  saveRejoinToken,
  url,
  Variant,
} from "~/lib/utils";

export default function ErrorHandler() {
  return (
    <Show
      when={error() !== undefined}
      fallback={
        <ErrorBoundary
          fallback={(error, reset) => <ErrorDisplay error={error as unknown} reset={reset} />}
        >
          <JoinGame />
        </ErrorBoundary>
      }
    >
      <ErrorDisplay error={error()} />
    </Show>
  );
}

type JoinGameState =
  | {
      type: "waiting" | "connecting";
    }
  | {
      type: "ready";
      gameInfo: ComponentProps<typeof Game>;
    };

function JoinGame() {
  let rootElement!: HTMLElement;

  let socket: WebSocket | undefined;
  const socketCloseListener = new AbortController();

  const roomName = useParams().name!;
  const [state, setState] = createSignal<JoinGameState>({ type: "waiting" });
  const [username, setUsername] = createSignal(localStorage.getItem("username") ?? "");
  const validUsername = () => username().length <= 12 && username() !== "";

  function join() {
    setState({ type: "connecting" });

    localStorage.setItem("username", username());

    const rejoinToken = getRejoinToken(roomName);
    const params = new URLSearchParams({
      username: username(),
      ...(rejoinToken && { rejoin_token: rejoinToken }),
    });

    socket = new WebSocket(url(`/api/room/${roomName}?${params}`));

    socket.addEventListener("message", (event) => {
      callEventListeners(JSON.parse(event.data));
    });

    const removeInfoHandler = useEvent(
      "Info",
      (data) => {
        removeInfoHandler();

        const gameInfo = {
          ...data,
          sendMsg: (message: ClientMessage) => socket?.send(JSON.stringify(message)),
        };

        rootElement.classList.add("quick-fade-out");
        rootElement.addEventListener("animationend", () => setState({ type: "ready", gameInfo }), {
          once: true,
        });
      },
      false,
    );

    socket.addEventListener(
      "close",
      (event) => {
        setError(
          new GameError({
            type: "socket closed",
            message: event.reason ?? "connection closed",
          }),
        );
      },
      { signal: socketCloseListener.signal },
    );
  }

  onCleanup(() => {
    socketCloseListener.abort();
    socket?.close();
  });

  return (
    <Show
      when={state().type === "ready"}
      fallback={
        <main ref={rootElement} class="flex h-screen flex-col items-center justify-center">
          <div class="flex flex-col items-center gap-y-2.5">
            <h1 class="mb-1 text-lg">
              joining room <span class="text-green">{roomName}!</span>
            </h1>
            <Input
              class="min-w-64"
              maxlength="20"
              placeholder="username"
              disabled={state().type !== "waiting"}
              value={username()}
              autofocus
              onInput={(event) => setUsername(event.target.value)}
              onEnter={() => {
                if (validUsername()) {
                  join();
                }
              }}
            />
            <Button
              size="lg"
              class="min-w-64"
              disabled={!validUsername() || state().type !== "waiting"}
              onClick={join}
            >
              Join
            </Button>
          </div>
        </main>
      }
    >
      <div class="quick-fade-in">
        <Game {...(state() as Variant<JoinGameState, "ready">).gameInfo} />
      </div>
    </Show>
  );
}

function Game({ uuid, room: roomInfo, sendMsg }: ServerMessageData<"Info"> & { sendMsg: SendFn }) {
  const roomName = useParams().name!;
  const [postGameInfo, setPostGameInfo] = createSignal<PostGameInfo | undefined>(undefined);
  const [messages, setMessages] = createSignal<Array<ChatMessage>>([]);
  const [room, setRoom] = createStore<Room>({
    uuid,
    clients: roomInfo.clients,
    owner: roomInfo.owner,
    settings: roomInfo.settings,
  });
  const [state, setState] = createStore<State>(convertStateMessage(roomInfo.state));

  useEvents({
    Error: (data) => {
      setMessages((messages) => [
        ...messages,
        { type: ChatMessageType.Error, content: data.content },
      ]);
    },
    RoomSettings: (data) => {
      const { type, ...settings } = data;
      setRoom("settings", settings);
    },
    ChatMessage: (data) => {
      setMessages((messages) => [
        ...messages,
        { type: ChatMessageType.Client, uuid: data.author, content: data.content },
      ]);
    },
    ConnectionUpdate: (data) => {
      if (data.state.type === "Connected" || data.state.type === "Reconnected") {
        const content = `${data.state.username} has joined`;
        setMessages((messages) => [...messages, { type: ChatMessageType.Info, content }]);

        const newClient = {
          uuid: data.uuid,
          username: data.state.username,
          disconnected: false,
        };
        setRoom("clients", (clients) => [
          ...clients.filter((client) => client.uuid !== data.uuid),
          newClient,
        ]);
      } else {
        const content = `${getUsername(room, data.uuid)} has left`;
        setMessages((messages) => [...messages, { type: ChatMessageType.Info, content }]);

        if (data.state.new_room_owner) {
          setRoom("owner", data.state.new_room_owner);
        }

        if (state.type !== "Lobby") {
          setRoom("clients", (client) => client.uuid == data.uuid, "disconnected", true);
        } else {
          setRoom("clients", (clients) => clients.filter((client) => client.uuid !== data.uuid));
        }
      }
    },
    GameStarted: (data) => {
      if (data.rejoin_token) {
        saveRejoinToken(roomName, data.rejoin_token);
      }

      setState(convertStateMessage(data.game));
    },
    GameEnded: (data) => {
      if (data.new_room_owner) {
        setRoom("owner", data.new_room_owner);
      }

      setPostGameInfo(data.info);
      setState({ type: "Lobby", ready: [], startingCountdown: undefined });
      setRoom("clients", (clients) => clients.filter((client) => !client.disconnected));
    },
  });

  return (
    <div class="flex h-screen flex-col">
      <GameNav sendMsg={sendMsg} room={() => room} />
      <Chat sendMsg={sendMsg} room={() => room} messages={messages} />
      <Switch>
        <Match when={state.type === "Lobby"}>
          <Lobby
            sendMsg={sendMsg}
            room={() => room}
            state={() => state}
            setState={setState}
            postGameInfo={postGameInfo()}
          />
        </Match>
        <Match when={state.type === "WordBomb"}>
          <WordBomb sendMsg={sendMsg} room={() => room} state={() => state} setState={setState} />
        </Match>
        <Match when={state.type === "Anagrams"}>
          <Anagrams sendMsg={sendMsg} room={() => room} state={() => state} setState={setState} />
        </Match>
      </Switch>
    </div>
  );
}
