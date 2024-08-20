import { Accessor, For, onMount } from "solid-js";
import { SetStoreFunction } from "solid-js/store";
import { Input } from "~/lib/components/ui/Input";
import { useEvents } from "~/lib/events";
import { Heart, LostHeart } from "~/lib/icons";
import { Room, SendFn, State, WordBombState } from "~/lib/types/game";
import { Uuid, WordBombPlayerData } from "~/lib/types/messages";
import { getClient } from "~/lib/utils";

const green = "rgb(98 226 151)";
const red = "rgb(220 38 38)";

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

  const animateInput = (correct: boolean) => {
    inputElement.animate(
      { borderColor: [correct ? green : red, "rgb(255 255 255 / 0.1)"] },
      { duration: 800 },
    );
  };

  const animatePlayer = (uuid: Uuid, correct: boolean) => {
    const playerElement = document.getElementById(uuid);

    if (playerElement) {
      playerElement.animate(
        { color: [correct ? green : red, "rgb(255 255 255)"] },
        { duration: 800 },
      );
    }
  };

  useEvents({
    WordBombInput: (data) => {
      setGame("players", (player) => player.uuid === data.uuid, "input", data.input);
    },
    WordBombInvalidGuess: (data) => {
      animatePlayer(data.uuid, false);

      if (data.uuid === room().uuid) {
        animateInput(false);
      }
    },
    WordBombPrompt: (data) => {
      setGame(
        "players",
        (player) => player.uuid === game().turn,
        "lives",
        (lives) => lives + data.life_change,
      );

      animatePlayer(game().turn, data.correct_guess ? true : false);

      if (game().turn === room().uuid) {
        if (data.correct_guess) {
          setGame(
            "usedLetters",
            (usedLetters) => new Set([...usedLetters!, ...data.correct_guess!]),
          );

          animateInput(true);
        } else {
          animateInput(false);
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
      <div
        style="background: linear-gradient(185deg, rgba(38, 209, 108, 0.5) 7.28%, rgba(76, 118, 93, 0.1) 82.41%);"
        class="flex h-24 w-full items-center justify-center font-mono text-[34px]"
      >
        {game().prompt}
      </div>
      <div class="flex h-full w-full items-center justify-around">
        <For each={game().players}>{(player) => <Player room={room} player={player} />}</For>
      </div>
      <Input
        ref={inputElement}
        size="lg"
        class="absolute bottom-6 left-1/2 -translate-x-1/2 transition-all duration-[800ms] ease-[ease] focus-visible:border-foreground"
        placeholder={`a word containing ${game().prompt}`}
        maxlength="35"
        disabled={game().turn !== room().uuid}
        onInput={(event) => sendMsg({ type: "WordBombInput", input: event.target.value })}
        onEnter={(input) => sendMsg({ type: "WordBombGuess", word: input.value })}
      />
      <Letters usedLetters={() => game().usedLetters!} />
    </main>
  );
}

function Player({ room, player }: { room: Accessor<Room>; player: WordBombPlayerData }) {
  const client = () => getClient(room(), player.uuid)!;

  return (
    <div
      id={player.uuid}
      classList={{ "opacity-50": client().disconnected }}
      class="flex h-min flex-col items-center gap-y-2.5 text-xl transition-opacity"
    >
      <div class="flex items-center gap-x-4">
        <img
          src={`https://avatar.vercel.sh/${client().username}`}
          alt={`profile picture`}
          height={100}
          width={100}
          class="rounded-full"
        />
        <div class="flex flex-col gap-y-2">
          <h1>{client().username}</h1>
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

function Letters({ usedLetters }: { usedLetters: Accessor<Set<string>> }) {
  const keyboard = [[..."qwertyuiop"], [..."asdfghjkl"], [..."zxcvbnm"]];

  return (
    <div class="absolute bottom-6 right-6 flex flex-col space-y-1.5">
      {keyboard.map((row) => (
        <div class="flex justify-center gap-x-1.5">
          {row.map((key) => (
            <kbd
              classList={{ "bg-dark-green": usedLetters().has(key) }}
              class="flex h-8 w-8 items-center justify-center rounded border transition-colors"
            >
              {key}
            </kbd>
          ))}
        </div>
      ))}
    </div>
  );
}
