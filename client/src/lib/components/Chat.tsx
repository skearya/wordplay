import { Accessor, createEffect, For, on, onCleanup, onMount } from "solid-js";
import { Input } from "~/lib/components/ui/Input";
import { ChatMessage, ChatMessageType, SendFn } from "~/lib/types/game";

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
  let fadeOutTimeout: ReturnType<typeof setTimeout> | undefined;

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
        fadeOutTimeout = setTimeout(startFadeOut, 1500);
      },
    ),
  );

  onMount(() => {
    startFadeOut();
  });

  function onDocumentKeydown(event: KeyboardEvent) {
    if (document.activeElement?.tagName === "INPUT") return;

    if (event.key === "t") {
      chatInputElement.focus();
      event.preventDefault();
    }
  }

  document.addEventListener("keydown", onDocumentKeydown);

  onCleanup(() => document.removeEventListener("keydown", onDocumentKeydown));

  return (
    <div
      ref={chatElement}
      class="fixed bottom-0 left-0 z-50 flex w-96 flex-col gap-y-2 rounded-tr-lg border-r border-t bg-transparent p-2"
      onMouseEnter={reappear}
      onMouseLeave={startFadeOut}
    >
      <ul ref={chatContentElement} class="list-item h-48 overflow-y-auto text-wrap break-all">
        <li class="text-green">server: welcome to wordplay beta</li>
        <li class="text-green">
          server: leave issues/feedback on{" "}
          <a href="https://github.com/skearya/wordplay" target="_blank" class="text-gray-200">
            github
          </a>
        </li>
        <li class="text-green">
          server: use <kbd>t</kbd> to open chat
        </li>
        <li class="text-green">
          server: and <kbd>esc</kbd> to focus game input
        </li>
        <For each={messages()}>
          {([content, type]) => {
            switch (type) {
              case ChatMessageType.Client:
                return <li>{content}</li>;
              case ChatMessageType.Server:
                return <li class="text-green">{`server: ${content}`}</li>;
              case ChatMessageType.Error:
                return <li class="text-red-400">{`error: ${content}`}</li>;
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
          sendMsg({ type: "ChatMessage", content: input.value });
          input.value = "";
        }}
        onFocus={reappear}
        onBlur={startFadeOut}
      />
    </div>
  );
}
