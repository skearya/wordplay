import { Accessor, Show } from "solid-js";
import { Link, QuestionMark } from "~/lib/icons";
import { Room } from "~/lib/types/game";

export function GameNav({ room }: { room: Accessor<Room> }) {
  return (
    <nav class="absolute top-0 flex w-full items-center justify-between px-6 py-5">
      <h1 class="text-2xl">wordplay</h1>
      <div class="flex items-center gap-x-5">
        <div
          style="box-shadow: 0px 0px 15.5px 1px #26D16C"
          class="h-[13px] w-[13px] rounded-full bg-[#26D16C]"
        />
        <h1 class="text-[#B1C1AE]">44ms</h1>
        <div class="flex -space-x-2">
          {room()
            .clients.slice(0, 3)
            .map((client) => (
              <img
                src={`https://avatar.vercel.sh/${client.username}`}
                alt={client.username}
                title={client.username}
                height={38}
                width={38}
                class="rounded-full border-[3px] border-black"
              />
            ))}
          <Show when={room().clients.length > 3}>
            <div class="flex h-[38px] w-[38px] items-center justify-center rounded-full bg-black">
              <h1>+{room().clients.length - 3}</h1>
            </div>
          </Show>
        </div>
        <Link />
        <QuestionMark />
      </div>
    </nav>
  );
}
