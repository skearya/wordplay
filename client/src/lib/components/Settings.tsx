import { Accessor, createSignal } from "solid-js";
import { Settings as SettingsIcon } from "../icons";
import { Room, SendFn } from "../types/game";
import { Button } from "./ui/Button";

export function Settings({ sendMsg, room }: { sendMsg: SendFn; room: Accessor<Room> }) {
  const [visible, setVisible] = createSignal(false);

  return (
    <div
      classList={{ "-translate-x-full": !visible() }}
      class="fixed left-0 top-1/2 z-50 -translate-y-1/2 transition-transform"
    >
      <div class="relative flex flex-col gap-y-2 rounded-r-lg border border-l-0 bg-light-background p-3.5">
        <div class="space-y-1">
          {(["WordBomb", "Anagrams"] as const).map((game) => (
            <div class="space-x-2.5">
              <input
                id={game}
                name={game}
                type="radio"
                checked={room().settings.game === game}
                disabled={room().owner !== room().uuid}
                onChange={(event) => {
                  if (event.target.checked) {
                    sendMsg({ type: "RoomSettings", ...room().settings, game });
                  }
                }}
              />
              <label
                for={game}
                classList={{ "text-green": room().settings.game === game }}
                class="transition-colors"
              >
                {game === "WordBomb" ? "Word Bomb" : "Anagrams"}
              </label>
            </div>
          ))}
        </div>
        <div class="flex items-center justify-between">
          <label
            for="visibility"
            classList={{ "text-green": room().settings.public }}
            class="transition-colors"
          >
            Public
          </label>
          <input
            type="checkbox"
            name="visibility"
            id="visibility"
            checked={room().settings.public}
            onChange={(event) => {
              sendMsg({ type: "RoomSettings", ...room().settings, public: event.target.checked });
            }}
          />
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
