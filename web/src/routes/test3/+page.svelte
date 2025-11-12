<script lang="ts">
	import { onMount } from 'svelte';
	import Voronoi from '$lib/voronoi/rhill-voronoi-core';

	let canvasElement: HTMLCanvasElement;
	let targetElement: HTMLElement;

	const POINTS = 10;

	function explode(targetElement: HTMLElement) {
		const bbox = targetElement.getBoundingClientRect();

		const width = bbox.width;
		const height = bbox.height;

		canvasElement.width = width;
		canvasElement.height = height;

		const diagram = new Voronoi().compute(
			Array.from({ length: POINTS }, () => ({
				x: Math.random() * width,
				y: Math.random() * height
			})),
			{ xl: 0, xr: width, yt: 0, yb: height }
		);

		const ctx = canvasElement.getContext('2d')!;

		ctx.fillStyle = 'white';
		ctx.fillRect(0, 0, width, height);

		ctx.strokeStyle = 'black';
		ctx.lineWidth = 1;

		ctx.beginPath();

		for (const cell of diagram.cells) {
			for (const edge of cell.halfedges) {
				ctx.moveTo(edge.getStartpoint().x, edge.getStartpoint().y);
				ctx.lineTo(edge.getEndpoint().x, edge.getEndpoint().y);
			}
		}

		ctx.stroke();

		const centerX = width / 2;
		const centerY = height / 2;

		for (const cell of diagram.cells) {
			const paths = [
				`M${cell.halfedges[0].getStartpoint().x},${cell.halfedges[0].getStartpoint().y}`
			];

			for (const edge of cell.halfedges) {
				paths.push(`L${edge.getEndpoint().x},${edge.getEndpoint().y}`);
			}

			paths.push('Z');

			const clone = targetElement.cloneNode(true) as HTMLElement;

			clone.style.position = 'absolute';
			clone.style.top = `${bbox.top}px`;
			clone.style.left = `${bbox.left}px`;
			clone.style.translate = `0px 0px`;
			clone.style.clipPath = `path("${paths.join(' ')}")`;

			const dist = Math.hypot(cell.site.y - centerY, cell.site.x - centerX);
			const angle = Math.atan2(cell.site.y - centerY, cell.site.x - centerX);

			const dx = Math.cos(angle) * dist * 1.5;
			const dy = Math.sin(angle) * dist * 1.5;

			document.body.appendChild(clone);

			clone.animate(
				{
					translate: `${dx}px ${dy}px`,
					rotate: `${Math.random() - 0.5}rad`
				},
				{
					fill: 'forwards',
					easing: 'cubic-bezier(0.7, 0, 0.3, 1)',
					duration: 350
				}
			);

			clone.animate(
				{
					opacity: '0%'
				},
				{
					fill: 'forwards',
					easing: 'ease-out',
					duration: 5000
				}
			);
		}

		targetElement.style.display = 'none';
	}

	onMount(() => {
		setTimeout(() => {
			explode(targetElement);
		}, 100);
	});
</script>

{#snippet heartIcon()}
	<svg width="26" height="22" viewBox="0 0 25 21" fill="none" xmlns="http://www.w3.org/2000/svg">
		<path
			d="M0 7.13885C0 13.0709 5.02429 16.2321 8.70216 19.0615C10 20.06 11.25 21 12.5 21C13.75 21 15 20.06 16.2979 19.0615C19.9757 16.2321 25 13.0709 25 7.13885C25 1.20675 18.1248 -3.00018 12.5 2.70287C6.8752 -3.00018 0 1.20675 0 7.13885Z"
			fill="#DA5858"
		/>
	</svg>
{/snippet}

<canvas bind:this={canvasElement}></canvas>

<div
	bind:this={targetElement}
	class="absolute left-1/2 top-1/2 flex -translate-x-1/2 -translate-y-1/2 flex-col items-center border p-2"
>
	<div class="relative mb-2">
		<img
			src={`https://avatar.vercel.sh/${0}`}
			alt="avatar"
			width="120"
			height="120"
			class="size-24 rounded-full"
		/>
		<div class="absolute bottom-0 left-0 flex flex-col gap-y-1.5 mix-blend-plus-lighter">
			{@render heartIcon()}
			{@render heartIcon()}
		</div>
	</div>
	<p>skeary</p>
	<p>ovens</p>
</div>
