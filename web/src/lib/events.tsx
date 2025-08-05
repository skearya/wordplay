/* eslint-disable @typescript-eslint/no-explicit-any */
import type { ServerAnagrams } from "@bindings/ServerAnagrams";
import type { ServerGame } from "@bindings/ServerGame";
import type { ServerGeneral } from "@bindings/ServerGeneral";
import type { ServerLobby } from "@bindings/ServerLobby";
import type { ServerWordBomb } from "@bindings/ServerWordBomb";
import { useEffect } from "react";

export type Subscriber<Events extends { kind: string }> = ReturnType<
	typeof eventEmitter<Events>
>["on"];

export const generalEmitter = eventEmitter<ServerGeneral>();
export const lobbyEmitter = eventEmitter<ServerLobby>();
export const gameEmitter = eventEmitter<ServerGame>();

export const wordBombEmitter = eventEmitter<ServerWordBomb>();
export const anagramsEmitter = eventEmitter<ServerAnagrams>();

function eventEmitter<Events extends { kind: string }>() {
	const subscriptions: Map<string, Set<(message: any) => void>> = new Map();
	const unhandled: Map<string, any[]> = new Map();

	return {
		on<Kind extends Events["kind"]>(
			kind: Kind,
			fn: (message: Extract<Events, { kind: Kind }>) => void,
		) {
			const unhandledKind = unhandled.get(kind);

			if (unhandledKind) {
				for (const message of unhandledKind) {
					fn(message);
				}

				unhandled.delete(kind);
			}

			let set = subscriptions.get(kind);

			if (set) {
				set.add(fn);
			} else {
				set = new Set([fn]);

				subscriptions.set(kind, set);
			}

			return () => {
				set.delete(fn);
			};
		},

		emit(message: Events) {
			const set = subscriptions.get(message.kind);

			if (set) {
				for (const fn of set) {
					fn(message);
				}
			} else {
				const unhandledKind = unhandled.get(message.kind);

				if (unhandledKind) {
					unhandledKind.push(message);
				} else {
					unhandled.set(message.kind, [message]);
				}
			}
		},
	};
}

export function useEmitter<
	Events extends { kind: string },
	Kind extends Events["kind"],
>(
	emitter: ReturnType<typeof eventEmitter<Events>>,
	kind: Kind,
	fn: (message: Extract<Events, { kind: Kind }>) => void,
) {
	useEffect(() => emitter.on(kind, fn), [emitter, fn, kind]);
}
