import { Accessor } from "solid-js";
import { SetStoreFunction } from "solid-js/store";
import { useEvents } from "../events";
import { AnagramsState, Room, SendFn, State } from "../types/game";
import { Input } from "./ui/Input";

export function Anagrams({
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
    state as Accessor<AnagramsState>,
    setState as SetStoreFunction<AnagramsState>,
  ];

  useEvents({
    AnagramsInvalidGuess: () => {},
    AnagramsCorrectGuess: () => {},
  });

  return (
    <main class="flex h-screen flex-col items-center justify-center gap-y-2">
      <h1 class="font-mono text-[34px] tracking-wide">ABCDEF</h1>
      <Input placeholder="word with ABCDEF" />
    </main>
  );
}
