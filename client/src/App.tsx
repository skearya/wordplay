import type { Component } from 'solid-js';
import { ContextProvider } from './lib/context';
import { Game } from './Game';

const App: Component = () => {
	return (
		<ContextProvider>
			<Game />
		</ContextProvider>
	);
};

export { App };
