import { Route, Router, useParams } from '@solidjs/router';
import { lazy, type Component } from 'solid-js';
import { ContextProvider } from './lib/context';

const Root = lazy(() => import('./routes/Root'));
const Game = lazy(() => import('./routes/Game'));
const NotFound = lazy(() => import('./routes/NotFound'));

const App: Component = () => {
	return (
		<Router>
			<Route path="/" component={Root} />
			<Route
				path="/game/:room"
				component={() => (
					<ContextProvider room={useParams().room}>
						<Game />
					</ContextProvider>
				)}
			/>
			<Route path="*" component={NotFound} />
		</Router>
	);
};

export default App;
