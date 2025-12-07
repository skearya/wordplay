<script lang="ts">
	import type { PageProps } from './$types';
	import type { Attachment } from 'svelte/attachments';
	import Matter from 'matter-js';
	import { onMount } from 'svelte';
	import Socket from '$lib/components/Socket.svelte';
	import { createLetterCanvas } from '$lib/letters';

	const { params }: PageProps = $props();

	let ready = $state(false);
	let username = $state('');

	const setupCanvas: Attachment<HTMLCanvasElement> = (canvas) => {
		const cleanupCanvas = createLetterCanvas(canvas, 'dark', 0.05, (width, height) =>
			'wordplaybyskeary.me'
				.split('')
				.toReversed()
				.map((letter) => ({
					letter,
					x: Math.random() * width,
					y: Math.random() * height,
					angle: Math.random() - 0.5 * 2.0
				}))
		);

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
				onsubmit={(e) => {
					e.preventDefault();

					if (username.length > 0 && username.length <= 32) {
						ready = true;
					}
				}}
			>
				<label>Username <input bind:value={username} /></label>
				<button>Join</button>
			</form>
		</main>
	{/if}

	{#snippet failed(_error, _reset)}
		<button onclick={() => window.location.reload()}>oops! try again</button>
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
