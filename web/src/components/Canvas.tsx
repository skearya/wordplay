export function Canvas() {
	return <canvas ref={initCanvas} width="800px" height="450px" />;
}

function initCanvas(canvas: HTMLCanvasElement) {
	const ctx = canvas.getContext("2d");

	if (!ctx) {
		throw new Error("canvas.getContext('2d') failed");
	}

	let rafId: number;
	let lastTime = performance.now();

	const frame = (time: number) => {
		draw(ctx, time - lastTime);

		rafId = requestAnimationFrame(frame);
		lastTime = time;
	};

	rafId = requestAnimationFrame(frame);

	return () => cancelAnimationFrame(rafId);
}

function draw(ctx: CanvasRenderingContext2D, dt: number) {
	ctx.fillStyle = "orange";
	ctx.fillRect(0, 0, 100, 100);
}
