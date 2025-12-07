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

export function setupCanvas(
	canvas: HTMLCanvasElement,
	onFrame: (ctx: CanvasRenderingContext2D, dt: number, width: number, height: number) => void
) {
	const ctx = canvas.getContext('2d');

	if (!ctx) {
		throw new Error("getContext('2d') failed");
	}

	// `true` to make sure `if (resized)` code runs once
	let resized = true;

	const canvasResizeObserver = new ResizeObserver(() => (resized = true));
	canvasResizeObserver.observe(canvas);

	let rafId: number;
	let lastTime = performance.now();
	let width: number;
	let height: number;

	const rafFn = (time: number) => {
		if (resized) {
			const dpr = window.devicePixelRatio || 1;

			width = canvas.clientWidth;
			height = canvas.clientHeight;

			canvas.width = width * dpr;
			canvas.height = height * dpr;

			ctx.scale(dpr, dpr);

			resized = false;
		}

		onFrame(ctx, time - lastTime, width, height);

		lastTime = time;
		rafId = requestAnimationFrame(rafFn);
	};

	rafId = requestAnimationFrame(rafFn);

	return () => {
		if (rafId) cancelAnimationFrame(rafId);
		canvasResizeObserver.disconnect();
	};
}

export const lerp = (start: number, end: number, amount: number) =>
	start * (1 - amount) + end * amount;

// Source - https://stackoverflow.com/questions/29325069/how-to-generate-random-numbers-biased-towards-one-value-in-a-range
// Posted by user1693593, modified by community. See post 'Timeline' for change history
// Retrieved 2025-12-06, License - CC BY-SA 3.0
function randomWithBias(min: number, max: number, bias: number, influence: number) {
	const rnd = Math.random() * (max - min) + min,
		mix = Math.random() * influence;
	return rnd * (1 - mix) + bias * mix;
}
