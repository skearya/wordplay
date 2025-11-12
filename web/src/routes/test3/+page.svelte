<script lang="ts">
	import { onMount } from 'svelte';
	import Voronoi from '$lib/voronoi/rhill-voronoi-core';

	const width = 800;
	const height = 600;

	const diagram = new Voronoi().compute(
		Array.from({ length: 5 }, () => ({ x: Math.random() * width, y: Math.random() * 600 })),
		{ xl: 0, xr: width, yt: 0, yb: height }
	);

	let canvasElement: HTMLCanvasElement;
	let target: HTMLElement;

	onMount(() => {
		const ctx = canvasElement.getContext('2d')!;

		ctx.fillStyle = 'white';
		ctx.fillRect(0, 0, width, height);

		ctx.strokeStyle = 'black';
		ctx.lineWidth = 1;

		ctx.beginPath();

		// for (let i = 0; i < diagram.edges.length; i++) {
		// 	ctx.moveTo(diagram.edges[i].va.x, diagram.edges[i].va.y);
		// 	ctx.lineTo(diagram.edges[i].vb.x, diagram.edges[i].vb.y);
		// }

		for (const cell of diagram.cells) {
			for (const edge of cell.halfedges) {
				ctx.moveTo(edge.getStartpoint().x, edge.getStartpoint().y);
				ctx.lineTo(edge.getEndpoint().x, edge.getEndpoint().y);
			}
		}

		ctx.stroke();

		const clone = target.cloneNode(true) as HTMLElement;
		clone.style.clipPath = 'polygon(0 0, 50% 0, 50% 50%, 0 50%)';

		target.parentElement!.appendChild(clone);

		clone.animate(
			{
				translate: 'calc(-50% + -40px) calc(-50% + -40px)'
			},
			{
				fill: 'forwards',
				easing: 'ease-out',
				duration: 200
			}
		);
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

<canvas bind:this={canvasElement} width="800" height="600"></canvas>

<div
	bind:this={target}
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
