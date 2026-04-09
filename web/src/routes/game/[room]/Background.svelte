<script lang="ts">
	import type { Attachment } from 'svelte/attachments';
	import { createLetterCanvas, darkStyle } from '$lib/letters';
	import { getRandomRange } from '$lib/utils';

	const setupCanvas: Attachment<HTMLCanvasElement> = (canvas) => {
		const cleanupCanvas = createLetterCanvas(canvas, {
			style: darkStyle,
			gravity: 0.05,
			initLetters: (width, height) =>
				'wordplaybyskeary.me'
					.split('')
					.reverse()
					.map((letter) => ({
						letter,
						x: getRandomRange(75, width - 75),
						y: getRandomRange(75, height - 75),
						angle: Math.random() - 0.5 * 2.0
					})),
			initialForce: true
		});

		return () => cleanupCanvas();
	};
</script>

<div class="absolute top-0 left-0 -z-10 h-screen w-screen overflow-hidden">
	<canvas {@attach setupCanvas} class="background background-scroll h-full w-full"></canvas>
</div>

<style>
	.background {
		animation:
			400ms cubic-bezier(0.33, 1, 0.68, 1) background-fade-in,
			120s linear infinite background-scroll-keyframes;
		background-image:
			repeating-linear-gradient(
				90deg,
				transparent,
				transparent 30px,
				rgba(233, 184, 255, 0.06) 30px,
				rgba(233, 184, 255, 0.06) 31px
			),
			repeating-linear-gradient(
				150deg,
				transparent,
				transparent 35px,
				rgba(233, 184, 255, 0.04) 35px,
				rgba(233, 184, 255, 0.04) 36px
			);
	}

	@keyframes background-fade-in {
		0% {
			scale: 1.1;
		}
		100% {
			scale: 1;
		}
	}
</style>
