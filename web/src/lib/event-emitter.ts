import { getOrSet } from './utils';

export class EventEmitter<Message extends { kind: Kinds }, Kinds extends string = Message['kind']> {
	subscriptions: Map<Kinds, ((data: Message) => void)[]> = new Map();
	unhandledMessages: Map<Kinds, Message[]> = new Map();

	on<K extends Kinds>(kind: K, handler: (data: Extract<Message, { kind: K }>) => void) {
		const handlers = getOrSet({ map: this.subscriptions, key: kind, val: [] });

		if (handlers.length === 0) {
			const unhandled = this.unhandledMessages.get(kind);

			if (unhandled) {
				for (const message of unhandled) {
					handler(message as Extract<Message, { kind: K }>);
				}
			}
		}

		handlers.push(handler as (data: Message) => void);

		return () => {
			const index = handlers.indexOf(handler as (data: Message) => void);

			if (index !== -1) {
				handlers.splice(index, 1);
			}
		};
	}

	handle(handlers: { [K in Kinds]?: (data: Extract<Message, { kind: K }>) => void }) {
		const distructors: (() => void)[] = [];

		for (const key in handlers) {
			if (!handlers[key]) continue;

			const distructor = this.on(key, handlers[key]);
			distructors.push(distructor);
		}

		return () => distructors.forEach((f) => f());
	}

	emit(message: Message) {
		const handlers = getOrSet({ map: this.subscriptions, key: message.kind, val: [] });

		if (handlers.length === 0) {
			const unhandled = getOrSet({ map: this.unhandledMessages, key: message.kind, val: [] });
			unhandled.push(message);
		} else {
			for (const handler of handlers) {
				handler(message);
			}
		}
	}
}
