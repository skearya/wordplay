import { useContext, type Component, For, createSignal, createEffect } from 'solid-js';
import { Context } from '../context';
import { ClientMessage } from '../types/messages';

const Lobby: Component<{ sendMessage: (message: ClientMessage) => void }> = (props) => {
	const context = useContext(Context);
	if (!context) throw new Error('Not called inside context provider?');
	const { lobby } = context[0];

	const [status, setStatus] = createSignal('waiting for players...');

	createEffect(() => {
		if (lobby.startingCountdown) {
			setStatus(`starting in ${lobby.startingCountdown}`);
		} else {
			setStatus(`waiting for players...`);
		}
	});

	return (
		<>
			<h1 class="text-outline fixed bottom-4 right-4 skew-x-6 text-7xl font-semibold italic text-background">
				{status()}
			</h1>
			<section class="flex min-h-screen w-full items-center justify-center">
				<div class="flex flex-col items-center gap-4 rounded-xl border bg-secondary-100 p-4">
					{lobby.previousWinner && <h1>winner: {lobby.previousWinner}</h1>}
					<div class="flex flex-col gap-3">
						<h1 class="text-2xl">Ready Players</h1>
						<div class="flex gap-4">
							<For each={lobby.readyPlayers}>
								{(player) => <Player username={player.username} />}
							</For>
							<For each={new Array(Math.max(0, 2 - lobby.readyPlayers.length))}>
								{() => <Player />}
							</For>
						</div>
					</div>
					<button
						class="rounded-lg border bg-secondary p-2"
						onClick={() => props.sendMessage({ type: 'ready' })}
					>
						Ready
					</button>
				</div>
			</section>
		</>
	);
};

const Player: Component<{ username?: string }> = (props) => {
	if (props.username) {
		return (
			<div class="flex min-w-36 flex-col items-center gap-2 rounded-xl border bg-secondary-200 p-2">
				<img
					class="h-[100px] w-[100px] rounded-full"
					src={`https://avatar.vercel.sh/${props.username}`}
					alt="avatar"
				/>
				<h1>{props.username}</h1>
			</div>
		);
	}

	return (
		<div class="flex min-w-36 animate-pulse flex-col items-center gap-2 rounded-xl p-2 outline-dashed outline-1 outline-accent-500">
			<div class="flex h-[100px] w-[100px] items-center justify-center text-xl">?</div>
			<h1>Waiting...</h1>
		</div>
	);
};

export { Lobby };
