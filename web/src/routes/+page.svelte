<script lang="ts">
	import type { PageProps } from './$types';
	import Matter from 'matter-js';
	import { onMount } from 'svelte';
	import gridSvg from '$lib/assets/grid.svg';
	import homepageNoiseImage from '$lib/assets/homepage-noise.png';
	import Bomb from '$lib/icons/Bomb.svelte';
	import Github from '$lib/icons/Github.svelte';
	import Me from '$lib/icons/Me.svelte';
	import Search from '$lib/icons/Search.svelte';
	import Settings from '$lib/icons/Settings.svelte';
	import { lerp, setupCanvas } from '$lib/utils';

	const { data }: PageProps = $props();

	const { Engine, Bodies, Composite, Mouse, MouseConstraint, Vector, Body } = Matter;

	let backgroundCanvasElement: HTMLCanvasElement;
	let headerTextElement: HTMLElement;
	let contentElement: HTMLElement;

	onMount(() => {
		const animation = contentElement.animate(
			{
				translate: ['0px 50vh', '0px 0px']
			},
			{
				fill: 'forwards',
				delay: 500,
				duration: 1000,
				easing: 'cubic-bezier(0.87, 0, 0.13, 1)'
			}
		);

		headerTextElement.animate(
			{
				opacity: '100%'
			},
			{
				fill: 'forwards',
				delay: 1500,
				duration: 750,
				easing: 'ease-in'
			}
		);

		const originalWidth = backgroundCanvasElement.clientWidth;
		const originalHeight = backgroundCanvasElement.clientHeight;

		const engine = Engine.create({ gravity: { y: 0.01 } });

		const letterWidth = 64;
		const letterHeight = 64;
		const letters = new Map(
			['wordplay', 'byskeary.me', 'abcdefghijkl'].toReversed().flatMap((line, lineIndex) =>
				line.split('').map((letter, letterIndex) => {
					const lineWidth = line.length * (letterWidth + 4);
					const start = originalWidth / 2 - lineWidth / 2;

					const x = start + letterIndex * (letterWidth + 4) + letterWidth / 2;
					const y = originalHeight - lineIndex * letterHeight - letterHeight / 2;

					const letterBody = Bodies.rectangle(x, y, letterWidth, letterHeight, {
						angle: (Math.random() - 0.5) * 0.05
					});

					return [letterBody, letter] as const;
				})
			)
		);

		Composite.add(engine.world, [...letters.keys()]);

		const wallThickness = 100;
		const wallOptions = { isStatic: true };

		const leftWall = Bodies.rectangle(
			0 - wallThickness / 2,
			originalHeight / 2,
			wallThickness,
			originalHeight,
			wallOptions
		);
		const rightWall = Bodies.rectangle(
			originalWidth + wallThickness / 2,
			originalHeight / 2,
			wallThickness,
			originalHeight,
			wallOptions
		);
		const topWall = Bodies.rectangle(
			originalWidth / 2,
			0 - wallThickness / 2,
			originalWidth,
			wallThickness,
			wallOptions
		);
		const bottomWall = Bodies.rectangle(
			originalWidth / 2,
			originalHeight + wallThickness / 2,
			originalWidth,
			wallThickness,
			wallOptions
		);

		Composite.add(engine.world, [leftWall, rightWall, topWall, bottomWall]);

		const mouse = Mouse.create(backgroundCanvasElement);

		// https://github.com/liabru/matter-js/issues/678
		// https://github.com/liabru/matter-js/issues/929
		// @ts-expect-error
		mouse.element.removeEventListener('touchmove', mouse.mousemove);
		// @ts-expect-error
		mouse.element.removeEventListener('touchstart', mouse.mousedown);
		// @ts-expect-error
		mouse.element.removeEventListener('touchend', mouse.mouseup);
		// @ts-expect-error
		mouse.element.removeEventListener('wheel', mouse.mousewheel);
		// @ts-expect-error
		mouse.element.removeEventListener('DOMMouseScroll', mouse.mousewheel);

		// TODO: Update on zoom
		mouse.pixelRatio = window.devicePixelRatio || 1;

		const mouseConstraint = MouseConstraint.create(engine, {
			mouse: mouse,
			constraint: {
				stiffness: 1.0,
				render: { visible: false }
			}
		});

		Composite.add(engine.world, mouseConstraint);

		const cleanupCanvas = setupCanvas(backgroundCanvasElement, (ctx, dt, width, height) => {
			const timing = animation?.effect?.getComputedTiming().progress ?? 0;
			const bottom = lerp(height, height / 2, timing);

			Body.setPosition(leftWall, Vector.create(0 - wallThickness / 2, height / 2));
			Body.setPosition(rightWall, Vector.create(width + wallThickness / 2, height / 2));
			Body.setPosition(topWall, Vector.create(width / 2, 0 - wallThickness / 2));
			Body.setPosition(
				bottomWall,
				Vector.create(width / 2, bottom + wallThickness / 2),
				// @ts-expect-error Sets velocity. Not typed for some reason.
				true
			);

			for (const [body] of letters) {
				const clampedX = Math.max(Math.min(body.position.x, width), 0);
				const clampedY = Math.max(Math.min(body.position.y, height), 0);

				if (clampedX !== body.position.x || clampedY !== body.position.y) {
					Body.setPosition(body, Vector.create(clampedX, clampedY));
					Body.setVelocity(body, Vector.create(0, 0));
				}
			}

			Engine.update(engine, Math.min(dt, 60));

			ctx.clearRect(0, 0, width, bottom);

			ctx.lineWidth = 1;
			ctx.textBaseline = 'middle';

			for (const [box, letter] of letters) {
				ctx.save();

				ctx.translate(box.position.x, box.position.y);
				ctx.rotate(box.angle);

				ctx.fillStyle = `rgba(255, 255, 255, ${Math.min(Vector.magnitude(box.velocity) / 2, 0.8)})`;
				ctx.fillRect(-letterWidth / 2, -letterHeight / 2, letterWidth, letterHeight);
				ctx.fillStyle = 'black';

				ctx.strokeRect(-letterWidth / 2, -letterHeight / 2, letterWidth, letterHeight);

				ctx.font = '40px sans-serif';
				ctx.textAlign = 'center';
				ctx.fillText(letter, 0, 0);

				ctx.font = '12px sans-serif';
				ctx.textAlign = 'right';
				ctx.fillText(`${letter.charCodeAt(0)}`, letterWidth / 2 - 4, letterHeight / 2 - 10);

				ctx.restore();
			}
		});

		return () => cleanupCanvas();
	});
