<script lang="ts">
	import type { RoomInfoResponse } from '@bindings/RoomInfoResponse';
	import type { Attachment } from 'svelte/attachments';
	import { onMount } from 'svelte';
	import { fade } from 'svelte/transition';
	import AnimatedDoubleRightArrow from '$lib/icons/AnimatedDoubleRightArrow.svelte';
	import LogoFilled from '$lib/icons/LogoFilled.svelte';
	import { createLetterCanvas, darkStyle } from '$lib/letters';
	import Tag from '$lib/ui/Tag.svelte';
	import { camelCaseToWords, getRandomRange, serverURL } from '$lib/utils';

	let {
		room,
		connection,
		onEnter
	}: {
		room: string;
		connection: 'awaiting' | 'connecting' | 'connected';
		onEnter: (username: string) => void;
	} = $props();

	let username = $state('');
	let message = $state('');

	onMount(() => {
		const prevUsername = localStorage.getItem('username');

		if (prevUsername) {
			username = prevUsername;
		}

		fetchRoomInfo(room)
			.then(({ room }) => {
				if (!room) {
					message = "This room doesn't exist yet, create it!";
					return;
				}

				message = `This room is currently in ${
					room.state.kind === 'game'
						? `a game of ${camelCaseToWords(room.state.variant.kind)}`
						: `lobby for a game of ${camelCaseToWords(room.settings.game)}`
				}.`;
			})
			.catch(() => {
				message += 'Something went wrong loading room data...';
			});
	});

	async function fetchRoomInfo(room: string): Promise<RoomInfoResponse> {
		return (await (await fetch(serverURL(`/info/${room}`))).json()) as RoomInfoResponse;
	}

	const setupCanvas: Attachment<HTMLCanvasElement> = (canvas) => {
		const cleanupCanvas = createLetterCanvas(canvas, {
			style: darkStyle,
			gravity: 0.05,
			initLetters: (width, height) =>
				'wordplaybyskeary.me'
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

		canvas.animate({ opacity: '100%' }, { fill: 'forwards', duration: 150, easing: 'ease-in' });

		return () => cleanupCanvas();
	};
</script>

<main out:fade>
	<div class="absolute top-0 left-0 -z-10 h-screen w-screen overflow-hidden">
		<canvas {@attach setupCanvas} class="background background-scroll h-full w-full opacity-0"
		></canvas>
	</div>
	<form
		style="background: radial-gradient(at top left, var(--color-background) 0%, color-mix(in srgb, var(--color-pastel-pink) 10%, transparent) 100%), var(--color-background); border-image: conic-gradient(from -112deg, rgba(121, 120, 150, 0.5), rgba(203, 201, 252, 1)) 1;"
		class="absolute top-28 left-1/2 flex h-64 w-full max-w-xl -translate-x-1/2 flex-col border"
		onsubmit={(e) => {
			e.preventDefault();

			if (1 <= username.length && username.length <= 32) {
				localStorage.setItem('username', username);
				onEnter(username);
			}
		}}
	>
		<Tag class="bg-pink">Joining Game <code>'{room}'</code></Tag>
		<div class="flex flex-1 items-center">
			<div class="flex flex-1 flex-col justify-center p-4">
				<label for="username" class="mb-1.5 block text-sm font-medium">Username</label>
				<input
					type="text"
					id="username"
					name="username"
					class="border border-green bg-background px-3 py-2.5 text-sm shadow-xs focus:border-green"
					placeholder="wordplayer"
					required
					minlength="1"
					maxlength="32"
					disabled={connection !== 'awaiting'}
					bind:value={username}
				/>
			</div>
			<button
				style="clip-path: polygon(35% 0, 100% 0, 100% 100%, 0% 100%);"
				class="relative h-full bg-pink pr-6 pl-12"
			>
				<p class="bg-pink font-serif text-2xl text-background">Enter!</p>
				<AnimatedDoubleRightArrow class="absolute right-2 bottom-2.5" />
			</button>
		</div>
		<div class="overflow-hidden border-t border-pastel-pink/20 bg-pastel-pink/5 font-mono">
			<div class="marquee -mb-1 py-0.5">
				{#each { length: 2 }}
					<p class="pl-36 text-sm text-nowrap text-white/75">
						{message}
					</p>
				{/each}
			</div>
		</div>
		<LogoFilled
			class="absolute bottom-4 left-4 -translate-x-1/2 translate-y-1/2 scale-125 rotate-3"
		/>
	</form>
</main>

<style>
	.background {
		animation:
			400ms cubic-bezier(0.33, 1, 0.68, 1) background-fade-in,
			120s linear infinite background-scroll-keyframes;
		background-image:
			repeating-linear-gradient(
				90deg,
				transparent,
				transparent 30px,
				color-mix(in srgb, var(--color-pastel-pink) 6%, transparent) 30px,
				color-mix(in srgb, var(--color-pastel-pink) 6%, transparent) 31px
			),
			repeating-linear-gradient(
				150deg,
				transparent,
				transparent 35px,
				color-mix(in srgb, var(--color-pastel-pink) 4%, transparent) 35px,
				color-mix(in srgb, var(--color-pastel-pink) 4%, transparent) 36px
			);
	}

	@keyframes background-fade-in {
		0% {
			scale: 1.1;
		}
		100% {
			scale: 1;
		}
	}
</style>
