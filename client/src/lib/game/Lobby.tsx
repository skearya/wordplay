import { useContext, type Component, For, createSignal, createEffect, Show } from 'solid-js';
import { Context } from '../context';
import { ClientMessage } from '../types/messages';

const Lobby: Component<{ sendMessage: (message: ClientMessage) => void }> = (props) => {
	const context = useContext(Context);
	if (!context) throw new Error('Not called inside context provider?');
	const { connection, lobby } = context[0];

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
			<h1 class="text-outline fixed bottom-0 right-4 skew-x-6 text-[6vw] font-semibold italic text-background">
				{status()}
			</h1>
			<section class="flex min-h-screen w-full flex-col items-center justify-center gap-4">
				<Show when={lobby.previousWinner}>
					<div class="flex items-center gap-2 rounded-xl border p-4">
						<h1 class="pr-2">Winner:</h1>
						<img
							class="h-10 w-10 rounded-full"
							src={`https://avatar.vercel.sh/${lobby.previousWinner}`}
							alt="avatar"
						/>
						<h1>{lobby.previousWinner}</h1>
					</div>
				</Show>
				<div class="flex flex-col items-center gap-4 rounded-xl border bg-secondary-100 p-4">
					<div class="flex flex-col gap-3">
						<h1 class="text-2xl">Ready Players</h1>
						<div class="flex gap-4">
							<For each={lobby.readyPlayers}>
								{(ready) => (
									<Player
										username={connection.clients.find((client) => client.uuid === ready)!.username}
									/>
								)}
							</For>
							<For each={new Array(Math.max(0, 2 - lobby.readyPlayers.length))}>
								{() => <Player />}
							</For>
						</div>
					</div>
					<div class="flex gap-3">
						<button
							class="rounded-lg border bg-secondary px-3 py-2"
							onClick={() =>
								props.sendMessage({
									type: lobby.readyPlayers.includes(connection.uuid) ? 'unready' : 'ready'
								})
							}
						>
							{lobby.readyPlayers.includes(connection.uuid) ? 'Unready' : 'Ready'}
						</button>
						<Show when={connection.uuid === connection.roomOwner && lobby.readyPlayers.length >= 2}>
							<button
								class="rounded-lg border bg-sky-700 px-3 py-2"
								onClick={() => props.sendMessage({ type: 'startEarly' })}
							>
								Start Early
							</button>
						</Show>
					</div>
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
		<div class="flex min-w-36 flex-col items-center gap-2 rounded-xl p-2 outline-dashed outline-1 outline-accent-500">
			<div class="flex h-[100px] w-[100px] items-center justify-center text-xl">?</div>
			<h1>Waiting...</h1>
		</div>
	);
};

export { Lobby };
