import { Accessor, createEffect, For, on, onCleanup, onMount } from "solid-js";
import { Input } from "~/lib/components/ui/Input";
import { ChatMessage, ChatMessageType, Room, SendFn } from "~/lib/types/game";
import { generateGradient } from "../avatar";
import { getUsername } from "../utils";

export function Chat({
  sendMsg,
  room,
  messages,
}: {
  sendMsg: SendFn;
  room: Accessor<Room>;
  messages: Accessor<Array<ChatMessage>>;
}) {
  let chatElement!: HTMLDivElement;
  let chatContentElement!: HTMLUListElement;
  let chatInputElement!: HTMLInputElement;

  let animations: Array<Animation> = [];
  let fadeOutTimeout: ReturnType<typeof setTimeout> | undefined;

  function startFadeOut(duration = 3500) {
    clearTimeout(fadeOutTimeout);

    if (chatContentElement.matches(":hover") || document.activeElement == chatInputElement) {
      return;
    }

    const options = {
      easing: "ease-out",
      duration,
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
        reappear();
        chatContentElement.scrollTop = chatContentElement.scrollHeight;
        setTimeout(startFadeOut, 1500);
      },
    ),
  );

  onMount(() => {
    startFadeOut(5000);
  });

  const controller = new AbortController();

  document.addEventListener(
    "keydown",
    (event) => {
      if (document.activeElement?.tagName === "INPUT") return;

      if (event.key === "t") {
        chatInputElement.focus();
        event.preventDefault();
      }
    },
    { signal: controller.signal },
  );

  onCleanup(() => controller.abort());

  return (
    <div
      ref={chatElement}
      class="fixed bottom-0 left-0 z-10 flex w-96 flex-col gap-y-2 rounded-tr-lg border-r border-t bg-transparent p-2"
      onMouseEnter={reappear}
      onMouseLeave={() => startFadeOut()}
    >
      <ul
        ref={chatContentElement}
        class="list-item h-48 overflow-y-auto overflow-x-hidden text-wrap"
      >
        <li class="text-green">server: welcome to wordplay beta</li>
        <li class="text-green">
          server: leave issues/feedback on{" "}
          <a href="https://github.com/skearya/wordplay" target="_blank" class="text-gray-200">
            github
          </a>
        </li>
        <li class="text-green">
          server: use <kbd class="small-key">t</kbd> to open chat
        </li>
        <li class="text-green">
          server: and <kbd class="small-key">esc</kbd> to focus game input
        </li>
        <For each={messages()}>
          {(message) => {
            switch (message.type) {
              case ChatMessageType.Client:
                const username = getUsername(room(), message.uuid)!;
                const { fromColor, toColor } = generateGradient(username);

                return (
                  <li>
                    <span
                      style={{
                        "background-image": `linear-gradient(45deg, ${fromColor}, ${toColor})`,
                      }}
                      class="bg-clip-text text-transparent"
                    >
                      {username}:
                    </span>{" "}
                    {message.content}
                  </li>
                );
              case ChatMessageType.Info:
                return <li class="text-green">{`server: ${message.content}`}</li>;
              case ChatMessageType.Error:
                return <li class="text-red-400">{`error: ${message.content}`}</li>;
            }
          }}
        </For>
      </ul>
      <Input
        ref={chatInputElement}
        size="sm"
        class="bg-transparent"
        maxLength={250}
        placeholder="send a message..."
        onEnter={(input) => {
          if (input.value.length !== 0) {
            sendMsg({ type: "ChatMessage", content: input.value });
            input.value = "";
          }
        }}
        onFocus={reappear}
        onBlur={() => startFadeOut()}
      />
    </div>
  );
}
