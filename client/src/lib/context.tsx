import type { ConnectionData, State, LobbyData, WordBombData, AnagramsData } from './types/stores';
import { ParentComponent, createContext, createSignal } from 'solid-js';
import { createStore } from 'solid-js/store';

function makeContext(room = '') {
	const [state, setState] = createSignal<State>('connecting');

	const [connection, setConnection] = createStore<ConnectionData>({
		room,
		uuid: '',
		settings: { game: 'wordBomb', public: false },
		username: prompt('username', 'username')!,
		roomOwner: '',
		clients: [],
		chatMessages: []
	});

	const [lobby, setLobby] = createStore<LobbyData>({
		readyPlayers: [],
		startingCountdown: null,
		postGame: null
	});

	const [wordBomb, setWordBomb] = createStore<WordBombData>({
		players: [],
		currentTurn: '',
		prompt: '',
		input: '',
		usedLetters: null
	});

	const [anagrams, setAnagrams] = createStore<AnagramsData>({
		players: [],
		anagram: ''
	});

	return [
		{ state, connection, lobby, wordBomb, anagrams },
		{ setState, setConnection, setLobby, setWordBomb, setAnagrams }
	] as const;
}

const Context = createContext<ReturnType<typeof makeContext>>();

const ContextProvider: ParentComponent<{ room: string }> = (props) => {
	return <Context.Provider value={makeContext(props.room)}>{props.children}</Context.Provider>;
};

export { Context, ContextProvider };
