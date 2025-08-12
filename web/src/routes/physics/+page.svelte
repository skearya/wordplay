<script lang="ts">
	import type { Attachment } from 'svelte/attachments';
	import type { PageProps } from './$types';
	import Matter from 'matter-js';

	const { Engine, Render, Runner, Bodies, Composite, Mouse, MouseConstraint } = Matter;

	const { data }: PageProps = $props();

	function setupCanvas(canvas: HTMLCanvasElement): CanvasRenderingContext2D {
		const dpr = window.devicePixelRatio || 1;

		canvas.width = canvas.clientWidth * dpr;
		canvas.height = canvas.clientHeight * dpr;

		const ctx = canvas.getContext('2d');

		if (!ctx) {
			throw new Error("getContext('2d') failed");
		}

		ctx.scale(dpr, dpr);

		return ctx;
	}

	const background: Attachment<HTMLCanvasElement> = (canvas) => {
		// const engine = Engine.create();

		// const boxA = Bodies.rectangle(400, 200, 80, 80);
		// const boxB = Bodies.rectangle(450, 50, 80, 80);
		// const ground = Bodies.rectangle(400, 610, 810, 60, { isStatic: true });

		// Composite.add(engine.world, [boxA, boxB, ground]);

		// const render = Render.create({
		// 	element: document.body,
		// 	engine: engine
		// });

		// const runner = Runner.create();

		// Runner.run(runner, engine);
		// Render.run(render);

		// const mouse = Mouse.create(render.canvas);
		// const mouseConstraint = MouseConstraint.create(engine, {
		// 	mouse: mouse,
		// 	constraint: {
		// 		stiffness: 0.2
		// 	}
		// });

		// Composite.add(engine.world, mouseConstraint);

		const ctx = setupCanvas(canvas);

		const onResize = () => {
			const dpr = window.devicePixelRatio || 1;

			canvas.width = canvas.clientWidth * dpr;
			canvas.height = canvas.clientHeight * dpr;

			ctx.scale(dpr, dpr);
		};

		window.addEventListener('resize', onResize);

		let rafId: number | undefined;
		let lastTime = performance.now();

		const onFrame = (time: number) => {
			console.log(canvas.width, canvas.height);
			ctx.clearRect(0, 0, canvas.width, canvas.height);

			ctx.lineWidth = 2;
			ctx.beginPath();
			ctx.roundRect(0, 0, 64, 64, 4);
			ctx.stroke();

			ctx.font = '42px sans-serif';
			ctx.textAlign = 'center';
			ctx.textBaseline = 'middle';
			ctx.fillText('A', 32, 36);

			ctx.font = '12px sans-serif';
			ctx.textAlign = 'right';
			ctx.fillText('12', 60, 56);

			lastTime = time;
			rafId = requestAnimationFrame(onFrame);
		};

		rafId = requestAnimationFrame(onFrame);

		return () => {
			if (rafId) {
				cancelAnimationFrame(rafId);
			}

			window.removeEventListener('resize', onResize);
		};
	};
</script>

<canvas {@attach background} class="h-screen w-screen"></canvas>
