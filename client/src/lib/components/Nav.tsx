import { Accessor, createSignal, onCleanup, Show } from "solid-js";
import { useEvent } from "~/lib/events";
import { Link } from "~/lib/icons";
import { Room, SendFn } from "~/lib/types/game";
import { Avatar } from "./ui/Avatar";

export function GameNav({ sendMsg, room }: { sendMsg: SendFn; room: Accessor<Room> }) {
  const [ping, setPing] = createSignal(0);

  const sendPing = () => sendMsg({ type: "Ping", timestamp: Date.now() });
  sendPing();

  const pingInterval = setInterval(sendPing, 10000);

  useEvent("Pong", (data) => setPing(Date.now() - data.timestamp));

  onCleanup(() => clearInterval(pingInterval));

  return (
    <nav class="flex w-full items-center justify-between px-6 py-5 text-light-green">
      <h1 class="text-xl text-foreground">wordplay</h1>
      <div class="flex items-center gap-x-5">
        <div
          style="box-shadow: 0px 0px 15.5px 1px #26D16C"
          class="h-[13px] w-[13px] rounded-full bg-green"
        />
        <h1>{ping()}ms</h1>
        <div class="flex -space-x-2">
          {room()
            .clients.slice(0, 3)
            .map((client) => (
              <Avatar username={client.username} size={38} class="border-[3px] border-black" />
            ))}
          <Show when={room().clients.length > 3}>
            <div class="flex h-[38px] w-[38px] items-center justify-center rounded-full bg-black">
              <h1>+{room().clients.length - 3}</h1>
            </div>
          </Show>
        </div>
        <button
          class="transition-opacity active:opacity-50"
          onClick={() => navigator.clipboard.writeText(window.location.href)}
        >
          <Link />
        </button>
        {/* <button class="transition-opacity active:opacity-50">
          <QuestionMark />
        </button> */}
      </div>
    </nav>
  );
}
