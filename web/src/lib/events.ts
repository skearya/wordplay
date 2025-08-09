import type { Variant } from '$lib/utils';
import type { ServerAnagrams } from '@bindings/ServerAnagrams';
import type { ServerGame } from '@bindings/ServerGame';
import type { ServerGeneral } from '@bindings/ServerGeneral';
import type { ServerLobby } from '@bindings/ServerLobby';
import type { ServerMessage } from '@bindings/ServerMessage';
import type { ServerWordBomb } from '@bindings/ServerWordBomb';

export const rootEmitter = eventEmitter<ServerMessage>();

export const generalEmitter = eventEmitter<ServerGeneral>();
export const lobbyEmitter = eventEmitter<ServerLobby>();
export const gameEmitter = eventEmitter<ServerGame>();

export const wordBombEmitter = eventEmitter<ServerWordBomb>();
export const anagramsEmitter = eventEmitter<ServerAnagrams>();

function eventEmitter<Events extends { kind: string }>() {
	const subscriptions: Set<Record<string, (message: any) => void>> = new Set();
	const unhandled: Map<string, any[]> = new Map();

	return {
		on(handlers: { [Kind in Events['kind']]: (message: Variant<Events, Kind>) => void }) {
			for (const kind in handlers) {
				const unhandledKind = unhandled.get(kind);

				if (unhandledKind) {
					for (const message of unhandledKind) {
						handlers[kind as Events['kind']](message);
					}

					unhandled.delete(kind);
				}
			}

			subscriptions.add(handlers);

			return () => subscriptions.delete(handlers);
		},

		emit(message: Events) {
			if (subscriptions.size === 0) {
				const unhandledKind = unhandled.get(message.kind);

				if (unhandledKind) {
					unhandledKind.push(message);
				} else {
					unhandled.set(message.kind, [message]);
				}

				return;
			}

			for (const handlers of subscriptions) {
				handlers[message.kind](message);
			}
		}
	};
}