</script>

<header
	style={`background-image: url("${homepageNoiseImage}");`}
	class="absolute top-0 left-0 -z-10 h-screen w-screen bg-cover"
>
	<canvas bind:this={backgroundCanvasElement} class="h-full w-full"></canvas>
	<h1
		bind:this={headerTextElement}
		class="pointer-events-none absolute bottom-[50vh] left-0 p-4 font-serif text-8xl text-background opacity-0"
	>
		Wordplay
	</h1>
</header>

<section
	bind:this={contentElement}
	style={`background-image: url("${gridSvg}");`}
	class="background-scroll mt-[50vh] flex min-h-[64rem] translate-y-[50vh] items-start gap-2.5 bg-background bg-repeat p-4 inset-shadow-[0_20px_20px] inset-shadow-black"
>
	<div class="sticky top-4 w-[325px] space-y-2.5 text-background">
		<button class="block w-full bg-pastel-red py-7 text-2xl font-medium">Join room</button>
		<button class="block w-full bg-pastel-light-red py-7 text-2xl font-medium">Create room</button>
		<button class="block w-full bg-pastel-green py-7 text-2xl font-medium">Singleplayer</button>
		<div class="flex items-center gap-x-2.5 p-2.5">
			<Settings />
			<Github />
			<Me width={42} height={42} class="ml-auto" />
		</div>
	</div>
	<div class="flex-1 space-y-2.5 p-2.5">
		<div class="flex items-center justify-between font-serif">
			<h1 class="text-2xl">Public rooms</h1>
			<div class="flex items-center justify-between gap-x-4 text-[#B0B0B0]">
				<input type="text" placeholder="Search..." class="text-xl" />
				<Search />
			</div>
		</div>
		<div class="grid grid-cols-3 gap-2.5">
			{#each { length: 12 }}
				<a href="/" class="relative border border-faded-green bg-dark-green p-2.5">
					<h1 class="mb-10 text-lg">Stupid Room Name</h1>
					<div class="flex -space-x-2">
						{#each { length: 3 }, i}
							<img
								src={`https://avatar.vercel.sh/${i}`}
								width="38px"
								height="38px"
								alt="avatar"
								class="aspect-square size-[38px] rounded-full border-2 border-background"
							/>
						{/each}
						<p
							class="flex aspect-square size-[38px] items-center justify-center rounded-full bg-background"
						>
							+21
						</p>
					</div>
					<Bomb class="absolute right-2.5 bottom-2.5" />
				</a>
			{/each}
		</div>
	</div>
</section>

<style>
	@keyframes background-scroll-keyframes {
		0% {
			background-position: 0px 0px;
		}
		100% {
			background-position: -100vw 100vh;
		}
	}

	.background-scroll {
		animation: 120s linear infinite background-scroll-keyframes;
	}
</style>
