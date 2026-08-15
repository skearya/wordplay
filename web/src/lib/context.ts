import type { ClientMessage } from '@bindings/ClientMessage';
import type { ServerMessage } from '@bindings/ServerMessage';
import type { Variant } from '$lib/utils';

/// Initial state the server sends on connection.
export type Context = Variant<ServerMessage, 'info'>['data'];

export type Props<State> = {
	ctx: Context;
	state: State;
	sendMsg: (message: ClientMessage) => void;
};
