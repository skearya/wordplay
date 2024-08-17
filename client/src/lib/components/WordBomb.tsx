import { Accessor } from "solid-js";
import { SetStoreFunction } from "solid-js/store";
import { Heart, LostHeart } from "~/lib/icons";
import { Room, SendFn, State, WordBombState } from "~/lib/types/game";

export function WordBomb({
  sendMsg,
  room,
  state,
  setState,
}: {
  sendMsg: SendFn;
  room: Accessor<Room>;
  state: Accessor<State>;
  setState: SetStoreFunction<State>;
}) {
  const [game, setGame] = [
    state as Accessor<WordBombState>,
    setState as SetStoreFunction<WordBombState>,
  ];

  return (
    <main class="mt-[82px] flex h-[calc(100vh_-_82px)] flex-col justify-start">
      <header
        style="background: linear-gradient(185deg, rgba(38, 209, 108, 0.5) 7.28%, rgba(76, 118, 93, 0.1) 82.41%);"
        class="flex h-24 w-full items-center justify-center font-mono text-[34px]"
      >
        UTS
      </header>
      <div class="flex h-full w-full items-center justify-around">
        {Array.from({ length: 3 }).map(() => (
          <Player />
        ))}
      </div>
    </main>
  );
}

function Player() {
  return (
    <div class="flex h-min flex-col items-center gap-y-2.5 text-xl">
      <div class="flex items-center gap-x-4">
        <img
          src={`https://avatar.vercel.sh/skeary`}
          alt={`profile picture`}
          height={100}
          width={100}
          class="rounded-full"
        />
        <div class="flex flex-col gap-y-2">
          <h1>Player 1</h1>
          <div class="flex gap-x-2">
            <Heart />
            <LostHeart />
          </div>
        </div>
      </div>
      <h1>cats</h1>
    </div>
  );
}
