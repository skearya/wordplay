import { Accessor, createSignal, onMount } from "solid-js";
import { Settings as SettingsIcon } from "../icons";
import { Room, SendFn } from "../types/game";
import { Games } from "../types/messages";
import { Button } from "./ui/Button";
import { Select } from "./ui/Select";

export function Settings({ sendMsg, room }: { sendMsg: SendFn; room: Accessor<Room> }) {
  let gameElement!: HTMLSelectElement;
  let wordBombDifficultyElement!: HTMLSelectElement;

  const [visible, setVisible] = createSignal(false);
  const notRoomOwner = () => room().owner !== room().uuid;

  // https://github.com/solidjs/solid/issues/1754
  onMount(() => {
    gameElement.value = room().settings.game.toString();
    wordBombDifficultyElement.value = room().settings.word_bomb.min_wpm.toString();
  });

  return (
    <div
      classList={{ "-translate-x-full": !visible() }}
      class="fixed top-1/2 left-0 z-50 -translate-y-1/2 transition-transform"
    >
      <div class="bg-light-background relative flex min-w-52 flex-col gap-y-2.5 rounded-r-lg border border-l-0 p-3.5">
        <div class="space-y-1.5">
          <h1 class="text-light-green pb-0.5 text-lg">Room</h1>
          <div class="flex items-center justify-between">
            <label for="game">game</label>
            <Select
              ref={gameElement}
              size="xs"
              name="game"
              id="game"
              value={room().settings.game}
              disabled={notRoomOwner()}
              onChange={(event) => {
                sendMsg({
                  type: "RoomSettings",
                  ...room().settings,
                  game: event.target.value as Games,
                });
              }}
            >
              <option value="WordBomb">Word Bomb</option>
              <option value="Anagrams">Anagrams</option>
            </Select>
          </div>
          <div class="flex items-center justify-between">
            <label for="visibility">public</label>
            <input
              type="checkbox"
              name="visibility"
              id="visibility"
              checked={room().settings.public}
              disabled={notRoomOwner()}
              onChange={(event) => {
                sendMsg({ type: "RoomSettings", ...room().settings, public: event.target.checked });
              }}
            />
          </div>
        </div>
        <div class="bg-dark-green/30 h-px w-full"></div>
        <div class="space-y-1.5">
          <h1 class="text-light-green pb-0.5 text-lg">Word Bomb</h1>
          <div class="flex items-center justify-between">
            <label for="min-wpp">difficulty</label>
            <Select
              ref={wordBombDifficultyElement}
              size="xs"
              name="min-wpp"
              id="min-wpp"
              value={room().settings.word_bomb.min_wpm}
              disabled={notRoomOwner()}
              onChange={(event) => {
                const wpp = parseInt(event.target.value);

                sendMsg({
                  type: "RoomSettings",
                  ...room().settings,
                  word_bomb: {
                    ...room().settings.word_bomb,
                    min_wpm: wpp,
                  },
                });
              }}
            >
              <optgroup label="min 1000wpp (words per prompt)">
                <option value="1000">very easy</option>
              </optgroup>
              <optgroup label="min 500wpp">
                <option value="500">easy</option>
              </optgroup>
              <optgroup label="min 300wpp">
                <option value="300">medium</option>
              </optgroup>
              <optgroup label="min 100wpp">
                <option value="100">hard</option>
              </optgroup>
            </Select>
          </div>
        </div>
        <Button
          color="muted"
          size="sm"
          class="text-light-green absolute top-1/2 -right-2.5 translate-x-full -translate-y-1/2 border-none bg-transparent p-0"
          onClick={() => setVisible((visible) => !visible)}
        >
          <SettingsIcon />
        </Button>
      </div>
    </div>
  );
}
