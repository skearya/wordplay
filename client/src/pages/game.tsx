import { Link, QuestionMark } from "~/lib/icons";

export default function Game() {
  return (
    <>
      <GameNav />
      <main class="flex h-[calc(100vh_-_82px)] items-center justify-start">
        <div class="rounded-xl rounded-l-none border border-l-0 p-4">
          <h1>Ready Players</h1>
        </div>
      </main>
    </>
  );
}

function GameNav() {
  return (
    <nav class="flex justify-between px-6 py-5">
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
