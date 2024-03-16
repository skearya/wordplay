import { ConnectionData, GameData, LobbyData } from './types/stores';
import { ParentComponent, createContext, createSignal } from 'solid-js';
import { createStore } from 'solid-js/store';

function makeContext() {
	const [state, setState] = createSignal<'connecting' | 'error' | 'lobby' | 'game'>('connecting');

	const [connection, setConnection] = createStore<ConnectionData>({
		room: prompt('room', 'one')!,
		username: prompt('username', 'skeary')!,
		uuid: '',
		chatMessages: []
	});

	const [lobby, setLobby] = createStore<LobbyData>({
		readyPlayers: [],
		previousWinner: null,
		startingCountdown: null
	});

	const [game, setGame] = createStore<GameData>({
		players: [],
		currentTurn: '',
		prompt: '',
		usedLetters: new Set(),
		input: ''
	});

	return [
		{ state, connection, lobby, game },
		{ setState, setConnection, setLobby, setGame }
	] as const;
}

export const Context = createContext<ReturnType<typeof makeContext>>();

export const ContextProvider: ParentComponent = (props) => {
	return <Context.Provider value={makeContext()}>{props.children}</Context.Provider>;
};
