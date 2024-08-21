import { Accessor, For, Match, Show, Switch } from "solid-js";
import { SetStoreFunction, unwrap } from "solid-js/store";
import { Button } from "~/lib/components/ui/Button";
import { Copy } from "~/lib/components/ui/Copy";
import { Input } from "~/lib/components/ui/Input";
import { useEvents } from "~/lib/events";
import { Bomb } from "~/lib/icons";
import { LobbyState, Room, SendFn, State } from "~/lib/types/game";
import { PostGameInfo, Uuid } from "~/lib/types/messages";
import { getUsername, Variant } from "../utils";

export function Lobby({
  sendMsg,
  room,
  state,
  setState,
  postGameInfo,
}: {
  sendMsg: SendFn;
  room: Accessor<Room>;
  state: Accessor<State>;
  setState: SetStoreFunction<State>;
  postGameInfo: PostGameInfo | undefined;
}) {
  const [lobby, setLobby] = [
    state as Accessor<LobbyState>,
    setState as SetStoreFunction<LobbyState>,
  ];

  useEvents({
    ReadyPlayers: (data) => {
      setLobby("ready", data.ready);

      if (data.countdown_update) {
        setLobby(
          "startingCountdown",
          data.countdown_update.type === "InProgress" ? data.countdown_update.time_left : undefined,
        );
      }
    },
    StartingCountdown: (data) => {
      setLobby("startingCountdown", data.time_left);
    },
  });

  return (
    <main class="flex h-full items-center justify-center overflow-hidden">
      <div class="z-10 flex h-[480px] gap-x-4 rounded-xl border bg-[#0B0D0A] p-3.5">
        <Show when={postGameInfo}>
          <Switch>
            <Match when={postGameInfo!.type === "WordBomb"}>
              <WordBombPostGameInfo
                room={unwrap(room())}
                info={postGameInfo as Variant<PostGameInfo, "WordBomb">}
              />
            </Match>
            <Match when={postGameInfo!.type === "Anagrams"}>we arent there yet</Match>
          </Switch>
          <div class="w-[1px] scale-y-90 self-stretch bg-[#475D50]/30"></div>
        </Show>
        <div class="flex w-[475px] flex-col gap-y-2">
          <ReadyPlayers room={room} lobby={lobby} />
          <JoinButtons sendMsg={sendMsg} room={room} lobby={lobby} />
        </div>
      </div>
      <Status lobby={lobby} />
      <Practice />
    </main>
  );
}

function WordBombPostGameInfo({
  room,
  info,
}: {
  room: Room;
  info: Variant<PostGameInfo, "WordBomb">;
}) {
  return (
    <div class="flex flex-col gap-y-3.5">
      <Winner room={room} winner={info.winner} />
      <div class="grid grid-cols-2 gap-x-10 gap-y-2.5 overflow-y-scroll">
        {(
          [
            ["fastest guess", info.fastest_guesses],
            ["longest word", info.longest_words],
            ["wpm", info.avg_wpms],
            ["word length", info.avg_word_lengths],
          ] as const
        ).map(([title, items]) => (
          <>{items.length !== 0 && <Leaderboard room={room} title={title} items={items} />}</>
        ))}
      </div>
      <Stats
        minsElapsed={info.mins_elapsed}
        wordsUsed={info.words_used}
        lettersTyped={info.letters_typed}
      />
    </div>
  );
}

function Winner({ room, winner }: { room: Room; winner: Uuid }) {
  const username = getUsername(room, winner);

  return (
    <div
      style="background-image: linear-gradient(135deg, #171717 6.52%, transparent 6.52%, transparent 50%, #171717 50%, #171717 56.52%, transparent 56.52%, transparent 100%); background-size: 23.00px 23.00px;"
      class="flex min-w-72 items-center justify-between gap-x-2 rounded-lg border p-2.5"
    >
      <h1 class="text-lg font-medium">winner!</h1>
      <div class="flex items-center gap-x-4">
        <h1 class="text-lg">{username}</h1>
        <img
          src={`https://avatar.vercel.sh/${username}`}
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
  room,
  title,
  items,
}: {
  room: Room;
  title: string;
  items: Array<[Uuid, value: string | number]>;
}) {
  return (
    <div>
      <h1 class="mb-1.5">{title}</h1>
      <div class="w-48 space-y-1.5">
        {items.map(([uuid, value], i) => {
          const username = getUsername(room, uuid);

          return (
            <div class="flex items-center gap-x-1">
              <h1 class="text-sm tabular-nums">{i + 1}.</h1>
              <img
                src={`https://avatar.vercel.sh/${getUsername(room, uuid)}`}
                alt="profile picture"
                width={18}
                height={18}
                class="flex-none rounded-full"
              />
              <h1 class="flex-1 overflow-hidden text-ellipsis whitespace-nowrap text-sm">
                {username}
              </h1>
              <h1 class="justify-self-end text-sm text-light-green">{value}</h1>
            </div>
          );
        })}
      </div>
    </div>
  );
}

