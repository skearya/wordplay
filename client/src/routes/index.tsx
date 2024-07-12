import { A, useNavigate } from "@solidjs/router";
import { Match, Show, Switch, createResource, createSignal } from "solid-js";
import { cloneElement } from "~/lib/utils";

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
    <main class="mx-auto mt-16 flex h-[calc(100vh_-_4rem)] min-h-96 max-w-xl flex-col gap-y-4 rounded-t-2xl border bg-[#0C0D0A] p-5">
      <h1 class="text-3xl">wordplay</h1>
      <div class="min-h-[1px] w-full bg-[#475D50]/50"></div>
      <CreateOrJoinRoom />
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

function CreateOrJoinRoom() {
  let creatingElement!: HTMLButtonElement;
  let joiningElement!: HTMLButtonElement;
  let roomInputElement!: HTMLInputElement;
  let roomErrorElement!: HTMLHeadingElement;

  let animationInProgress = false;
  const [joining, setJoining] = createSignal(false);

  const navigate = useNavigate();

  const animationOptions = {
    easing: "ease",
    duration: 500,
  };

  const animation = (f: () => Promise<void>) => {
    if (animationInProgress) return;
    animationInProgress = true;
    f().then(() => (animationInProgress = false));
  };

  async function clickAnimation(clicked: "creating" | "joining") {
    animation(async () => {
      const expanding = clicked === "creating" ? creatingElement : joiningElement;
      const shrinking = clicked === "creating" ? joiningElement : creatingElement;

      const prevWidth = expanding.offsetWidth;

      const shrinkingClone = cloneElement(shrinking);
      shrinking.style.display = "none";
      document.body.appendChild(shrinkingClone);

      shrinkingClone
        .animate(
          {
            opacity: 0,
            transform: `translateX(${clicked === "creating" ? 100 : -100}%)`,
            filter: "blur(25px)",
          },
          animationOptions,
        )
        .finished.then(() => shrinkingClone.remove());

      const padding = document.createElement("div");
      padding.style.width = prevWidth + "px";
      padding.style.flex = "none";

      if (clicked === "creating") {
        expanding.parentElement!.appendChild(padding);
      } else {
        joiningElement.parentElement!.insertBefore(padding, joiningElement);
      }

      padding.animate({ width: "0px" }, animationOptions).finished.then(() => padding.remove());

      const gapAnimation = expanding.parentElement!.animate({ gap: "0px" }, animationOptions);
      await gapAnimation.finished;
      gapAnimation.commitStyles();

      await expanding.animate(
        {
          color: "transparent",
          backgroundColor: "transparent",
          borderColor: "rgb(255 255 255 / 0.1)",
        },
        animationOptions,
      ).finished;

      expanding.style.display = "none";
      creatingElement.parentElement!.style.gap = "0.5rem";

      setJoining(true);
      await roomInputElement.animate({ opacity: [0, 100] }, animationOptions).finished;
      roomInputElement.focus();
    });
  }

  async function exitInput() {
    animation(async () => {
      await creatingElement.parentElement!.animate(
        { opacity: [100, 0], filter: "blur(5px)" },
        animationOptions,
      ).finished;
      setJoining(false);

      creatingElement.style.display = "block";
      joiningElement.style.display = "block";
      await creatingElement.parentElement!.animate({ opacity: [0, 100] }, animationOptions)
        .finished;
    });
  }

  async function checkRoomAvailability() {
    if (roomInputElement.value.length <= 1) {
      return;
    }

    roomInputElement.disabled = true;

    const res = await fetch(
      `${import.meta.env.PUBLIC_SERVER}/api/room-available/${roomInputElement.value}`,
    );
    const available = (await res.text()) === "true";

    if (available) {
      navigate(`/room/${roomInputElement.value}`);
      return;
    }

    roomErrorElement.animate(
      { opacity: [100, 0] },
      {
        easing: "ease",
        duration: 3000,
      },
    );

    roomInputElement.disabled = false;
  }

  return (
    <div class="flex h-14 gap-2">
      <button
        ref={creatingElement}
        class="w-full overflow-hidden whitespace-nowrap rounded-lg border border-transparent bg-[#475D50] p-4 font-medium transition-all"
        onClick={() => clickAnimation("creating")}
      >
        Create Game
      </button>
      <button
        ref={joiningElement}
        class="w-full overflow-hidden whitespace-nowrap rounded-lg border border-transparent bg-[#345C8A] p-4 font-medium transition-all"
        onClick={() => clickAnimation("joining")}
      >
        Join Game
      </button>
      <Show when={joining()}>
        <div class="relative w-full rounded-lg border">
          <h1
            ref={roomErrorElement}
            class="absolute right-4 top-1/2 -translate-y-1/2 text-red-400 opacity-0"
          >
            name already in use
          </h1>
          <input
            ref={roomInputElement}
            type="text"
            placeholder="room name"
            minlength={1}
            maxlength={6}
            class="w-full rounded-lg bg-transparent p-4 transition-all disabled:opacity-50"
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                checkRoomAvailability();
              } else if (event.key === "Escape") {
                exitInput();
              }
            }}
          />
        </div>
      </Show>
    </div>
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
