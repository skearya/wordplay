import { useContext, type Component } from 'solid-js';
import { Context } from '../context';
import { ClientMessage } from '../types/messages';

const Lobby: Component<{ sendMessage: (message: ClientMessage) => void }> = (props) => {
	const context = useContext(Context);
	if (!context) throw new Error('Not called inside context provider?');
	const { lobby } = context[0];

	return (
		<>
			{lobby.startingCountdown && <h1>starting soon: {lobby.startingCountdown}</h1>}
			{lobby.previousWinner && <h1>winner: {lobby.previousWinner}</h1>}
			<h1>ready players: {lobby.readyPlayers.map((player) => player.username).join(' ')}</h1>
			<button onClick={() => props.sendMessage({ type: 'ready' })}>ready</button>
		</>
	);
};

export { Lobby };
