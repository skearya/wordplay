<script lang="ts">
	import type { Props } from '$lib/context';
	import GreenSettings from '$lib/icons/GreenSettings.svelte';
	import Logo from '$lib/icons/Logo.svelte';

	const { ctx, sendMsg }: Omit<Props<never>, 'initial'> = $props();

	const navOnFocus = (nav: HTMLElement) => {
		if (ctx.state.kind === 'lobby') return;
		nav.classList.remove('opacity-0');
	};

	const navOnBlur = (nav: HTMLElement) => {
		if (ctx.state.kind === 'lobby') return;
		nav.classList.add('opacity-0');
	};
</script>

<nav
	class={[
		'flex items-center justify-between px-5 py-4',
		ctx.state.kind !== 'lobby'
			? 'bg-background/90 fixed left-0 top-0 z-40 w-full opacity-0 transition-opacity'
			: 'opacity-100'
	]}
	onmouseover={(e) => navOnFocus(e.currentTarget)}
	onfocus={(e) => navOnFocus(e.currentTarget)}
	onmouseout={(e) => navOnBlur(e.currentTarget)}
	onblur={(e) => navOnBlur(e.currentTarget)}
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
</nav>
