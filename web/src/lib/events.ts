import type { ServerMessage } from '@bindings/ServerMessage';

export const serverMessageEmitter = eventEmitter<ServerMessage>();

function eventEmitter<Message>() {
	const subscriptions: Set<(data: Message) => void> = new Set();
	const unhandled: Message[] = [];

	return {
		on: (handler: (data: Message) => void) => {
			subscriptions.add(handler);

			return () => {
				subscriptions.delete(handler);
			};
		},

		emit: (message: Message) => {
			if (subscriptions.size === 0) {
				unhandled.push(message);
				return;
			}

			for (const handlers of subscriptions) {
				handlers(message);
			}
		}
	};
}
