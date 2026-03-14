import type { IChamferableBodyDefinition } from 'matter-js';
import Matter from 'matter-js';
import { setupCanvas } from './canvas';
import { lerp } from './utils';

const { Engine, Bodies, Composite, Mouse, MouseConstraint, Vector, Body } = Matter;

export function createLetterCanvas(
	canvas: HTMLCanvasElement,
	{
		style,
		gravity,
		initLetters,
		bottomPosition = undefined,
		initialForce = false
	}: {
		style: 'light' | 'dark';
		gravity: number;
		initLetters: (
			width: number,
			height: number
		) => (IChamferableBodyDefinition & { x: number; y: number; letter: string })[];
		bottomPosition?: () => number;
		initialForce?: boolean;
	}
): () => void {
	const light = style === 'light';
	const originalWidth = canvas.clientWidth;
	const originalHeight = canvas.clientHeight;

	const engine = Engine.create({ gravity: { y: gravity } });

	const letterWidth = 64;
	const letterHeight = 64;
	const letters = new Map<Matter.Body, string>();

	for (const { x, y, letter, ...rest } of initLetters(originalWidth, originalHeight)) {
		const letterBody = Bodies.rectangle(x, y, letterWidth, letterHeight, rest);

		letters.set(letterBody, letter);
	}

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

	const cleanupCanvas = setupCanvas(canvas, (ctx, dt, width, height, time) => {
		const bottom = lerp(height, height / 2, bottomPosition?.() ?? 0);

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
			if (initialForce && time < 50) {
				Body.applyForce(body, body.position, Vector.create(0, -0.01));
				Body.setAngularVelocity(body, (Math.random() - 0.5) * 0.03);
			}

			const clampedX = Math.max(Math.min(body.position.x, width), 0);
			const clampedY = Math.max(Math.min(body.position.y, height), 0);

			if (clampedX !== body.position.x || clampedY !== body.position.y) {
				Body.setPosition(body, Vector.create(clampedX, clampedY));
				Body.setVelocity(body, Vector.create(0, 0));
			}
		}

		Engine.update(engine, Math.min(dt, 60));

		ctx.clearRect(0, 0, width, height);

		ctx.lineWidth = 1;
		ctx.textBaseline = 'middle';

		for (const [box, letter] of letters) {
			ctx.save();

			ctx.translate(box.position.x, box.position.y);
			ctx.rotate(box.angle);

			ctx.fillStyle = `rgba(${light ? '255, 255, 255' : '71, 93, 80'}, ${Math.min(Vector.magnitude(box.velocity) / 2, 0.8)})`;
			ctx.fillRect(-letterWidth / 2, -letterHeight / 2, letterWidth, letterHeight);

			ctx.strokeStyle = light ? 'black' : 'rgba(139, 166, 152, 0.5)';
			ctx.strokeRect(-letterWidth / 2, -letterHeight / 2, letterWidth, letterHeight);

			ctx.font = '300 40px sans-serif';
			ctx.textAlign = 'center';
			ctx.fillStyle = light ? 'black' : 'rgba(139, 166, 152, 1)';
			ctx.fillText(letter, 0, 0);

			ctx.font = '500 12px sans-serif';
			ctx.textAlign = 'right';
			ctx.fillText(`${letter.charCodeAt(0)}`, letterWidth / 2 - 4, letterHeight / 2 - 10);

			ctx.restore();
		}
	});

	return () => cleanupCanvas();
}
