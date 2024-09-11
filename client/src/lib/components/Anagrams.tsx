import { Accessor, createSignal, For, onCleanup, onMount } from "solid-js";
import { SetStoreFunction } from "solid-js/store";
import { useEvents } from "../events";
import { AnagramsState, Room, SendFn, State } from "../types/game";
import { AnagramsPlayerData } from "../types/messages";
import {
  calculateAnagramsPoints,
  colors,
  cubicEasing,
  getClient,
  translateAnagramsGuessError,
} from "../utils";
import { Avatar } from "./ui/Avatar";
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
  let inputElement!: HTMLInputElement;
  let guessErrorElement!: HTMLHeadingElement;
  let letterElementMap: { [anagramIndex: number]: HTMLHeadingElement } = {};

  const [game, setGame] = [
    state as Accessor<AnagramsState>,
    setState as SetStoreFunction<AnagramsState>,
  ];
  const [guessError, setGuessError] = createSignal("");

  const animateInput = (correct: boolean) => {
    inputElement.animate(
      { borderColor: [correct ? colors.green : colors.red, "rgb(255 255 255 / 0.1)"] },
      { easing: cubicEasing, duration: 800 },
    );
  };

  const animateError = () => {
    guessErrorElement.animate({ opacity: ["100%", "0%"] }, { easing: "ease-in", duration: 3000 });
  };

  useEvents({
    AnagramsInvalidGuess: (data) => {
      setGuessError(translateAnagramsGuessError(data.reason));

      animateInput(false);
      animateError();
    },
    AnagramsCorrectGuess: (data) => {
      setGame(
        "players",
        (player) => player.uuid === data.uuid,
        "used_words",
        (usedWords) => [...usedWords, data.guess],
      );

      if (data.uuid === room().uuid) {
        const anagramChars = [...game().anagram];

        let indexes: Array<number> = [];
        function pushCharIndex(character: string, lookAfter = 0) {
          const index = anagramChars.findIndex(
            (anagramsChar, i) => anagramsChar === character && i >= lookAfter,
          );

          if (index !== -1) {
            if (indexes.includes(index)) {
              pushCharIndex(character, index + 1);
            } else {
              indexes.push(index);
            }
          }
        }

        [...data.guess].forEach((character) => pushCharIndex(character));

        inputElement.value = "";
        animateInput(true);

        indexes.map((i) => {
          letterElementMap[i]!.animate(
            { backgroundColor: [colors.green, "transparent"] },
            { easing: cubicEasing, duration: 600 },
          );
        });
      }
    },
  });

  onMount(() => inputElement.focus());

  const controller = new AbortController();

  document.addEventListener(
    "keydown",
    (event) => {
      if (event.key === "Escape") {
        inputElement.focus();
        event.preventDefault();
      }
    },
    { signal: controller.signal },
  );

  onCleanup(() => controller.abort());

  return (
    <main class="flex h-screen items-center justify-center">
      <div class="relative flex items-center gap-x-8">
        <Leaderboard room={room} players={() => game().players} />
        <div class="w-[1px] self-stretch bg-dark-green/30"></div>
        <div class="flex font-mono text-[36px]">
          {[...game().anagram].map((char, i) => (
            <h1
              ref={letterElementMap[i]}
              classList={{ "border-l-0": i !== 0 }}
              class="flex h-16 w-16 items-center justify-center border"
            >
              {char}
            </h1>
          ))}
        </div>
        <h1
          ref={guessErrorElement}
          class="absolute -bottom-12 left-1/2 -translate-x-1/2 text-lg text-red-400 opacity-0"
        >
          {guessError()}
        </h1>
      </div>
      <Input
        ref={inputElement}
        size="lg"
        class="absolute bottom-6 left-1/2 -translate-x-1/2 focus-visible:border-white/10"
        placeholder={`words containing ${game().anagram}`}
        maxlength="6"
        disabled={
          !game()
            .players.map(({ uuid }) => uuid)
            .includes(room().uuid)
        }
        onEnter={(input) => {
          if (input.value.length !== 0) {
            sendMsg({ type: "AnagramsGuess", word: input.value });
          }
        }}
      />
    </main>
  );
}

function Leaderboard({
  room,
  players,
}: {
  room: Accessor<Room>;
  players: Accessor<Array<AnagramsPlayerData>>;
}) {
  return (
    <div class="flex max-h-96 w-64 flex-col gap-y-1.5 overflow-y-auto text-lg">
      <For
        each={players()
          .map(({ uuid, used_words }) => ({
            uuid,
            score: used_words.reduce((acc, word) => acc + calculateAnagramsPoints(word), 0),
          }))
          .sort((a, b) => b.score - a.score)}
      >
        {({ uuid, score }, i) => {
          const client = () => getClient(room(), uuid)!;

          return (
            <div
              classList={{ "opacity-50": client().disconnected }}
              class="flex items-center gap-x-1.5 transition-opacity"
            >
              <h1 class="tabular-nums">{i() + 1}.</h1>
              <Avatar username={client().username} size={25} />
              <h1 class="min-w-4 flex-1 truncate">{client().username}</h1>
              <h1 class="justify-self-end truncate text-light-green">{score}</h1>
            </div>
          );
        }}
      </For>
    </div>
  );
}
