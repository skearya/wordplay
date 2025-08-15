import type { RoomSettings } from '@bindings/RoomSettings';
import type { ServerClient } from '@bindings/ServerClient';
import type { ServerState } from '@bindings/ServerState';

export type Context<State, ClientMessage> = {
	uuid: string;
	clients: { [uuid: string]: ServerClient | undefined };
	settings: RoomSettings;
	initialState: State;
	setRootState: (state: ServerState) => void;
	sendMsg: (message: ClientMessage) => void;
};
