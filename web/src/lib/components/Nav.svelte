<script lang="ts">
	import type { Props } from '$lib/context';
	import GreenSettings from '$lib/icons/GreenSettings.svelte';
	import Logo from '$lib/icons/Logo.svelte';
	import Avatar from './Avatar.svelte';

	let { ctx = $bindable() }: Omit<Props<never>, 'state' | 'sendMsg'> = $props();

	let innerNavElement: HTMLElement;

	const navOnFocus = () => {
		if (ctx.state.kind === 'lobby') return;
		innerNavElement.classList.remove('-translate-y-full');
	};

	const navOnBlur = () => {
		if (ctx.state.kind === 'lobby') return;
		innerNavElement.classList.add('-translate-y-full');
	};
</script>

<nav
	onmouseover={() => navOnFocus()}
	onfocus={() => navOnFocus()}
	onmouseout={() => navOnBlur()}
	onblur={() => navOnBlur()}
	class={ctx.state.kind !== 'lobby' ? 'fixed top-0 left-0 z-40 w-full' : null}
>
	<div
		bind:this={innerNavElement}
		class={[
			'flex items-center justify-between px-5 py-4 transition-transform',
			ctx.state.kind === 'lobby' ? 'translate-y-0' : '-translate-y-full bg-background'
		]}
	>
		<Logo />
		<div class="flex items-center gap-x-4">
			<div class="flex -space-x-4 border-green">
				{#each Object.keys(ctx.clients).slice(0, 4) as uuid (uuid)}
					<Avatar {ctx} {uuid} size="sm" class="border border-black" />
				{/each}
				{#if Object.keys(ctx.clients).length > 4}
					<div class="size-10 content-center rounded-full border border-green bg-black text-center">
						+{Object.keys(ctx.clients).length - 4}
					</div>
				{/if}
			</div>
			<GreenSettings />
		</div>
	</div>
</nav>
