import type { ClientMessage } from '@bindings/ClientMessage';
import type { RoomSettings } from '@bindings/RoomSettings';
import type { ServerClient } from '@bindings/ServerClient';
import type { ServerState } from '@bindings/ServerState';

export type Context<State = ServerState, Client = ClientState, Message = ClientMessage> = {
	uuid: string;
	clients: { [uuid: string]: ServerClient | undefined };
	settings: RoomSettings;
	state: State;
	client: Client;
	send: (message: Message) => void;
};

export type ClientState = {
	ping: number;
	chatMessages: { author: string; content: string }[];
	state: ({ kind: 'lobby' } & ClientLobbyState) | ({ kind: 'game' } & ClientGameState);
};

export type ClientLobbyState = {
	practicePrompts: string[];
};

export type ClientGameState = {};

export const defaultClientContext = (state: ServerState): ClientState => ({
	ping: 0,
	chatMessages: [],
	state: state.kind === 'lobby' ? { kind: 'lobby', practicePrompts: [] } : { kind: 'game' }
});
