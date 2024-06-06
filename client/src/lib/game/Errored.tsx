import type { Accessor, Component } from 'solid-js';

export const Errored: Component<{ errorMessage: Accessor<string | null> }> = (props) => {
	return (
		<section class="flex min-h-screen flex-col items-center justify-center">
			<h1>we errored</h1>
			{props.errorMessage() && <h1>{props.errorMessage()}</h1>}
		</section>
	);
};
