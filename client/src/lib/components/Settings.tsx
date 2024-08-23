import { Accessor, createSignal, Match, Show, Switch } from "solid-js";
import { Room, SendFn } from "../types/game";

export function Settings({ sendMsg, room }: { sendMsg: SendFn; room: Accessor<Room> }) {
  const [visible, setVisible] = createSignal(false);
  const [selected, setSelected] = createSignal<"room" | "game">("room");

  return (
    <Show when={visible()}>
      <div class="fixed left-0 top-0 z-50 flex h-full w-full items-center justify-start bg-transparent/60">
        <div class="flex rounded-r-lg border border-l-0 bg-light-background p-3.5">
          <div class="flex flex-col text-green">
            <button onClick={() => setSelected("room")}>room</button>
            <button onClick={() => setSelected("game")}>game</button>
          </div>
          <div class="w-[1px] self-stretch bg-dark-green/30"></div>
          <Switch>
            <Match when={selected() === "room"}>
              {(["WordBomb", "Anagrams"] as const).map((game) => (
                <div class="space-x-2">
                  <input
                    id={game}
                    type="radio"
                    checked={room().settings.game === game}
                    disabled={room().owner !== room().uuid}
                    onChange={(event) => {
                      if (event.target.checked) {
                        sendMsg({
                          type: "RoomSettings",
                          ...room().settings,
                          game,
                        });
                      }
                    }}
                  />
                  <label for={game}>{game}</label>
                </div>
              ))}
            </Match>
            <Match when={selected() === "game"}>
              <h1>e</h1>
            </Match>
          </Switch>
        </div>
      </div>
    </Show>
  );
}
