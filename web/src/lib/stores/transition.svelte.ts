import type { Context } from '$lib/context';

type State = { kind: 'idle' } | { kind: 'transitioning'; update: Context['state'] };

export const transitionState = $state<State>({ kind: 'idle' });
