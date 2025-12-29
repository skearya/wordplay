<script lang="ts">
	import { onMount } from 'svelte';
	import { Spring } from 'svelte/motion';

	let targetElement: HTMLElement;

	const x = new Spring(0);
	const y = new Spring(0);
	const scale = new Spring(1);

	let lastTime = 0;
	let prevX = 0;
	let prevY = 0;
	let velocityX = 0;
	let velocityY = 0;

	let pressed = false;
	let startX = $state(0);
	let startY = $state(0);

	onMount(() => {
		const controller = new AbortController();
		const signal = controller.signal;

		document.addEventListener('pointermove', onDocumentPointerMove, { signal });
		document.addEventListener('pointerup', onDocumentPointerUp, { signal });

		return () => controller.abort();
	});

	$effect(() => {
		const time = performance.now();
		const dt = (time - lastTime) / 1000;

		velocityX = (x.current - prevX) / dt;
		velocityY = (y.current - prevY) / dt;
		prevX = x.current;
		prevY = y.current;
		lastTime = time;

		console.log(velocityX, velocityY);
	});

	function onCardPointerDown(e: PointerEvent) {
		pressed = true;

		startX = e.clientX - x.current;
		startY = e.clientY - y.current;

		scale.set(0.9);
	}

	function onDocumentPointerMove(e: PointerEvent) {
		if (!pressed) return;

		x.set(e.clientX - startX);
		y.set(e.clientY - startY);
	}

	function onDocumentPointerUp() {
		pressed = false;

		scale.set(1);
	}
</script>

<div
	bind:this={targetElement}
	onpointerdown={onCardPointerDown}
	style={`translate: ${x.current}px ${y.current}px;` +
		`scale: ${scale.current};` +
		`transform-origin: ${startX}px ${startY}px;`}
	class="bg-pastel-pink h-96 w-64 rounded"
></div>
