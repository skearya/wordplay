import { useParams } from "@solidjs/router";
import { createSignal, Match, Show, Switch } from "solid-js";
import { createStore } from "solid-js/store";
import { Chat } from "~/lib/components/Chat";
import { Lobby } from "~/lib/components/Lobby";
import { GameNav } from "~/lib/components/Nav";
import { WordBomb } from "~/lib/components/WordBomb";
import { callEventListeners, ServerMessageData, useEvent, useEvents } from "~/lib/events";
import { Room, SendFn, State } from "~/lib/types/game";
import { ClientMessage, PostGameInfo } from "~/lib/types/messages";
import { getUsername, roomStateToCamelCase } from "~/lib/utils";

export default function JoinGame() {
  const roomName = useParams().name;
  const [username, setUsername] = createSignal(localStorage.getItem("username") ?? "");
  const [gameInfo, setGameInfo] = createSignal<Parameters<typeof Game>[0] | undefined>(undefined);
  const canJoin = () => username().length <= 20 && username() !== "";

  function join() {
    localStorage.setItem("username", username());

    const rejoinToken: string | undefined = JSON.parse(
      localStorage.getItem("rejoinTokens") ?? "{}",
    )[roomName];

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
        setGameInfo({
          ...data,
          sendMsg: (message: ClientMessage) => socket.send(JSON.stringify(message)),
        });

        removeInfoHandler();
      },
      false,
    );

    // TODO: actually have error handling
    socket.addEventListener("close", (event) => {
      throw new Error(event.reason ?? "Unknown error");
    });

    if (import.meta.hot) {
      import.meta.hot.on("vite:beforeUpdate", () => socket.close());
    }
  }

  return (
    <Show
      when={gameInfo()}
      fallback={
        <main class="flex h-screen flex-col items-center justify-center">
          <div class="flex min-w-64 flex-col gap-y-2.5">
            <input
              type="text"
              maxlength="20"
              placeholder="username"
              autofocus
              class="rounded-lg border bg-transparent px-3 py-2.5"
              value={username()}
              onInput={(event) => setUsername(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter" && canJoin()) {
                  join();
                }
              }}
            />
            <button
              class="rounded-lg border bg-[#475D50] py-3 font-medium transition-opacity disabled:opacity-50"
              disabled={!canJoin()}
              onClick={join}
            >
              Join
            </button>
          </div>
        </main>
      }
    >
      <Game {...gameInfo()!} />
    </Show>
  );
}

function Game({ uuid, room: roomInfo, sendMsg }: ServerMessageData<"Info"> & { sendMsg: SendFn }) {
  const postGameInfo: PostGameInfo | undefined = undefined;
  // {
  //   type: "WordBomb",
  //   winner: "2",
  //   mins_elapsed: 21,
  //   words_used: 2121,
  //   letters_typed: 212121,
  //   fastest_guesses: [],
  //   longest_words: [],
  //   avg_wpms: [],
  //   avg_word_lengths: [],
  // }

  const [room, setRoom] = createStore<Room>({
    uuid,
    clients: roomInfo.clients,
    owner: roomInfo.owner,
    settings: roomInfo.settings,
  });
  const [state, setState] = createStore<State>(roomStateToCamelCase(roomInfo.state));
  const [messages, setMessages] = createSignal<Array<string>>([]);

  useEvents({
    Error: (data) => {
      window.alert(data.content);
    },
    RoomSettings: (data) => {
      const { type, ...settings } = data;
      setRoom("settings", settings);
    },
    ConnectionUpdate: (data) => {
      if (data.state.type === "Connected" || data.state.type === "Reconnected") {
        const message = `${data.state.username} has joined`;
        setMessages((messages) => [...messages, message]);

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
        setMessages((messages) => [...messages, message]);

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
  });

  return (
    <>
      <GameNav room={() => room} />
      <Chat sendMsg={sendMsg} room={() => room} messages={messages} setMessages={setMessages} />
      <Switch>
        <Match when={state.type === "Lobby"}>
          <Lobby
            sendMsg={sendMsg}
            room={() => room}
            state={() => state}
            setState={setState}
            postGameInfo={postGameInfo}
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
