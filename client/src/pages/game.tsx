import { Link, QuestionMark } from "~/lib/icons";

function GameNav() {
  return (
    <nav class="flex items-center justify-between px-6 py-5">
      <h1 class="text-2xl">wordplay</h1>
      <div class="flex items-center gap-x-5">
        <div
          style="box-shadow: 0px 0px 15.5px 1px #26D16C"
          class="h-[13px] w-[13px] rounded-full bg-[#26D16C]"
        />
        <h1 class="text-[#B1C1AE]">44ms</h1>
        <div class="flex -space-x-2">
          {Array.from({ length: 3 }).map((_, i) => (
            <img
              src={`https://avatar.vercel.sh/${i}`}
              alt={`${i}`}
              height={38}
              width={38}
              class="rounded-full border-[3px] border-black"
            />
          ))}
        </div>
        <Link />
        <QuestionMark />
      </div>
    </nav>
  );
}

function Tabs() {
  return (
    <div class="flex overflow-hidden rounded-lg border text-center">
      <button class="flex-1 rounded-lg bg-[#475D50] py-2">Info</button>
      <button class="flex-1 rounded-lg py-2">Settings</button>
    </div>
  );
}

function Winner() {
  return (
    <div
      style="background-image: linear-gradient(135deg, #171717 6.52%, transparent 6.52%, transparent 50%, #171717 50%, #171717 56.52%, transparent 56.52%, transparent 100%); background-size: 23.00px 23.00px;"
      class="flex items-center justify-between rounded-lg border p-2.5"
    >
      <h1 class="text-lg font-medium">winner!</h1>
      <div class="flex items-center gap-x-4">
        <h1 class="text-lg">Player 1</h1>
        <img
          src="https://avatar.vercel.sh/skeary"
          alt="profile picture"
          width={50}
          height={50}
          class="rounded-full"
        />
      </div>
    </div>
  );
}

function Leaderboard({
  title,
  items,
}: {
  title: string;
  items: Array<{
    pfp: string;
    name: string;
    content: string;
  }>;
}) {
  return (
    <div>
      <h1 class="mb-1.5">{title}</h1>
      <div class="grid max-h-[100px] w-36 grid-cols-[min-content_18px_auto_min-content] items-center gap-x-1 gap-y-1.5 overflow-y-scroll">
        {items.map(({ pfp, name, content }, i) => (
          <>
            <h1 class="text-sm">{i + 1}.</h1>
            <img src={pfp} alt="profile picture" width={18} height={18} class="rounded-full" />
            <h1 class="max-w-full overflow-hidden text-ellipsis whitespace-nowrap text-sm">
              {name}
            </h1>
            <h1 class="justify-self-end text-sm text-[#8BA698]">{content}</h1>
          </>
        ))}
      </div>
    </div>
  );
}

function Stats() {
  return (
    <div class="mt-auto flex items-center justify-around text-center [&>h1>span]:text-[#62E297]">
      <h1 class="flex-1 border-r border-[#62E297]">
        <span>21</span> mins elapsed
      </h1>
      <h1 class="flex-1 border-r border-[#62E297]">
        <span>209</span> words used
      </h1>
      <h1 class="flex-1">
        <span>2032</span> letters typed
      </h1>
    </div>
  );
}

function ReadyPlayers() {
  return (
    <>
      <div class="flex items-baseline justify-between">
        <h1 class="text-xl">Ready Players</h1>
        <h1 class="text-lg text-[#26D16C]">96 slots left</h1>
      </div>
      <div class="grid grid-cols-2 gap-2.5 overflow-y-scroll">
        {Array.from({ length: 5 }).map(() => (
          <div class="flex items-center justify-between gap-x-4 rounded-lg border bg-[#475D50]/30 p-3">
            <img
              src={`https://avatar.vercel.sh/skeary`}
              alt={`profile picture`}
              height={55}
              width={55}
              class="rounded-full"
            />
            <h1 class="overflow-hidden text-ellipsis whitespace-nowrap text-lg">skeary</h1>
          </div>
        ))}
      </div>
    </>
  );
}

function JoinButtons() {
  return (
    <div class="mt-auto flex gap-x-2.5">
      <button class="flex-1 rounded-lg border bg-[#475D50] py-4 font-medium">Ready</button>
      <button class="flex-1 rounded-lg border bg-[#345C8A] py-4 font-medium">Start Early</button>
    </div>
  );
}

export default function Game() {
  const items = [
    {
      pfp: "https://avatar.vercel.sh/skeary",
      name: "skeary",
      content: "4.3s",
    },
    {
      pfp: "https://avatar.vercel.sh/skeary2",
      name: "skeary2",
      content: "4s",
    },
    {
      pfp: "https://avatar.vercel.sh/skeary3",
      name: "skeary3",
      content: ".3s",
    },
    {
      pfp: "https://avatar.vercel.sh/skeary4",
      name: "skeary4",
      content: "0.1s",
    },
    {
      pfp: "https://avatar.vercel.sh/skeary3",
      name: "skeary3",
      content: ".3s",
    },
    {
      pfp: "https://avatar.vercel.sh/skeary4",
      name: "skeary4",
      content: "0.1s",
    },
  ];

  return (
    <>
      <GameNav />
      <main class="flex h-[calc(100vh_-_82px)] items-center justify-start overflow-hidden">
        <div class="flex h-[480px] gap-x-4 rounded-xl rounded-l-none border border-l-0 bg-[#0B0D0A] p-3.5">
          <div class="flex flex-col gap-y-3.5">
            <Winner />
            <div class="flex flex-wrap gap-x-10 gap-y-2.5">
              {["fastest guess", "longest word", "wpm", "word length"].map((title) => (
                <Leaderboard title={title} items={items} />
              ))}
            </div>
            <Stats />
          </div>
          <div class="w-[1px] scale-y-90 self-stretch bg-[#475D50]/30"></div>
          <div class="flex w-[475px] flex-col gap-y-2">
            <ReadyPlayers />
            <JoinButtons />
          </div>
        </div>
      </main>
    </>
  );
}
