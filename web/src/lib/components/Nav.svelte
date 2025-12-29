<script lang="ts">
	import type { Props } from '$lib/context';
	import GreenSettings from '$lib/icons/GreenSettings.svelte';
	import Logo from '$lib/icons/Logo.svelte';

	const { ctx }: Omit<Props<never>, 'initial' | 'sendMsg'> = $props();

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
	class={ctx.state.kind !== 'lobby' ? 'fixed left-0 top-0 z-40 w-full' : null}
>
	<div
		bind:this={innerNavElement}
		class={[
			'flex items-center justify-between px-5 py-4 transition-transform',
			ctx.state.kind === 'lobby' ? 'translate-y-0' : 'bg-background/90 -translate-y-full'
		]}
	>
		<Logo />
		<div class="flex items-center gap-x-4">
			<div class="border-green flex -space-x-4">
				{#each Object.entries(ctx.clients).slice(0, 4) as [uuid, client]}
					<img
						src={`https://avatar.vercel.sh/${client!.username}`}
						alt={client!.username}
						title={`${client!.username} (${uuid})`}
						width="120"
						height="120"
						class="size-10 rounded-full border border-black"
					/>
				{/each}
				{#if Object.keys(ctx.clients).length > 4}
					<div class="border-green size-10 content-center rounded-full border bg-black text-center">
						+{Object.keys(ctx.clients).length - 4}
					</div>
				{/if}
			</div>
			<GreenSettings />
		</div>
	</div>
</nav>
