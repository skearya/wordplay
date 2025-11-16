import type { ClientMessage } from '@bindings/ClientMessage';
import type { ServerMessage } from '@bindings/ServerMessage';
import type { Variant } from '$lib/utils';

/// Initial state the server sends on connection.
export type Context = Variant<ServerMessage, 'info'>['data'];

export type Props<InitialState> = {
	ctx: Context;
	initial: InitialState;
	sendMsg: (message: ClientMessage) => void;
};
