import { useContext, type Component, For, createSignal, createEffect, Show } from 'solid-js';
import { Context } from '../context';
import type { ClientMessage, PostGameInfo } from '../types/messages';
import type { ConnectionData } from '../types/stores';

const Lobby: Component<{ sender: (message: ClientMessage) => void }> = (props) => {
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
		<section class="flex min-h-screen w-full items-center justify-center gap-6">
			<h1 class="text-outline fixed bottom-0 right-4 skew-x-6 text-[6vw] font-semibold italic text-background">
				{status()}
			</h1>
			<div class="fixed bottom-1/2 left-4 space-y-2">
				<div>
					{['public', 'private'].map((visibility) => (
						<div class="space-x-2">
							<input
								type="radio"
								name={visibility}
								disabled={connection.roomOwner !== connection.uuid}
								checked={
									connection.settings.public ? visibility === 'public' : visibility === 'private'
								}
								onChange={(event) => {
									if (event.target.checked) {
										props.sender({
											type: 'roomSettings',
											...connection.settings,
											public: visibility === 'public'
										});
									}
								}}
							/>
							<label for={visibility}>{visibility}</label>
						</div>
					))}
				</div>
				<div class="h-[2px] w-full bg-primary-300"></div>
				<div>
					{['word bomb', 'anagrams'].map((game) => (
						<div class="space-x-2">
							<input
								type="radio"
								name={game}
								disabled={connection.roomOwner !== connection.uuid}
								checked={
									connection.settings.game === (game === 'word bomb' ? 'wordBomb' : 'anagrams')
								}
								onChange={(event) => {
									if (event.target.checked) {
										props.sender({
											type: 'roomSettings',
											...connection.settings,
											game: game === 'word bomb' ? 'wordBomb' : 'anagrams'
										});
									}
								}}
							/>
							<label for={game}>{game}</label>
						</div>
					))}
				</div>
			</div>
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
							props.sender({
								type: lobby.readyPlayers.includes(connection.uuid) ? 'unready' : 'ready'
							})
						}
					>
						{lobby.readyPlayers.includes(connection.uuid) ? 'Unready' : 'Ready'}
					</button>
					<Show when={connection.uuid === connection.roomOwner && lobby.readyPlayers.length >= 2}>
						<button
							class="rounded-lg border bg-sky-700 px-3 py-2"
							onClick={() => props.sender({ type: 'startEarly' })}
						>
							Start Early
						</button>
					</Show>
				</div>
			</div>
			<Show when={lobby.postGame}>
				<pre class="max-h-96 overflow-y-scroll text-xs">
					{postGameInfoString(lobby.postGame!, connection)}
				</pre>
			</Show>
		</section>
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

function postGameInfoString(data: PostGameInfo, connection: ConnectionData): string {
	const uuidToUsername = (uuid: string) =>
		connection.clients.find((client) => uuid === client.uuid)!.username;

	return JSON.stringify(
		Object.fromEntries(
			Object.entries(data).map(([k, v]) => {
				if (k === 'winner') {
					return [k, uuidToUsername(v as string)];
				}
				if (Array.isArray(v)) {
					return [k, v.map((item) => item.with(0, uuidToUsername(item[0])))];
				}
				return [k, v];
			})
		),
		null,
		2
	);
}

export { Lobby };
