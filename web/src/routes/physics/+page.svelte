<script lang="ts">
	import type { Attachment } from 'svelte/attachments';
	import type { PageProps } from './$types';
	import Matter from 'matter-js';
	import { setupCanvas } from '$lib/utils';

	const { Engine, Bodies, Composite, Composites, Mouse, MouseConstraint, Vector, Body } = Matter;

	const { data }: PageProps = $props();

	const background: Attachment<HTMLCanvasElement> = (canvas) => {
		const engine = Engine.create({ gravity: { y: 0 } });

		const letterWidth = 64;
		const letterHeight = 64;
		const letterData: Map<Matter.Body, string> = new Map();

		const stack = Composites.pyramid(
			canvas.clientWidth / 2 - (18 * 64) / 2,
			300,
			18,
			3,
			0,
			0,
			(x: number, y: number, _column: any, _row: any, _lastBody: any, i: any) => {
				const letter = Bodies.rectangle(x, y, letterWidth, letterHeight);
				letterData.set(letter, String.fromCharCode('A'.charCodeAt(0) + (i % 26)));

				return letter;
			}
		);

		Composite.add(engine.world, stack);

		const wallThickness = 100;
		const wallOptions = { isStatic: true };

		const originalWidth = canvas.clientWidth;
		const originalHeight = canvas.clientHeight;

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

		const mouseConstraint = MouseConstraint.create(engine, {
			mouse: mouse,
			constraint: {
				stiffness: 1.0,
				render: { visible: false }
			}
		});

		Composite.add(engine.world, mouseConstraint);

		const canvasResizeObserver = new ResizeObserver(() => {
			Body.setPosition(leftWall, Vector.create(0 - wallThickness / 2, canvas.clientHeight / 2));
			Body.setPosition(
				rightWall,
				Vector.create(canvas.clientWidth + wallThickness / 2, canvas.clientHeight / 2)
			);
			Body.setPosition(topWall, Vector.create(canvas.clientWidth / 2, 0 - wallThickness / 2));
			Body.setPosition(
				bottomWall,
				Vector.create(canvas.clientWidth / 2, canvas.clientHeight + wallThickness / 2)
			);
		});

		canvasResizeObserver.observe(canvas);

		// const debugRender = Render.create({
		// 	canvas,
		// 	engine,
		// 	options: {
		// 		width: canvas.clientWidth,
		// 		height: canvas.clientHeight
		// 	}
		// });

		// Render.run(debugRender);

		// const runner = Runner.create();
		// Runner.run(runner, engine);

		mouse.pixelRatio = window.devicePixelRatio || 1;

		const cleanupCanvas = setupCanvas(canvas, (ctx, dt) => {
			Engine.update(engine, Math.min(dt, 60));

			ctx.clearRect(0, 0, canvas.width, canvas.height);

			ctx.lineWidth = 1;
			ctx.textBaseline = 'middle';

			for (const [box, letter] of letterData) {
				ctx.save();

				ctx.translate(box.position.x, box.position.y);
				ctx.rotate(box.angle);

				ctx.strokeRect(-letterWidth / 2, -letterHeight / 2, letterWidth, letterHeight);

				ctx.font = '42px sans-serif';
				ctx.textAlign = 'center';
				ctx.fillText(letter, 0, 0);

				ctx.font = '12px sans-serif';
				ctx.textAlign = 'right';
				ctx.fillText(`${letter.charCodeAt(0)}`, letterWidth / 2 - 4, letterHeight / 2 - 10);

				ctx.restore();
			}
		});

		return () => {
			cleanupCanvas();
			canvasResizeObserver.disconnect();
		};
	};
</script>

<canvas {@attach background} class="h-screen w-screen"></canvas>
