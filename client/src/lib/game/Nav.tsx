import type { Component } from 'solid-js';
import { LinkIcon } from '../icons';

const Nav: Component = () => {
	return (
		<nav class="fixed top-0 flex w-full justify-between p-6">
			<h1 class="text-xl">wordplay</h1>
			<button onClick={async () => await navigator.clipboard.writeText(window.location.href)}>
				<LinkIcon />
			</button>
		</nav>
	);
};

export { Nav };
