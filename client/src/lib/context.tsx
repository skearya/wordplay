import { ConnectionData, GameData, LobbyData } from './types/stores';
import { ParentComponent, createContext, createSignal } from 'solid-js';
import { createStore } from 'solid-js/store';

function makeContext(room = '') {
	const [state, setState] = createSignal<'connecting' | 'error' | 'lobby' | 'game'>('connecting');

	const [connection, setConnection] = createStore<ConnectionData>({
		room,
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
		guessError: ['', ''],
		usedLetters: new Set(),
		input: ''
	});

	return [
		{ state, connection, lobby, game },
		{ setState, setConnection, setLobby, setGame }
	] as const;
}

const Context = createContext<ReturnType<typeof makeContext>>();

const ContextProvider: ParentComponent<{ room: string }> = (props) => {
	return <Context.Provider value={makeContext(props.room)}>{props.children}</Context.Provider>;
};

export { Context, ContextProvider };
