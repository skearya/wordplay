import { Accessor, createSignal, onMount } from "solid-js";
import { Settings as SettingsIcon } from "../icons";
import { Room, SendFn } from "../types/game";
import { Games } from "../types/messages";
import { Button } from "./ui/Button";
import { Select } from "./ui/Select";

export function Settings({ sendMsg, room }: { sendMsg: SendFn; room: Accessor<Room> }) {
  let wordBombDifficultyElement!: HTMLSelectElement;

  const [visible, setVisible] = createSignal(false);
  const notRoomOwner = () => room().owner !== room().uuid;

  // https://github.com/solidjs/solid/issues/1754
  onMount(() => {
    wordBombDifficultyElement.value = room().settings.word_bomb.min_wpm.toString();
  });

  return (
    <div
      classList={{ "-translate-x-full": !visible() }}
      class="fixed left-0 top-1/2 z-50 -translate-y-1/2 transition-transform"
    >
      <div class="relative flex min-w-48 flex-col gap-y-2.5 rounded-r-lg border border-l-0 bg-light-background p-3.5">
        <div class="space-y-1.5">
          <h1 class="pb-0.5 text-lg text-light-green">Room</h1>
          <div class="flex items-center justify-between">
            <label for="game">game</label>
            <Select
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
        <div class="h-[1px] w-full bg-dark-green/30"></div>
        <div class="space-y-1.5">
          <h1 class="pb-0.5 text-lg text-light-green">Word Bomb</h1>
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
              <optgroup label="min 500wpp (words per prompt)">
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
          class="absolute -right-2.5 top-1/2 -translate-y-1/2 translate-x-full border-none bg-transparent p-0 text-light-green"
          onClick={() => setVisible((visible) => !visible)}
        >
          <SettingsIcon />
        </Button>
      </div>
    </div>
  );
}
