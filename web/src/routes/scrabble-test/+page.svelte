<script lang="ts">
	import { onMount } from 'svelte';
	import { setupCanvas } from '$lib/canvas';
	import { clamp } from '$lib/utils';

	let canvasElement: HTMLCanvasElement;

	let dragging = false;
	let dragStart = { x: 0, y: 0 };

	let gridSize = 64;
	let position = { x: 0, y: 0 };

	onMount(() => {
		return setupCanvas(canvasElement, (ctx, dt, width, height, duration) => {
			ctx.clearRect(0, 0, width, height);

			ctx.lineWidth = 1;
			ctx.strokeStyle = '#161a15';
			ctx.beginPath();

			const offset = { x: position.x % gridSize, y: position.y % gridSize };

			for (let x = offset.x; x < width; x += gridSize) {
				ctx.moveTo(x, 0);
				ctx.lineTo(x, height);
			}

			for (let y = offset.y; y < width; y += gridSize) {
				ctx.moveTo(0, y);
				ctx.lineTo(width, y);
			}

			ctx.stroke();
		});
	});
</script>

<canvas
	bind:this={canvasElement}
	onpointerdown={(e) => {
		dragging = true;
		dragStart = { x: e.clientX, y: e.clientY };
	}}
	onpointermove={(e) => {
		if (!dragging) return;
		position = { x: e.clientX - dragStart.x, y: e.clientY - dragStart.y };
	}}
	onpointerup={() => (dragging = false)}
	onwheel={(e) => (gridSize = clamp(gridSize + e.deltaY, 8, 96))}
	class="h-screen w-screen"
></canvas>
