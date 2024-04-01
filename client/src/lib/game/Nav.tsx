import { For, type Component, useContext } from 'solid-js';
import { LinkIcon, QuestionMarkIcon } from '../icons';
import { Context } from '../context';
import { ClientInfo } from '../types/messages';

const Nav: Component = () => {
	const context = useContext(Context);
	if (!context) throw new Error('Not called inside context provider?');
	const { connection } = context[0];

	return (
		<nav class="fixed top-0 flex w-full justify-between p-6">
			<h1 class="text-xl">wordplay</h1>
			<div class="flex items-center gap-4">
				<ConnectedClients clients={connection.clients} />
				<button onClick={async () => await navigator.clipboard.writeText(window.location.href)}>
					<LinkIcon />
				</button>
				<button onClick={() => alert('unimplemented')}>
					<QuestionMarkIcon />
				</button>
			</div>
		</nav>
	);
};

const ConnectedClients: Component<{ clients: ClientInfo[] }> = (props) => {
	return (
		<div class="group relative flex items-center -space-x-2 pr-1">
			<For each={props.clients}>
				{(client) => (
					<img
						class="h-9 w-9 rounded-full border-2 border-black"
						src={`https://avatar.vercel.sh/${client.username}`}
						alt="avatar"
					/>
				)}
			</For>
			<div class="absolute left-1/2 top-0 hidden -translate-x-1/2 translate-y-12 rounded-xl border bg-primary-50 p-2 group-hover:block">
				<div class="flex flex-col gap-y-2">
					<For each={props.clients}>
						{(client) => (
							<div class="flex min-w-40 items-center justify-between gap-2 rounded-lg border bg-primary-100 p-2">
								<h1>{client.username}</h1>
								<img
									class="h-8 w-8 rounded-full"
									src={`https://avatar.vercel.sh/${client.username}`}
									alt="avatar"
								/>
							</div>
						)}
					</For>
				</div>
			</div>
		</div>
	);
};

export { Nav };
