import { ParentComponent, createContext, createSignal } from 'solid-js';
import { createStore } from 'solid-js/store';
import type { AnagramsData, ConnectionData, LobbyData, State, WordBombData } from './types/context';

function makeContext(room = '') {
	const [state, setState] = createSignal<State>('Connecting');

	const [connection, setConnection] = createStore<ConnectionData>({
		room,
		uuid: '',
		settings: { game: 'WordBomb', public: false },
		roomOwner: '',
		clients: [],
		chatMessages: []
	});

	const [lobby, setLobby] = createStore<LobbyData>({
		ready: [],
		startingCountdown: null,
		postGame: null
	});

	const [wordBomb, setWordBomb] = createStore<WordBombData>({
		players: [],
		turn: '',
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

export const Context = createContext<ReturnType<typeof makeContext>>();

export const ContextProvider: ParentComponent<{ room: string }> = (props) => {
	return <Context.Provider value={makeContext(props.room)}>{props.children}</Context.Provider>;
};
