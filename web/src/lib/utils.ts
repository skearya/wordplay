export type Variant<T extends { kind: string }, Kind extends T['kind']> = Extract<
	T,
	{ kind: Kind }
>;

export function unreachable(message: any) {
	throw new Error(message);
}

export function setupCanvas(
	canvas: HTMLCanvasElement,
	onFrame: (ctx: CanvasRenderingContext2D, dt: number) => void
) {
	const ctx = canvas.getContext('2d');

	if (!ctx) {
		throw new Error("getContext('2d') failed");
	}

	const onResize = () => {
		const dpr = window.devicePixelRatio || 1;

		canvas.width = canvas.clientWidth * dpr;
		canvas.height = canvas.clientHeight * dpr;

		ctx.scale(dpr, dpr);
	};

	onResize();

	window.addEventListener('resize', onResize);

	let rafId: number;
	let lastTime = performance.now();

	const rafFn = (time: number) => {
		onFrame(ctx, time - lastTime);

		lastTime = time;
		rafId = requestAnimationFrame(rafFn);
	};

	rafId = requestAnimationFrame(rafFn);

	return () => {
		if (rafId) cancelAnimationFrame(rafId);
		window.addEventListener('resize', onResize);
	};
}
