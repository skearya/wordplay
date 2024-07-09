import { A } from "@solidjs/router";
import { Match, Switch, createResource } from "solid-js";

type Info = {
  clients_connected: number;
  public_rooms: Array<{
    name: string;
    players: Array<string>;
  }>;
};

export default function Home() {
  const [data] = createResource<Info>(async () => {
    const res = await fetch(`${import.meta.env.PUBLIC_SERVER}/api/info`);
    return await res.json();
  });

  return (
    <main class="mx-auto mt-16 flex h-[calc(100vh_-_4rem)] max-w-xl flex-col gap-y-4 rounded-t-2xl border bg-[#0C0D0A] p-5">
      <h1 class="text-3xl">wordplay</h1>
      <div class="h-[1px] w-full bg-[#475D50]/30"></div>
      <div class="flex gap-2 font-medium">
        <button class="w-full rounded-lg bg-[#475D50] p-4">Create Game</button>
        <button class="w-full rounded-lg bg-[#345C8A] p-4">Join Game</button>
      </div>
      <Switch>
        <Match when={data.loading}>
          <h1 class="text-center text-gray-400">loading...</h1>
        </Match>
        <Match when={data.error}>
          <h1 class="text-center text-red-400">something went wrong...</h1>
        </Match>
        <Match when={data()}>
          {(data) => (
            <>
              <div class="h-full space-y-3 overflow-y-scroll">
                {data().public_rooms.map((room) => (
                  <Room {...room} />
                ))}
                {data().public_rooms.length === 0 && (
                  <h1 class="text-center text-gray-400">there aren't any public rooms!</h1>
                )}
              </div>
              <div class="mt-auto text-center font-medium">
                <span class="text-[#62E297]">{data().clients_connected}</span> connected
              </div>
            </>
          )}
        </Match>
      </Switch>
    </main>
  );
}

function Room(props: { name: string; players: Array<string> }) {
  return (
    <A
      href={`/game/${props.name}`}
      class="relative flex items-center justify-between overflow-hidden rounded-lg border p-4 font-medium"
    >
      <h1 class="text-xl">{props.name}</h1>
      <div class="flex -space-x-2">
        {props.players.slice(0, 3).map((player) => (
          <img
            src={`https://avatar.vercel.sh/${player}`}
            alt={player}
            width="38"
            height="38"
            title={player}
            class="rounded-full border-[3px] border-black"
          />
        ))}
        {props.players.length > 3 && (
          <div class="flex h-[38px] w-[38px] items-center justify-center rounded-full bg-black">
            <h1>+{props.players.length - 3}</h1>
          </div>
        )}
      </div>
    </A>
  );
}
