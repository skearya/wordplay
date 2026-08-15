import Voronoi from '$lib/deps/voronoi/rhill-voronoi-core';

export function explode(
	targetElement: HTMLElement,
	{
		duration = 10000,
		pointCount = 10,
		distMultiplier = 2,
		elementModifications
	}: {
		duration?: number;
		pointCount?: number;
		distMultiplier?: number;
		elementModifications?: (element: HTMLElement) => void;
	}
) {
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

	const container = document.createElement('div');

	container.style.position = 'absolute';
	container.style.top = '0px';
	container.style.left = '0px';
	container.style.width = '100vw';
	container.style.height = '100vh';
	container.style.overflow = 'hidden';
	container.style.pointerEvents = 'none';

	document.body.appendChild(container);

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

		elementModifications?.(clone);

		const dist = Math.hypot(cell.site.y - centerY, cell.site.x - centerX);
		const angle = Math.atan2(cell.site.y - centerY, cell.site.x - centerX);

		const dx = Math.cos(angle) * dist * distMultiplier;
		const dy = Math.sin(angle) * dist * distMultiplier;

		container.appendChild(clone);

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
				translate: `${dx * 1.25}px ${dy + 300}px`
			},
			{
				fill: 'forwards',
				easing: 'ease-in',
				delay: 1000,
				duration
			}
		);

		clone.animate(
			{
				opacity: '0%',
				filter: 'brightness(200%) contrast(200%)'
			},
			{
				fill: 'forwards',
				easing: 'ease-out',
				duration
			}
		);
	}

	setTimeout(() => container.remove(), duration);
}
