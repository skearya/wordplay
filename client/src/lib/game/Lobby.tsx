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
		<section class="flex h-screen w-full items-center justify-center">
			<h1 class="text-secondary fixed bottom-4 right-4 skew-x-6 text-7xl font-semibold italic">
				{status()}
			</h1>
			<div class="flex flex-col items-center gap-4 rounded-xl border bg-secondary-100 p-4">
				{lobby.previousWinner && <h1>winner: {lobby.previousWinner}</h1>}
				<div class="flex flex-col gap-3">
					<h1 class="text-2xl">Ready Players</h1>
					<div class="flex gap-4">
						<For each={lobby.readyPlayers}>
							{(player) => (
								<div class="flex min-w-36 flex-col items-center gap-2 rounded-lg border bg-secondary-200 p-2">
									<img
										class="rounded-full"
										width="100px"
										height="100px"
										src={`https://avatar.vercel.sh/${player.username}`}
										alt="avatar"
									/>
									<h1>{player.username}</h1>
								</div>
							)}
						</For>
						<For each={new Array(Math.max(0, 2 - lobby.readyPlayers.length))}>
							{() => (
								<div class="flex min-w-36 flex-col items-center gap-2 rounded-lg p-2 outline-dashed outline-2">
									<div class="flex h-[100px] w-[100px] items-center justify-center">?</div>
									<h1>Waiting...</h1>
								</div>
							)}
						</For>
					</div>
				</div>
				<button class="rounded-lg border p-2" onClick={() => props.sendMessage({ type: 'ready' })}>
					Ready
				</button>
			</div>
		</section>
	);
};

export { Lobby };
