import { type Component, For, useContext, createEffect, on } from 'solid-js';
import { Context } from '../context';
import { ClientMessage } from '../types/messages';

const ChatMessages: Component<{ sender: (message: ClientMessage) => void }> = (props) => {
	const context = useContext(Context);
	if (!context) throw new Error('Not called inside context provider?');
	const { connection } = context[0];

	let messagesContainer!: HTMLUListElement;
	let chatInput!: HTMLInputElement;

	document.addEventListener('keydown', (event) => {
		if (document.activeElement?.tagName === 'INPUT') return;

		if (event.key === 't') {
			event.preventDefault();
			chatInput.focus();
		}
	});

	createEffect(
		on(
			() => connection.chatMessages.length,
			() => (messagesContainer.scrollTop = messagesContainer.scrollHeight)
		)
	);

	return (
		<section class="bg-primary-50/25 fixed bottom-0 left-0 flex w-96 flex-col rounded-tr-xl border">
			<ul
				ref={messagesContainer}
				class="m-2 mb-0 list-item h-48 overflow-y-auto text-wrap break-all"
			>
				<For each={connection.chatMessages}>{(message) => <li>{message}</li>}</For>
			</ul>
			<input
				ref={chatInput}
				class="m-2 h-10 rounded-lg border bg-transparent placeholder-white/50 placeholder:text-center"
				type="text"
				placeholder="send a message..."
				maxlength="250"
				onKeyDown={(event) => {
					if (event.key === 'Enter') {
						props.sender({
							type: 'chatMessage',
							content: event.currentTarget.value
						});

						event.currentTarget.value = '';
					}
				}}
			/>
		</section>
	);
};

export { ChatMessages };
