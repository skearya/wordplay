export type Variant<T extends { kind: string }, Kind extends T['kind']> = Extract<
	T,
	{ kind: Kind }
>;

export function unreachable(message: any) {
	throw new Error(message);
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
