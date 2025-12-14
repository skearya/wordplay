<script lang="ts">
	import type { PageProps } from './$types';
	import type { Attachment } from 'svelte/attachments';
	import Socket from '$lib/components/Socket.svelte';
	import FilledDoubleRightArrow from '$lib/icons/FilledDoubleRightArrow.svelte';
	import HeartIcon from '$lib/icons/HeartIcon.svelte';
	import LogoFilled from '$lib/icons/LogoFilled.svelte';
	import { createLetterCanvas } from '$lib/letters';
	import { getRandomRange } from '$lib/utils';

	const { params }: PageProps = $props();

	let ready = $state(false);
	let username = $state('');

	const setupCanvas: Attachment<HTMLCanvasElement> = (canvas) => {
		const cleanupCanvas = createLetterCanvas(canvas, {
			style: 'dark',
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

		return () => cleanupCanvas();
	};
</script>

<svelte:boundary>
	{#if ready}
		<Socket room={params.room} {username} />
	{:else}
		<main>
			<canvas
				{@attach setupCanvas}
				class="background background-scroll absolute left-0 top-0 -z-10 h-full w-full"
			></canvas>
			<form
				style="background: radial-gradient(at top left, var(--color-background) 0%, rgba(233, 184, 255, 0.1) 100%), var(--color-background); border-image: conic-gradient(from -112deg, rgba(121, 120, 150, 0.5), rgba(203, 201, 252, 1)) 1;"
				class="absolute left-1/2 top-28 flex h-64 max-w-xl -translate-x-1/2 flex-col border"
				onsubmit={(e) => {
					e.preventDefault();

					if (1 <= username.length && username.length <= 32) {
						ready = true;
					}
				}}
			>
				<div
					class="bg-pink absolute left-0 top-0 w-min text-nowrap rounded-br-2xl px-3 py-1 text-black"
				>
					<p>Joining Game <code>'{params.room}'</code></p>
				</div>
				<div class="flex flex-1 items-center">
					<div class="flex flex-1 flex-col justify-center p-4">
						<label for="username" class="mb-1.5 block text-sm font-medium">Username</label>
						<input
							type="text"
							id="username"
							name="username"
							class="bg-background border-green focus:ring-pasteborder-green focus:border-green shadow-xs border px-3 py-2.5 text-sm"
							placeholder="wordplayer"
							required
							minlength="1"
							maxlength="32"
							bind:value={username}
						/>
					</div>
					<button
						style="clip-path: polygon(35% 0, 100% 0, 100% 100%, 0% 100%);"
						class="bg-pink relative h-full pl-12 pr-6"
					>
						<p class="text-background bg-pink font-serif text-2xl">Enter!</p>
						<FilledDoubleRightArrow class="absolute bottom-2.5 right-2" />
					</button>
				</div>
				<div class="border-pastel-pink/20 bg-pastel-pink/5 overflow-hidden border-t font-mono">
					<div class="marquee -mb-1 py-0.5">
						{#each { length: 2 }}
							<p class="text-nowrap pl-32 text-sm text-white/75">
								The room is currently in a game of Word Bomb with 4 players for 3:39 minutes with 95
								words used. Consider supporting Wordplay! <HeartIcon
									width={19.5}
									height={16.5}
									class="inline-block"
								/>
							</p>
						{/each}
					</div>
				</div>
				<LogoFilled
					class="absolute bottom-4 left-4 -translate-x-1/2 translate-y-1/2 rotate-3 scale-125"
				/>
			</form>
		</main>
	{/if}

	{#snippet failed(_error, _reset)}
		<button onclick={() => window.location.reload()}>oops, something broke. try again?</button>
	{/snippet}
</svelte:boundary>

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
				rgba(233, 184, 255, 0.06) 30px,
				rgba(233, 184, 255, 0.06) 31px
			),
			repeating-linear-gradient(
				150deg,
				transparent,
				transparent 35px,
				rgba(233, 184, 255, 0.04) 35px,
				rgba(233, 184, 255, 0.04) 36px
			);
	}

	@keyframes background-fade-in {
		0% {
			scale: 0.8;
		}
		100% {
			scale: 1;
		}
	}
</style>
