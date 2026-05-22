<script lang="ts">
	import type { RoomInfo } from '@bindings/RoomInfo';
	import type { SvelteHTMLElements } from 'svelte/elements';
	import Bomb from '$lib/icons/Bomb.svelte';
	import StaticAvatar from '$lib/ui/StaticAvatar.svelte';
	import { unreachable } from '$lib/utils';

	let {
		name,
		info,
		class: className,
		...rest
	}: {
		name: string;
		info?: RoomInfo;
	} & (SvelteHTMLElements['a'] & SvelteHTMLElements['div']) = $props();

	const containerClass = [
		'relative flex h-32 flex-col justify-between border border-faded-green p-2.5',
		info && 'bg-background',
		className
	];
</script>

{#if info}
	<a href={`/game/${name}`} class={containerClass} {...rest}>
		{@render content(info)}
	</a>
{:else}
	<div class={containerClass} {...rest}>
		{@render content()}
	</div>
{/if}

{#snippet content(info?: RoomInfo)}
	<p class="text-lg">{name}</p>
	<div class="flex -space-x-2">
		{#if info}
			{#each info.clients as client}
				<StaticAvatar
					size="sm"
					username={client.username}
					avatarUrl={client.avatarUrl ?? undefined}
				/>
			{/each}
		{:else}
			<div class="size-[38px] rounded-full border border-pastel-red"></div>
			<div class="size-[38px] rounded-full border border-pastel-green"></div>
			<div class="size-[38px] rounded-full border border-pastel-blue"></div>
		{/if}
	</div>
	{#if info}
		{#if info.settings.game === 'wordBomb'}
			<Bomb class="absolute right-3 bottom-3" />
		{:else if info.settings.game === 'anagrams'}
			{unreachable(info.settings.game)}
		{:else if info.settings.game satisfies never}
			{unreachable(info.settings.game)}
		{/if}
	{:else}
		<Bomb class="absolute right-3 bottom-3" />
	{/if}
{/snippet}
