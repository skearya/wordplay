<script lang="ts">
	import type { PageProps } from './$types';
	import type { Attachment } from 'svelte/attachments';
	import Matter from 'matter-js';
	import { setupCanvas } from '$lib/utils';
	import homepageNoiseImage from '$lib/assets/homepage-noise.png';
	import gridSvg from '$lib/assets/grid.svg';
	import bombSvg from '$lib/assets/bomb.svg';
	import settingsSvg from '$lib/assets/settings.svg';
	import githubSvg from '$lib/assets/github.svg';
	import meSvg from '$lib/assets/me.svg';
	import searchSvg from '$lib/assets/search.svg';

	const { Engine, Bodies, Composite, Composites, Mouse, MouseConstraint, Vector, Body, Events } =
		Matter;

	const { data }: PageProps = $props();

	const backgroundCanvas: Attachment<HTMLCanvasElement> = (canvas) => {
		const originalWidth = canvas.clientWidth;
		const originalHeight = canvas.clientHeight;

		const engine = Engine.create({ gravity: { y: 0.01 } });

		const letterWidth = 64;
		const letterHeight = 64;
		const letters = new Map(
			['wordplay', 'byskeary.me', 'abcdefghijkl'].toReversed().flatMap((line, lineIndex) =>
				line.split('').map((letter, letterIndex) => {
					const lineWidth = line.length * 64;
					const start = originalWidth / 2 - lineWidth / 2;

					const x = start + letterIndex * 64 + letterWidth / 2;
					const y = originalHeight - lineIndex * 64 - letterHeight / 2;

					const letterBody = Bodies.rectangle(x, y, letterWidth, letterHeight);

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

		const mouse = Mouse.create(canvas);

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

		mouse.pixelRatio = window.devicePixelRatio || 1;

		const mouseConstraint = MouseConstraint.create(engine, {
			mouse: mouse,
			constraint: {
				stiffness: 1.0,
				render: { visible: false }
			}
		});

		Composite.add(engine.world, mouseConstraint);

		const cleanupCanvas = setupCanvas(canvas, (ctx, dt) => {
			const canvasWidth = canvas.clientWidth;
			const canvasHeight = canvas.clientHeight;

			Body.setPosition(leftWall, Vector.create(0 - wallThickness / 2, canvasHeight / 2));
			Body.setPosition(rightWall, Vector.create(canvasWidth + wallThickness / 2, canvasHeight / 2));
			Body.setPosition(topWall, Vector.create(canvasWidth / 2, 0 - wallThickness / 2));
			Body.setPosition(
				bottomWall,
				Vector.create(canvasWidth / 2, canvasHeight + wallThickness / 2),
				// @ts-expect-error Sets velocity. Not typed for some reason.
				true
			);

			for (const [body] of letters) {
				const clampedX = Math.max(Math.min(body.position.x, canvasWidth), 0);
				const clampedY = Math.max(Math.min(body.position.y, canvasHeight), 0);

				if (clampedX !== body.position.x || clampedY !== body.position.y) {
					Body.setPosition(body, Vector.create(clampedX, clampedY));
					Body.setVelocity(body, Vector.create(0, 0));
				}
			}

			Engine.update(engine, Math.min(dt, 60));

			ctx.clearRect(0, 0, canvas.width, canvas.height);

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
	};
</script>

<header
	style={`background-image: url("${homepageNoiseImage}");`}
	class="intro-background relative h-screen w-screen bg-cover"
>
	<canvas {@attach backgroundCanvas} class="h-full w-full"></canvas>
	<h1
		style="font-family: 'PP Editorial New';"
		class="intro-background-text text-background pointer-events-none absolute bottom-0 left-0 p-4 text-8xl opacity-0"
	>
		Wordplay
	</h1>
</header>

<section
	style={`background-image: url("${gridSvg}");`}
	class="background-scroll inset-shadow-[0_20px_20px] inset-shadow-black bg-background flex min-h-screen w-full gap-2.5 bg-repeat p-4"
>
	<div style="font-family: 'Mona Sans';" class="text-background w-[325px] space-y-2.5">
		<button class="block w-full bg-[#FEC5BB] py-7 text-2xl font-medium">Join room</button>
		<button class="block w-full bg-[#FAE1DD] py-7 text-2xl font-medium">Create room</button>
		<button class="block w-full bg-[#E8E8E4] py-7 text-2xl font-medium">Singleplayer</button>
		<div class="mt-6 flex items-center gap-x-2.5 p-2.5">
			<img src={settingsSvg} alt="Settings" />
			<img src={githubSvg} alt="Github" />
			<img src={meSvg} alt="Skeary" width="36px" height="36px" class="mt-[2px]" />
		</div>
	</div>
	<div class="flex-1 space-y-2.5 p-2.5">
		<div style="font-family: 'PP Editorial New';" class="flex items-center justify-between">
			<h1 class="text-2xl">Public rooms</h1>
			<div class="flex min-w-[185px] items-center justify-between text-[#B0B0B0]">
				<h1 class="text-xl">Search...</h1>
				<img src={searchSvg} alt="Search" />
			</div>
		</div>
		<div class="grid grid-cols-3 gap-2.5">
			{#each { length: 12 }}
				<a
					href="/"
					style="font-family: 'Mona Sans';"
					class="relative border border-[#545A4F] bg-[#1D1F1E] p-2.5"
				>
					<h1 class="mb-10 text-lg">Stupid Room Name</h1>
					<div class="flex -space-x-2">
						{#each { length: 3 }, i}
							<img
								src={`https://avatar.vercel.sh/${i}`}
								width="38px"
								height="38px"
								alt="avatar"
								class="border-background aspect-square size-[38px] rounded-full border-2"
							/>
						{/each}
						<div
							class="bg-background flex aspect-square size-[38px] items-center justify-center rounded-full"
						>
							+21
						</div>
					</div>
					<img src={bombSvg} alt="Word bomb logo" class="absolute bottom-2.5 right-2.5" />
				</a>
			{/each}
		</div>
	</div>
</section>

<style>
	@keyframes intro-background-keyframes {
		from {
			height: 100vh;
		}
		to {
			height: 50vh;
		}
	}

	.intro-background {
		animation: 1000ms cubic-bezier(0.87, 0, 0.13, 1) 500ms both intro-background-keyframes;
	}

	@keyframes intro-background-text-keyframes {
		from {
			opacity: 0%;
		}
		to {
			opacity: 100%;
		}
	}

	.intro-background-text {
		animation: 750ms ease-in 1500ms both intro-background-text-keyframes;
	}

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