function Stats({
  minsElapsed,
  wordsUsed,
  lettersTyped,
}: {
  minsElapsed: number;
  wordsUsed: number;
  lettersTyped: number;
}) {
  return (
    <div class="mt-auto text-center [&>h1>span]:text-[#62E297]">
      <h1 class="flex-1 border-[#62E297]">
        <span>{minsElapsed}</span> mins elapsed
      </h1>
      <h1 class="flex-1 border-[#62E297]">
        <span>{wordsUsed}</span> words used
      </h1>
      <h1 class="flex-1">
        <span>{lettersTyped}</span> letters typed
      </h1>
    </div>
  );
}

function ReadyPlayers({ room, lobby }: { room: Accessor<Room>; lobby: Accessor<LobbyState> }) {
  return (
    <>
      <div class="flex items-baseline justify-between">
        <h1 class="text-xl">Ready Players</h1>
        <h1 class="text-lg text-green">{100 - lobby().ready.length} slots left</h1>
      </div>
      <Show
        when={room().clients.length !== 0}
        fallback={
          <div class="flex h-full flex-col items-center justify-center gap-y-2.5 text-light-green">
            <h1>maybe invite someone!</h1>
            <Copy color="muted" size="sm" content={window.location.href}>
              copy invite link
            </Copy>
          </div>
        }
      >
        <div class="grid grid-cols-2 gap-2.5 overflow-y-scroll">
          <For each={room().clients}>
            {(client) => {
              return (
                <div
                  classList={{
                    "bg-dark-green/30": lobby().ready.includes(client.uuid),
                    "bg-red-600/20": !lobby().ready.includes(client.uuid),
                  }}
                  class="flex items-center justify-between gap-x-4 rounded-lg border p-2 transition-colors"
                >
                  <img
                    src={`https://avatar.vercel.sh/${client.username}`}
                    alt="profile picture"
                    height={55}
                    width={55}
                    class="rounded-full"
                  />
                  <h1 class="overflow-hidden text-ellipsis whitespace-nowrap text-lg">
                    {client.username}
                  </h1>
                </div>
              );
            }}
          </For>
        </div>
      </Show>
    </>
  );
}

function JoinButtons({
  sendMsg,
  room,
  lobby,
}: {
  sendMsg: SendFn;
  room: Accessor<Room>;
  lobby: Accessor<LobbyState>;
}) {
  const ready = () => (lobby().ready.includes(room().uuid) ? "Unready" : "Ready");

  return (
    <div class="mt-auto flex gap-x-2.5">
      <Button size="lg" class="flex-1" onClick={() => sendMsg({ type: ready() })}>
        {ready()}
      </Button>
      <Show when={room().owner === room().uuid}>
        <Button
          color="secondary"
          size="lg"
          class="flex-1"
          disabled={lobby().ready.length < 2}
          onClick={() => sendMsg({ type: "StartEarly" })}
        >
          Start Early
        </Button>
      </Show>
    </div>
  );
}

function Practice() {
  return (
    <div class="absolute right-4 top-1/2 flex w-40 -translate-y-1/2 flex-col gap-y-3">
      <div class="flex items-center justify-between">
        <h3 class="text-light-green">practice</h3>
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
      <Input size="sm" placeholder="word" class="bg-transparent" />
    </div>
  );
}

function Status({ lobby }: { lobby: Accessor<LobbyState> }) {
  return (
    <div class="absolute bottom-0 right-0 flex flex-col items-end overflow-hidden">
      <div class="mr-3">
        <Bomb />
      </div>
      <h1 class="text-outline -skew-x-6 text-[4.5vw] leading-tight text-[#050705]">
        {lobby().startingCountdown
          ? `starting in ${lobby().startingCountdown}`
          : "waiting for players..."}
      </h1>
    </div>
  );
}
