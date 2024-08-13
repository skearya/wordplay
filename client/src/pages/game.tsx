import { useParams } from "@solidjs/router";
import { Accessor, createSignal, For, Match, Setter, Show, Switch } from "solid-js";
import { createStore, SetStoreFunction } from "solid-js/store";
import { callEventListeners, ServerMessageData, useEvent, useEvents } from "~/lib/events";
import { Bomb, Link, QuestionMark } from "~/lib/icons";
import {
  AnagramsPlayerData,
  ClientInfo,
  ClientMessage,
  PostGameInfo,
  RoomSettings,
  Uuid,
  WordBombPlayerData,
} from "~/lib/types/messages";
import { getUsername, roomStateToCamelCase } from "~/lib/utils";

export default function JoinGame() {
  const roomName = useParams().name;
  const [username, setUsername] = createSignal(localStorage.getItem("username") ?? "");
  const [gameInfo, setGameInfo] = createSignal<Parameters<typeof Game>["0"] | undefined>(undefined);
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
      // TODO
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

const guesses = [
  {
    pfp: "https://avatar.vercel.sh/skeary",
    name: "skeary",
    content: "4.3s",
  },
  {
    pfp: "https://avatar.vercel.sh/skeary2",
    name: "skeary2efafnjewknfkn",
    content: "4s",
  },
  {
    pfp: "https://avatar.vercel.sh/skeary3",
    name: "skeary3",
    content: ".3s",
  },
];

type SendFn = (message: ClientMessage) => void;

export type Room = {
  uuid: Uuid;
  clients: ClientInfo[];
  owner: Uuid;
  settings: RoomSettings;
};

type LobbyState = {
  type: "Lobby";
  ready: Array<Uuid>;
  startingCountdown: number | undefined;
};

type WordBombState = {
  type: "WordBomb";
  players: Array<WordBombPlayerData>;
  turn: Uuid;
  prompt: string;
  usedLetters: Array<string> | undefined;
};

type AnagramsState = {
  type: "Anagrams";
  players: Array<AnagramsPlayerData>;
  anagram: string;
};

type State = LobbyState | WordBombState | AnagramsState;

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
      <GameNav />
      <Chat sendMsg={sendMsg} room={() => room} messages={messages} setMessages={setMessages} />
      <Switch>
        <Match when={state.type === "Lobby"}>
          <Lobby
            sendMsg={sendMsg}
            room={() => room}
            state={() => state as LobbyState}
            setState={setState as SetStoreFunction<LobbyState>}
            postGameInfo={postGameInfo}
          />
        </Match>
      </Switch>
    </>
  );
}

function Lobby({
  sendMsg,
  room,
  state,
  setState,
  postGameInfo,
}: {
  sendMsg: SendFn;
  room: Accessor<Room>;
  state: Accessor<LobbyState>;
  setState: SetStoreFunction<LobbyState>;
  postGameInfo: PostGameInfo | undefined;
}) {
  useEvent("ReadyPlayers", (data) => {
    setState("ready", data.ready);
  });

  return (
    <main class="flex h-screen items-center justify-center overflow-hidden">
      <div class="z-10 flex h-[480px] gap-x-4 rounded-xl border bg-[#0B0D0A] p-3.5">
        <Show when={postGameInfo}>
          <div class="flex flex-col gap-y-3.5">
            <Winner />
            <div class="grid grid-cols-2 gap-x-10 gap-y-2.5 overflow-y-scroll">
              {["fastest guess", "longest word", "wpm", "word length"].map((title) => (
                <Leaderboard title={title} items={guesses} />
              ))}
            </div>
            <Stats />
          </div>
          <div class="w-[1px] scale-y-90 self-stretch bg-[#475D50]/30"></div>
        </Show>
        <div class="flex w-[475px] flex-col gap-y-2">
          <ReadyPlayers room={room} state={state} />
          <JoinButtons sendMsg={sendMsg} room={room} state={state} />
        </div>
      </div>
      <Status />
      <Practice />
    </main>
  );
}

function GameNav() {
  return (
    <nav class="absolute top-0 flex w-full items-center justify-between px-6 py-5">
      <h1 class="text-2xl">wordplay</h1>
      <div class="flex items-center gap-x-5">
        <div
          style="box-shadow: 0px 0px 15.5px 1px #26D16C"
          class="h-[13px] w-[13px] rounded-full bg-[#26D16C]"
        />
        <h1 class="text-[#B1C1AE]">44ms</h1>
        <div class="flex -space-x-2">
          {Array.from({ length: 3 }).map((_, i) => (
            <img
              src={`https://avatar.vercel.sh/${i}`}
              alt={`${i}`}
              height={38}
              width={38}
              class="rounded-full border-[3px] border-black"
            />
          ))}
        </div>
        <Link />
        <QuestionMark />
      </div>
    </nav>
  );
}

function Chat({
  sendMsg,
  room,
  messages,
  setMessages,
}: {
  sendMsg: SendFn;
  room: Accessor<Room>;
  messages: Accessor<Array<string>>;
  setMessages: Setter<Array<string>>;
}) {
  useEvent("ChatMessage", (data) => {
    const message = `${getUsername(room(), data.author)}: ${data.content}`;
    setMessages((messages) => [...messages, message]);
  });

  return (
    <div class="bg-primary-50/25 fixed bottom-0 left-0 z-50 flex w-96 flex-col rounded-tr-lg border-r border-t">
      <ul class="m-2 mb-0 list-item h-48 overflow-y-auto text-wrap break-all">
        <For each={messages()}>{(message) => <li>{message}</li>}</For>
      </ul>
      <input
        class="m-2 h-10 rounded-lg border bg-transparent px-2.5 py-2 placeholder-white/50"
        type="text"
        maxlength="250"
        placeholder="send a message..."
        onKeyDown={(event) => {
          if (event.key === "Enter") {
            sendMsg({ type: "ChatMessage", content: (event.target as HTMLInputElement).value });
          }
        }}
      />
    </div>
  );
}

