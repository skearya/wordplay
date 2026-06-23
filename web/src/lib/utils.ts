import type { Attachment } from 'svelte/attachments';
import { dev } from '$app/environment';

export type Variant<T extends { kind: string }, Kind extends T['kind']> = Extract<
	T,
	{ kind: Kind }
>;

export function unreachable(message: any) {
	throw new Error(message);
}

export function getOrSet<K, V>({ map, key, val }: { map: Map<K, V>; key: K; val: V }) {
	const stored = map.get(key);

	if (stored) {
		return stored;
	} else {
		map.set(key, val);
		return val;
	}
}

export function objectAssign<T extends {}>(target: T, source: T): T {
	return Object.assign(target, source);
}

export function getRandomRange(min: number, max: number) {
	return Math.random() * (max - min) + min;
}

export function getRandomWithBias(min: number, max: number, bias: number, influence: number) {
	const rnd = Math.random() * (max - min) + min,
		mix = Math.random() * influence;
	return rnd * (1 - mix) + bias * mix;
}

export const lerp = (start: number, end: number, amount: number) =>
	start * (1 - amount) + end * amount;

export const clamp = (num: number, min: number, max: number) => Math.min(Math.max(num, min), max);

export function debounce<T extends any[]>(
	callback: (...args: T) => void,
	wait: number
): (...args: T) => void {
	let timeoutId: number | undefined = undefined;

	return (...args: T) => {
		clearTimeout(timeoutId);

		timeoutId = setTimeout(() => {
			callback(...args);
		}, wait);
	};
}

export function keepAlphanumeric(input: string) {
	return input.replace(/[^a-zA-Z0-9]/g, '');
}

export function camelCaseToWords(s: string) {
	const result = s.replace(/([A-Z])/g, ' $1');
	return result.charAt(0).toUpperCase() + result.slice(1);
}

export function onClickOutside(callback: () => void): Attachment {
	return (node) => {
		function handleClick(event: MouseEvent) {
			if (!node.contains(event.target as Node)) {
				callback();
			}
		}

		document.addEventListener('click', handleClick, true);

		return () => document.removeEventListener('click', handleClick, true);
	};
}

export function serverURL(path: string, ws?: boolean) {
	if (dev) {
		return `${ws ? 'ws' : 'http'}://localhost:3000${path}`;
	}

	return path;
}
