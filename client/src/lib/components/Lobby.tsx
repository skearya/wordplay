import { Accessor, For, Show } from "solid-js";
import { SetStoreFunction } from "solid-js/store";
import { useEvent } from "~/lib/events";
import { LobbyState, Room, SendFn } from "~/lib/types/game";
import { PostGameInfo } from "~/lib/types/messages";
import { Bomb } from "../icons";

export function Lobby({
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
                <Leaderboard title={title} items={[]} />
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