function Tabs() {
  return (
    <div class="flex overflow-hidden rounded-lg border text-center">
      <button class="flex-1 rounded-lg bg-[#475D50] py-2">Info</button>
      <button class="flex-1 rounded-lg py-2">Settings</button>
    </div>
  );
}

function Winner() {
  return (
    <div
      style="background-image: linear-gradient(135deg, #171717 6.52%, transparent 6.52%, transparent 50%, #171717 50%, #171717 56.52%, transparent 56.52%, transparent 100%); background-size: 23.00px 23.00px;"
      class="flex items-center justify-between rounded-lg border p-2.5"
    >
      <h1 class="text-lg font-medium">winner!</h1>
      <div class="flex items-center gap-x-4">
        <h1 class="text-lg">Player 1</h1>
        <img
          src="https://avatar.vercel.sh/skeary"
          alt="profile picture"
          width={50}
          height={50}
          class="rounded-full"
        />
      </div>
    </div>
  );
}

function Leaderboard({
  title,
  items,
}: {
  title: string;
  items: Array<{
    pfp: string;
    name: string;
    content: string;
  }>;
}) {
  return (
    <div>
      <h1 class="mb-1.5">{title}</h1>
      <div class="w-48 space-y-1.5">
        {items.map(({ pfp, name, content }, i) => (
          <div class="flex items-center gap-x-1">
            <h1 class="text-sm tabular-nums">{i + 1}.</h1>
            <img
              src={pfp}
              alt="profile picture"
              width={18}
              height={18}
              class="flex-none rounded-full"
            />
            <h1 class="flex-1 overflow-hidden text-ellipsis whitespace-nowrap text-sm">{name}</h1>
            <h1 class="justify-self-end text-sm text-[#8BA698]">{content}</h1>
          </div>
        ))}
      </div>
    </div>
  );
}

function Stats() {
  return (
    <div class="mt-auto text-center [&>h1>span]:text-[#62E297]">
      <h1 class="flex-1 border-[#62E297]">
        <span>21</span> mins elapsed
      </h1>
      <h1 class="flex-1 border-[#62E297]">
        <span>209</span> words used
      </h1>
      <h1 class="flex-1">
        <span>2032</span> letters typed
      </h1>
    </div>
  );
}

function ReadyPlayers({ room, state }: { room: Accessor<Room>; state: Accessor<LobbyState> }) {
  return (
    <>
      <div class="flex items-baseline justify-between">
        <h1 class="text-xl">Ready Players</h1>
        <h1 class="text-lg text-[#26D16C]">{100 - state().ready.length} slots left</h1>
      </div>
      <div class="grid grid-cols-2 gap-2.5 overflow-y-scroll">
        <For each={state().ready}>
          {(uuid) => {
            const username = room().clients.find((client) => client.uuid === uuid)!.username;

            return (
              <div class="flex items-center justify-between gap-x-4 rounded-lg border bg-[#475D50]/30 p-2">
                <img
                  src={`https://avatar.vercel.sh/${username}`}
                  alt={`profile picture`}
                  height={55}
                  width={55}
                  class="rounded-full"
                />
                <h1 class="overflow-hidden text-ellipsis whitespace-nowrap text-lg">{username}</h1>
              </div>
            );
          }}
        </For>
      </div>
    </>
  );
}

function JoinButtons({
  sendMsg,
  room,
  state,
}: {
  sendMsg: SendFn;
  room: Accessor<Room>;
  state: Accessor<LobbyState>;
}) {
  const ready = () => (state().ready.includes(room().uuid) ? "Unready" : "Ready");

  return (
    <div class="mt-auto flex gap-x-2.5">
      <button
        class="flex-1 rounded-lg border bg-[#475D50] py-4 font-medium"
        onClick={() => sendMsg({ type: ready() })}
      >
        {ready()}
      </button>
      <Show when={room().owner === room().uuid}>
        <button
          class="flex-1 rounded-lg border bg-[#345C8A] py-4 font-medium transition-all disabled:opacity-50"
          disabled={state().ready.length < 2}
          onClick={() => sendMsg({ type: "StartEarly" })}
        >
          Start Early
        </button>
      </Show>
    </div>
  );
}

function Practice() {
  return (
    <div class="absolute right-4 top-1/2 flex w-40 -translate-y-1/2 flex-col gap-y-3">
      <div class="flex items-center justify-between">
        <h3 class="text-[#8BA698]">practice</h3>
        <div
          style="background: linear-gradient(244.26deg, rgba(38, 209, 108, 0.5) 7.28%, rgba(76, 118, 93, 0.1) 82.41%);"
          class="rounded-lg px-2.5 py-0.5 font-mono text-lg"
        >
          UTS
        </div>
      </div>
      <div
        style="background: linear-gradient(to right, #26D16C 70%, transparent 30%)"
        class="h-[1px] w-full"
      ></div>
      <input type="text" placeholder="word" class="rounded-lg border bg-transparent px-2.5 py-2" />
    </div>
  );
}

function Status() {
  return (
    <div class="absolute bottom-0 right-0 flex flex-col items-end overflow-hidden">
      <div class="mr-3">
        <Bomb />
      </div>
      <h1 class="text-outline -skew-x-6 text-[4.5vw] leading-tight text-[#050705]">
        waiting for players...
      </h1>
    </div>
  );
}
