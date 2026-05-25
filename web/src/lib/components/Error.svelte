<script lang="ts">
	import type { Attachment } from 'svelte/attachments';
	import { fade } from 'svelte/transition';
	import { createLetterCanvas } from '$lib/letters';
	import Button from '$lib/ui/Button.svelte';
	import { getRandomRange } from '$lib/utils';
	import Tag from './Tag.svelte';

	let { message, fatal }: { message?: string; fatal?: boolean } = $props();

	const setupCanvas: Attachment<HTMLCanvasElement> = (canvas) => {
		const cleanupCanvas = createLetterCanvas(canvas, {
			style: { letterColor: '#ff645c', letterOutlineColor: '#ff645c', textColor: '#fec5bb' },
			gravity: 1.0,
			initLetters: (width, height) =>
				`╱|、(˚ˎ。7|、˜〵じしˍ,)ノ????`
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

<svg width="0">
	<defs>
		<filter id="background-filter">
			<feTurbulence
				type="turbulence"
				baseFrequency="0.2"
				numOctaves="1"
				seed="1"
				result="turbulence"
			/>
			<feComposite operator="in" in="turbulence" in2="SourceAlpha" result="composite" />
			<feDisplacementMap in="blended" in2="turbulence" scale="10" />
		</filter>
	</defs>
</svg>

<main out:fade>
	<div
		style="background-image: linear-gradient(90deg, rgba(16, 185, 129, 0.5) 1px, transparent 0), linear-gradient(180deg, rgba(16, 185, 129, 0.5) 1px, transparent 0), repeating-linear-gradient(45deg, rgba(16, 185, 129, 0.5) 0 2px, transparent 2px 6px); background-size: 24px 24px, 24px 24px, 24px 24px; filter: url(#background-filter);"
		class="absolute top-0 left-0 -z-10 h-screen w-screen overflow-hidden"
	></div>
	<div class="absolute top-0 left-0 -z-10 h-screen w-screen overflow-hidden">
		<canvas {@attach setupCanvas} class="h-full w-full"></canvas>
	</div>
	<div
		style="background: radial-gradient(at top left, var(--color-background) 0%, color-mix(in srgb, var(--color-red) 15%, transparent) 100%), var(--color-background);"
		class="absolute top-28 left-1/2 flex h-64 w-full max-w-xl -translate-x-1/2 flex-col items-stretch justify-center gap-y-4 border border-red bg-background p-4"
	>
		<Tag class="bg-red">
			<p>Something {fatal ? 'went really' : 'went'} wrong...</p>
		</Tag>
		<div class="flex flex-1 items-center justify-center">
			<p class="line-clamp-3 text-center font-mono">{message ? message : 'Unknown Error'}</p>
		</div>
		<Button color="pastel-blue" onclick={() => location.reload()}>Reload</Button>
	</div>
</main>
