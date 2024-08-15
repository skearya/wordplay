import { useParams } from "@solidjs/router";
import { createSignal, Match, Show, Switch } from "solid-js";
import { createStore } from "solid-js/store";
import { Chat } from "~/lib/components/Chat";
import { Lobby } from "~/lib/components/Lobby";
import { GameNav } from "~/lib/components/Nav";
import { WordBomb } from "~/lib/components/WordBomb";
import { callEventListeners, ServerMessageData, useEvent, useEvents } from "~/lib/events";
import { ChatMessage, Room, SendFn, State } from "~/lib/types/game";
import { ClientMessage, PostGameInfo } from "~/lib/types/messages";
import { convertStateMessage, getRejoinToken, getUsername, saveRejoinToken } from "~/lib/utils";

type JoinGameState =
  | {
      type: "waiting" | "connecting" | "fading out";
    }
  | {
      type: "ready";
      gameInfo: Parameters<typeof Game>[0];
    };

export default function JoinGame() {
  let rootElement!: HTMLElement;

  const roomName = useParams().name;
  const [state, setState] = createSignal<JoinGameState>({ type: "waiting" });
  const [username, setUsername] = createSignal(localStorage.getItem("username") ?? "");
  const validUsername = () => username().length <= 20 && username() !== "";

  function join() {
    setState({ type: "connecting" });

    localStorage.setItem("username", username());

    const rejoinToken = getRejoinToken(roomName);
    const params = new URLSearchParams({
      username: username(),
      ...(rejoinToken && { rejoin_token: rejoinToken }),
    });

    const socket = new WebSocket(
      `${(import.meta.env.PUBLIC_SERVER as string).replace(
        "http",
        "ws",
      )}/rooms/${roomName}?${params}`,
    );

    socket.addEventListener("message", (event) => {
      callEventListeners(JSON.parse(event.data));
    });

    const removeInfoHandler = useEvent(
      "Info",
      (data) => {
        removeInfoHandler();

        const gameInfo = {
          ...data,
          sendMsg: (message: ClientMessage) => socket.send(JSON.stringify(message)),
        };

        rootElement.classList.add("fade-out");
        rootElement.addEventListener("animationend", () => setState({ type: "ready", gameInfo }), {
          once: true,
        });
      },
      false,
    );

    // TODO: actually have error handling
    socket.addEventListener("close", (event) => {
      throw new Error(event.reason ?? "Unknown error");
    });
  }

  // TODO: show game info

  return (
    <Show
      when={state().type === "ready"}
      fallback={
        <main ref={rootElement} class="flex h-screen flex-col items-center justify-center">
          <div class="flex min-w-64 flex-col gap-y-2.5">
            <input
              type="text"
              maxlength="20"
              placeholder="username"
              disabled={state().type !== "waiting"}
              value={username()}
              autofocus
              class="rounded-lg border bg-transparent px-3 py-2.5 transition-opacity disabled:opacity-50"
              onInput={(event) => setUsername(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter" && validUsername()) {
                  join();
                }
              }}
            />
            <button
              disabled={!validUsername() || state().type !== "waiting"}
              class="rounded-lg border bg-[#475D50] py-3 font-medium transition-opacity disabled:opacity-50"
              onClick={join}
            >
              Join
            </button>
          </div>
        </main>
      }
    >
      <div class="fade-in">
        <Game {...(state() as Extract<JoinGameState, { type: "ready" }>).gameInfo} />
      </div>
    </Show>
  );
}

function Game({ uuid, room: roomInfo, sendMsg }: ServerMessageData<"Info"> & { sendMsg: SendFn }) {
  const roomName = useParams().name;
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
      window.alert(data.content);
    },
    RoomSettings: (data) => {
      const { type, ...settings } = data;
      setRoom("settings", settings);
    },
    ChatMessage: (data) => {
      const message = `${getUsername(room, data.author)}: ${data.content}`;
      setMessages((messages) => [...messages, [message, false]]);
    },
    ConnectionUpdate: (data) => {
      if (data.state.type === "Connected" || data.state.type === "Reconnected") {
        const message = `${data.state.username} has joined`;
        setMessages((messages) => [...messages, [message, true]]);

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
        const message = `${getUsername(room, data.uuid)} has left`;
        setMessages((messages) => [...messages, [message, true]]);

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
    },
  });

  return (
    <>
      <GameNav room={() => room} />
      <Chat sendMsg={sendMsg} messages={messages} />
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
          <WordBomb />
        </Match>
        <Match when={state.type === "Anagrams"}>
          <h1>anagrams unimplemented</h1>
        </Match>
      </Switch>
    </>
  );
}
