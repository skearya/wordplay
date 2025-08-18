import type { Variant } from './utils';
import type { ServerMessage } from '@bindings/ServerMessage';

export const serverMessageEmitter = eventEmitter<{
	[Kind in ServerMessage['kind']]: Variant<ServerMessage, Kind>;
}>();

function eventEmitter<Events extends { [kind: string]: any }>() {
	const subscriptions: Map<keyof Events, Set<(data: any) => void>> = new Map();

	return {
		on: <Kind extends keyof Events>(kind: Kind, handler: (data: Events[Kind]) => void) => {
			const kindSubscriptions = subscriptions.get(kind);

			if (kindSubscriptions) {
				kindSubscriptions.add(handler);
			} else {
				subscriptions.set(kind, new Set([handler]));
			}

			return () => void subscriptions.get(kind)?.delete(handler);
		},

		emit: <Kind extends keyof Events>(message: { kind: Kind } & Events[Kind]) => {
			const kindSubscriptions = subscriptions.get(message.kind);

			if (kindSubscriptions) {
				for (const handlers of kindSubscriptions) {
					handlers(message);
				}
			}
		}
	};
}
