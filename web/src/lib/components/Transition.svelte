<script lang="ts">
	import type { Props } from '$lib/context';
	import { tick } from 'svelte';
	import LogoFilled from '$lib/icons/LogoFilled.svelte';
	import { transitionState } from '$lib/stores/transition.svelte';

	const { ctx = $bindable() }: Omit<Props<never>, 'initial' | 'sendMsg'> = $props();

	let containerElement = $state<HTMLElement>();
	let logoElement = $state<HTMLElement>();
	let backgroundElement = $state<HTMLElement>();

	const easing = `linear(0, 0.015, 0.039 2.6%, 0.119 5.6%, 0.545 17.7%, 0.644, 0.728 23.8%, 0.803, 0.866, 0.917, 0.957, 0.986 40.4%, 1.007 44.1%, 1.021 48.2%, 1.028 52.8%, 1.027 60.6%, 1.005 84.7%, 1)`;

	$effect(() => {
		if (transitionState.kind === 'transitioning') {
			if (!containerElement || !logoElement || !backgroundElement) return;

			const { update } = transitionState;

			logoElement.animate(
				{
					scale: '100%',
					translate: '-50% -50%'
				},
				{
					duration: 1200,
					fill: 'forwards',
					easing
				}
			);

			backgroundElement
				.animate(
					{
						clipPath: 'circle(70.7% at 50% 50%)',
						opacity: '100%'
					},
					{
						delay: 800,
						duration: 800,
						fill: 'forwards',
						easing: 'ease-out'
					}
				)
				.addEventListener('finish', () => (ctx.state = update));

			containerElement
				.animate(
					{
						clipPath: ['circle(70.7% at 50% 50%)', 'circle(0.0% at 50% 50%)']
					},
					{
						delay: 1600,
						duration: 400,
						fill: 'forwards',
						easing: 'ease-in'
					}
				)
				.addEventListener('finish', () => (transitionState.kind = 'idle'));
		}
	});
</script>

{#if transitionState.kind === 'transitioning'}
	<div
		bind:this={containerElement}
		class="absolute top-0 left-0 z-50 h-screen w-screen overflow-hidden"
	>
		<div
			bind:this={backgroundElement}
			class="background absolute top-0 left-0 h-screen w-screen opacity-0"
			style="clip-path: circle(0% at 50% 50%);"
		></div>
		<div
			bind:this={logoElement}
			class="absolute top-1/2 left-1/2 -translate-x-1/2 translate-y-[60vh] scale-50"
		>
			<LogoFilled width={96 * 2} height={61 * 2} />
		</div>
	</div>
{/if}

<style>
	.background {
		background-color: var(--color-background);
		background-image:
			radial-gradient(at top left, var(--color-background) 0%, rgba(233, 184, 255, 0.1) 100%),
			repeating-linear-gradient(
				90deg,
				transparent,
				transparent 30px,
				rgba(233, 184, 255, 0.12) 30px,
				rgba(233, 184, 255, 0.12) 31px
			),
			repeating-linear-gradient(
				150deg,
				transparent,
				transparent 35px,
				rgba(233, 184, 255, 0.09) 35px,
				rgba(233, 184, 255, 0.09) 36px
			);
	}
</style>
