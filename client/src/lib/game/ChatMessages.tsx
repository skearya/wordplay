import { type Component, For, useContext } from 'solid-js';
import { Context } from '../context';
import { ClientMessage } from '../types/messages';

const ChatMessages: Component<{ sendMessage: (message: ClientMessage) => void }> = (props) => {
	const context = useContext(Context);
	if (!context) throw new Error('Not called inside context provider?');
	const { connection } = context[0];

	return (
		<section>
			<input
				class="border"
				type="text"
				maxlength="250"
				onKeyDown={(event) => {
					if (event.key === 'Enter') {
						props.sendMessage({
							type: 'chatMessage',
							content: event.currentTarget.value
						});
					}
				}}
			/>
			<ul class="list-item">
				<For each={connection.chatMessages}>{(message, _) => <li>{message}</li>}</For>
			</ul>
		</section>
	);
};

export { ChatMessages };
