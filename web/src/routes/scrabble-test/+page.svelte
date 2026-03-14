<script lang="ts">
	import { onMount } from 'svelte';
	import { setupCanvas } from '$lib/canvas';
	import { clamp } from '$lib/utils';

	let canvasElement: HTMLCanvasElement;

	let dragging = false;
	let dragStart = { x: 0, y: 0 };

	let scale = 1;
	let position = { x: 0, y: 0 };

	let gridSize = 64;
	let gridDimensions = { x: 8, y: 8 };

	onMount(() => {
		return setupCanvas(canvasElement, (ctx, dt, width, height, duration) => {
			ctx.save();

			ctx.clearRect(0, 0, width, height);

			ctx.lineWidth = 1;
			ctx.fillStyle = 'white';
			ctx.strokeStyle = 'white';
			ctx.beginPath();

			ctx.translate(width / 2 + position.x, height / 2 + position.y);
			ctx.scale(scale, scale);

			for (let x = 0; x <= gridDimensions.x * gridSize; x += gridSize) {
				ctx.moveTo(x, 0);
				ctx.lineTo(x, gridSize * gridDimensions.x);
			}

			for (let y = 0; y <= gridDimensions.y * gridSize; y += gridSize) {
				ctx.moveTo(0, y);
				ctx.lineTo(gridSize * gridDimensions.y, y);
			}

			ctx.arc(64 * 8, 64 * 8, 10, 0, 2 * Math.PI);

			ctx.stroke();
			ctx.fill();

			ctx.restore();
		});
	});
</script>

<canvas
	bind:this={canvasElement}
	onpointerdown={(e) => {
		dragging = true;
		dragStart = { x: e.clientX - position.x, y: e.clientY - position.y };
	}}
	onpointermove={(e) => {
		if (!dragging) return;
		position = { x: e.clientX - dragStart.x, y: e.clientY - dragStart.y };
	}}
	onpointerup={() => (dragging = false)}
	onwheel={(e) => (scale = clamp(scale + e.deltaY * 0.01, 0.5, 2.0))}
	class="h-screen w-screen"
></canvas>
