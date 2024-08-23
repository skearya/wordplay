import { Accessor, For, onCleanup, onMount, Show } from "solid-js";
import { SetStoreFunction } from "solid-js/store";
import { Input } from "~/lib/components/ui/Input";
import { useEvents } from "~/lib/events";
import { Heart, LostHeart, SmallBomb } from "~/lib/icons";
import { Room, SendFn, State, WordBombState } from "~/lib/types/game";
import { Uuid, WordBombPlayerData } from "~/lib/types/messages";
import { getClient } from "~/lib/utils";
import { Avatar } from "./ui/Avatar";

const keyboard = [[..."qwertyuiop"], [..."asdfghjkl"], [..."zxcvbnm"]];
const easing = "cubic-bezier(0.33, 1, 0.68, 1)";
const green = "rgb(98 226 151)";
const darkGreen = "rgb(71 93 80)";
const red = "rgb(220 38 38)";
const orange = "rgb(234 88 12)";

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
  let bombOverlayElement!: HTMLInputElement;

  let overlayPosSet = false;
  const [game, setGame] = [
    state as Accessor<WordBombState>,
    setState as SetStoreFunction<WordBombState>,
  ];

  const animateInput = (correct: boolean) => {
    inputElement.animate(
      { borderColor: [correct ? green : red, "rgb(255 255 255 / 0.1)"] },
      { easing, duration: 800 },
    );
  };

  const animatePlayer = (uuid: Uuid, correct: boolean) => {
    const playerElement = document.getElementById(uuid);

    if (playerElement) {
      playerElement.animate(
        { color: [correct ? green : red, "rgb(255 255 255)"] },
        { easing, duration: 800 },
      );
    }
  };

  const animateOverlay = (exploded?: boolean) => {
    const playerElement = document.getElementById(game().turn);

    if (playerElement) {
      const rect = playerElement.getBoundingClientRect();
      const padding = 35;
      const positionProps = {
        top: `${rect.top - padding / 2}px`,
        left: `${rect.left - padding / 2}px`,
        width: `${rect.width + padding}px`,
        height: `${rect.height + padding}px`,
      };

      if (!overlayPosSet) {
        Object.assign(bombOverlayElement.style, positionProps);
        overlayPosSet = true;
      } else {
        bombOverlayElement.animate(
          {
            ...positionProps,
            ...(exploded && {
              borderColor: [orange, darkGreen],
              color: [orange, darkGreen],
            }),
          },
          { easing, fill: "forwards", duration: 600 },
        );
      }
    }
  };

  useEvents({
    WordBombInput: (data) => {
      const prevLen = game().players.find((player) => player.uuid === data.uuid)!.input.length;

      if ((prevLen == 0 && data.input.length >= 1) || (prevLen >= 1 && data.input.length === 0)) {
        requestAnimationFrame(() => animateOverlay());
      }

      setGame("players", (player) => player.uuid === data.uuid, "input", data.input);
    },
    WordBombInvalidGuess: (data) => {
      animatePlayer(data.uuid, false);

      if (data.uuid === room().uuid) {
        animateInput(false);
      }
    },
    WordBombPrompt: (data) => {
      const prevTurn = game().turn;

      setGame({ prompt: data.prompt, turn: data.turn });

      setGame(
        "players",
        (player) => player.uuid === prevTurn,
        "lives",
        (lives) => lives + data.life_change,
      );

      if (prevTurn === room().uuid && data.correct_guess) {
        setGame("usedLetters", (usedLetters) => new Set([...usedLetters!, ...data.correct_guess!]));

        const letters = keyboard.reduce((acc, keys) => acc + keys.length, 0);

        if (data.life_change > 0 && game().usedLetters!.size === letters) {
          setGame("usedLetters", new Set());
        }
      }

      const guessIsCorrect = data.correct_guess !== undefined ? true : false;

      if (prevTurn === room().uuid) {
        animateInput(guessIsCorrect);
      }
      animatePlayer(prevTurn, guessIsCorrect);
      animateOverlay(!guessIsCorrect);

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

    animateOverlay();
  });

  const controller = new AbortController();

  document.addEventListener(
    "keydown",
    (event) => {
      if (document.activeElement?.tagName === "INPUT") return;

      if (event.key === "Escape") {
        inputElement.focus();
        event.preventDefault();
      }
    },
    { signal: controller.signal },
  );

  window.addEventListener("resize", () => animateOverlay(), { signal: controller.signal });

  onCleanup(() => controller.abort());

  return (
    <main class="flex h-full flex-col justify-start">
      <div
        style="background: linear-gradient(185deg, rgba(38, 209, 108, 0.5) 7.28%, rgba(76, 118, 93, 0.1) 82.41%);"
        class="flex h-24 w-full items-center justify-center font-mono text-[34px]"
      >
        {game().prompt}
      </div>
      <div class="flex h-full w-full items-center justify-around">
        <For each={game().players}>{(player) => <Player room={room} player={player} />}</For>
      </div>
      <div
        ref={bombOverlayElement}
        class="absolute rounded-xl border border-dark-green text-dark-green"
      >
        <SmallBomb class="absolute bottom-1.5 left-1.5 -translate-x-1/2 translate-y-1/2" />
      </div>
      <Input
        ref={inputElement}
        size="lg"
        class="absolute bottom-6 left-1/2 -translate-x-1/2 focus-visible:border-white/10"
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
      class="flex flex-col items-center gap-y-2.5 text-xl transition-opacity"
    >
      <div class="flex items-center gap-x-4">
        <Avatar username={client().username} size={100} />
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
      <Show when={player.input.length !== 0}>
        <p class="quick-fade-in">{player.input}</p>
      </Show>
    </div>
  );
}

function Letters({ usedLetters }: { usedLetters: Accessor<Set<string>> }) {
  return (
    <div class="absolute bottom-6 right-6 flex flex-col space-y-1.5">
      {keyboard.map((row) => (
        <div class="flex justify-center gap-x-1.5">
          {row.map((key) => (
            <kbd
              classList={{ "bg-dark-green": usedLetters().has(key) }}
              class="flex h-9 w-9 items-center justify-center rounded-md border text-neutral-200 transition-colors duration-300"
            >
              {key}
            </kbd>
          ))}
        </div>
      ))}
    </div>
  );
}
