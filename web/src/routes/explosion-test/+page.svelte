<script lang="ts">
	import { onMount } from 'svelte';
	import Voronoi from '$lib/deps/voronoi/rhill-voronoi-core';

	let targetElement: HTMLElement;

	function explode(targetElement: HTMLElement, { pointCount = 10, distMultiplier = 2 } = {}) {
		const { top, left, width, height } = targetElement.getBoundingClientRect();

		const diagram = new Voronoi().compute(
			Array.from({ length: pointCount }, () => ({
				x: Math.random() * width,
				y: Math.random() * height
			})),
			{ xl: 0, xr: width, yt: 0, yb: height }
		);

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
			clone.style.top = `${top}px`;
			clone.style.left = `${left}px`;
			clone.style.translate = `0px 0px`;
			clone.style.clipPath = `path("${paths.join(' ')}")`;

			const dist = Math.hypot(cell.site.y - centerY, cell.site.x - centerX);
			const angle = Math.atan2(cell.site.y - centerY, cell.site.x - centerX);

			const dx = Math.cos(angle) * dist * distMultiplier;
			const dy = Math.sin(angle) * dist * distMultiplier;

			document.body.appendChild(clone);

			clone.animate(
				{
					translate: `${dx}px ${dy}px`,
					rotate: `${Math.random() - 0.5}rad`
				},
				{
					fill: 'forwards',
					easing: 'cubic-bezier(0.1, 1.0, 0.9, 1)',
					duration: 1000
				}
			);

			clone.animate(
				{
					translate: `${dx}px ${dy + 300}px`
				},
				{
					fill: 'forwards',
					easing: 'ease-in',
					duration: 9000,
					delay: 1000
				}
			);

			clone
				.animate(
					{
						opacity: '0%',
						filter: 'brightness(200%) contrast(1000%)'
					},
					{
						fill: 'forwards',
						easing: 'ease-out',
						duration: 10000
					}
				)
				.finished.then(() => clone.remove());
		}

		targetElement.style.display = 'none';
	}

	onMount(() => {
		setTimeout(() => {
			explode(targetElement);
		}, 200);
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

<div
	bind:this={targetElement}
	class="absolute top-1/2 left-1/2 flex -translate-x-1/2 -translate-y-1/2 flex-col items-center border p-2"
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
