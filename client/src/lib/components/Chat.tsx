import { Accessor, createEffect, For, on, onCleanup, onMount } from "solid-js";
import { ChatMessage, SendFn } from "~/lib/types/game";

export function Chat({
  sendMsg,
  messages,
}: {
  sendMsg: SendFn;
  messages: Accessor<Array<ChatMessage>>;
}) {
  let chatElement!: HTMLDivElement;
  let chatContentElement!: HTMLUListElement;
  let chatInputElement!: HTMLInputElement;

  let animations: Array<Animation> = [];
  let fadeOutTimeout: NodeJS.Timeout | undefined;

  function startFadeOut() {
    clearTimeout(fadeOutTimeout);

    const options = {
      easing: "ease-out",
      duration: 5000,
      fill: "forwards",
    } as const;

    animations.push(chatElement.animate({ borderColor: "transparent" }, options));
    animations.push(chatContentElement.animate({ opacity: "0%" }, options));
  }

  function reappear() {
    animations.forEach((animation) => animation.cancel());
    animations = [];
  }

  createEffect(
    on(
      () => messages().length,
      () => {
        chatContentElement.scrollTop = chatContentElement.scrollHeight;

        reappear();
        clearTimeout(fadeOutTimeout);
        fadeOutTimeout = setTimeout(startFadeOut, 3000);
      },
    ),
  );

  onMount(() => {
    startFadeOut();
  });

  const controller = new AbortController();

  document.addEventListener(
    "keydown",
    (event) => {
      if (document.activeElement?.tagName === "INPUT") return;

      if (event.key === "t") {
        event.preventDefault();
        chatInputElement.focus();
        reappear();
      }
    },
    { signal: controller.signal },
  );

  onCleanup(() => {
    controller.abort();
  });

  return (
    <div
      ref={chatElement}
      class="bg-primary-50/25 fixed bottom-0 left-0 z-50 flex w-96 flex-col rounded-tr-lg border-r border-t"
      onMouseEnter={reappear}
      onMouseLeave={startFadeOut}
    >
      <ul
        ref={chatContentElement}
        class="m-2 mb-0 list-item h-48 overflow-y-auto text-wrap break-all"
      >
        <For each={messages()}>
          {([content, isServer]) => (
            <li class={isServer ? "text-green-400" : undefined}>
              {isServer ? `server: ${content}` : content}
            </li>
          )}
        </For>
      </ul>
      <input
        ref={chatInputElement}
        class="m-2 h-10 rounded-lg border bg-transparent px-2.5 py-2 placeholder-white/50"
        type="text"
        maxlength="250"
        placeholder="send a message..."
        onKeyDown={(event) => {
          const input = event.target as HTMLInputElement;

          if (event.key === "Enter" && input.value !== "") {
            sendMsg({ type: "ChatMessage", content: input.value });
            input.value = "";
          }
        }}
      />
    </div>
  );
}
