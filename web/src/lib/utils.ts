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
