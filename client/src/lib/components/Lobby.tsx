import { Accessor, For, Show } from "solid-js";
import { SetStoreFunction } from "solid-js/store";
import { Button } from "~/lib/components/ui/Button";
import { Copy } from "~/lib/components/ui/Copy";
import { Input } from "~/lib/components/ui/Input";
import { useEvents } from "~/lib/events";
import { Bomb } from "~/lib/icons";
import { LobbyState, Room, SendFn, State } from "~/lib/types/game";
import { PostGameInfo } from "~/lib/types/messages";

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
    <main class="flex h-screen items-center justify-center overflow-hidden">
      <div class="z-10 flex h-[480px] gap-x-4 rounded-xl border bg-[#0B0D0A] p-3.5">
        <Show when={postGameInfo}>
          <div class="flex flex-col gap-y-3.5">
            <Winner />
            <div class="grid grid-cols-2 gap-x-10 gap-y-2.5 overflow-y-scroll">
              {["fastest guess", "longest word", "wpm", "word length"].map((title) => (
                <Leaderboard title={title} items={[]} />
              ))}
            </div>
            <Stats />
          </div>
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

function ReadyPlayers({ room, lobby }: { room: Accessor<Room>; lobby: Accessor<LobbyState> }) {
  return (
    <>
      <div class="flex items-baseline justify-between">
        <h1 class="text-xl">Ready Players</h1>
        <h1 class="text-lg text-[#26D16C]">{100 - lobby().ready.length} slots left</h1>
      </div>
      <Show
        when={lobby().ready.length !== 0}
        fallback={
          <div class="flex h-full flex-col items-center justify-center gap-y-3 text-light-green">
            <h1>maybe invite someone!</h1>
            <Copy color="muted" size="sm" content={window.location.href}>
              copy invite link
            </Copy>
          </div>
        }
      >
        <div class="grid grid-cols-2 gap-2.5 overflow-y-scroll">
          <For each={lobby().ready}>
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
                  <h1 class="overflow-hidden text-ellipsis whitespace-nowrap text-lg">
                    {username}
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
