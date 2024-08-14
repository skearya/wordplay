import { Accessor, For, Setter } from "solid-js";
import { useEvent } from "~/lib/events";
import { Room, SendFn } from "~/lib/types/game";
import { getUsername } from "~/lib/utils";

export function Chat({
  sendMsg,
  room,
  messages,
  setMessages,
}: {
  sendMsg: SendFn;
  room: Accessor<Room>;
  messages: Accessor<Array<string>>;
  setMessages: Setter<Array<string>>;
}) {
  useEvent("ChatMessage", (data) => {
    const message = `${getUsername(room(), data.author)}: ${data.content}`;
    setMessages((messages) => [...messages, message]);
  });

  return (
    <div class="bg-primary-50/25 fixed bottom-0 left-0 z-50 flex w-96 flex-col rounded-tr-lg border-r border-t">
      <ul class="m-2 mb-0 list-item h-48 overflow-y-auto text-wrap break-all">
        <For each={messages()}>{(message) => <li>{message}</li>}</For>
      </ul>
      <input
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
