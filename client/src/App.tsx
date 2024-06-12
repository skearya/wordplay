import { Route, Router } from '@solidjs/router';
import { lazy, type Component } from 'solid-js';

const Root = lazy(() => import('./routes/Root'));
const Game = lazy(() => import('./routes/Game'));
const NotFound = lazy(() => import('./routes/NotFound'));

const App: Component = () => {
	return (
		<Router>
			<Route path="/" component={Root} />
			<Route path="/game/:room" component={Game} />
			<Route path="*" component={NotFound} />
		</Router>
	);
};

export default App;
