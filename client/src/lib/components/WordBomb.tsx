import { Accessor, For, onMount } from "solid-js";
import { SetStoreFunction } from "solid-js/store";
import { Input } from "~/lib/components/ui/Input";
import { Heart, LostHeart } from "~/lib/icons";
import { Room, SendFn, State, WordBombState } from "~/lib/types/game";
import { WordBombPlayerData } from "~/lib/types/messages";
import { getUsername } from "~/lib/utils";
import { useEvents } from "../events";

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
  let inputElement!: HTMLInputElement;

  const [game, setGame] = [
    state as Accessor<WordBombState>,
    setState as SetStoreFunction<WordBombState>,
  ];

  const animateInput = (color: "green" | "red") => {
    inputElement.animate(
      {
        borderColor: [
          color === "green" ? "rgb(98 226 151)" : "rgb(248 113 113)",
          inputElement.style.borderColor,
        ],
      },
      { duration: 800 },
    );
  };

  useEvents({
    WordBombInput: (data) => {
      setGame("players", (player) => player.uuid === data.uuid, "input", data.input);
    },
    WordBombInvalidGuess: (data) => {
      if (data.uuid === room().uuid) {
        animateInput("red");
      }
    },
    WordBombPrompt: (data) => {
      setGame(
        "players",
        (player) => player.uuid === game().turn,
        "lives",
        (lives) => lives + data.life_change,
      );

      if (game().turn === room().uuid) {
        if (data.correct_guess) {
          setGame(
            "usedLetters",
            (usedLetters) => new Set([...usedLetters!, ...data.correct_guess!]),
          );
          animateInput("green");
        } else {
          animateInput("red");
        }
      }

      setGame({ prompt: data.prompt, turn: data.turn });

      if (data.turn === room().uuid) {
        inputElement.value = "";
        inputElement.focus();
      }
    },
  });

  onMount(() => {
    if (game().turn === room().uuid) {
      inputElement.focus();
    }
  });

  return (
    <main class="mt-[82px] flex h-[calc(100vh_-_82px)] flex-col justify-start">
      <header
        style="background: linear-gradient(185deg, rgba(38, 209, 108, 0.5) 7.28%, rgba(76, 118, 93, 0.1) 82.41%);"
        class="flex h-24 w-full items-center justify-center font-mono text-[34px]"
      >
        {game().prompt}
      </header>
      <div class="flex h-full w-full items-center justify-around">
        <For each={game().players}>{(player) => <Player room={room} player={player} />}</For>
      </div>
      <Input
        ref={inputElement}
        class="absolute bottom-6 left-1/2 -translate-x-1/2 focus-visible:border-foreground disabled:opacity-100"
        placeholder={`a word containing ${game().prompt}`}
        maxlength="35"
        disabled={game().turn !== room().uuid}
        onInput={(event) => sendMsg({ type: "WordBombInput", input: event.target.value })}
        onEnter={(input) => sendMsg({ type: "WordBombGuess", word: input.value })}
      />
    </main>
  );
}

function Player({ room, player }: { room: Accessor<Room>; player: WordBombPlayerData }) {
  const username = getUsername(room(), player.uuid);

  return (
    <div class="flex h-min flex-col items-center gap-y-2.5 text-xl">
      <div class="flex items-center gap-x-4">
        <img
          src={`https://avatar.vercel.sh/${username}`}
          alt={`profile picture`}
          height={100}
          width={100}
          class="rounded-full"
        />
        <div class="flex flex-col gap-y-2">
          <h1>{username}</h1>
          <div class="flex gap-x-2">
            {Array.from({ length: player.lives }).map(() => (
              <Heart />
            ))}
            {Array.from({ length: 2 - player.lives }).map(() => (
              <LostHeart />
            ))}
          </div>
        </div>
      </div>
      <h1>{player.input}</h1>
    </div>
  );
}
