/* @refresh reload */
import './index.css';
import { render } from 'solid-js/web';

import App from './App';

document.documentElement.classList[
	window.matchMedia('(prefers-color-scheme: light)').matches ? 'remove' : 'add'
]('dark');

const root = document.getElementById('root');

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
	throw new Error(
		'Root element not found. Did you forget to add it to your index.html? Or maybe the id attribute got misspelled?'
	);
}

render(() => <App />, root!);
