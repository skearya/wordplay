<script lang="ts">
	import type { RoomInfo } from '@bindings/RoomInfo';
	import type { SvelteHTMLElements } from 'svelte/elements';
	import Bomb from '$lib/icons/Bomb.svelte';
	import Avatar from '$lib/ui/Avatar.svelte';
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

	const containerClass = () => [
		'relative flex h-32 flex-col justify-between bg-background border border-faded-green p-2.5',
		className
	];
</script>

{#if info}
	<a href={`/game/${name}`} class={containerClass()} {...rest}>
		{@render content(info)}
	</a>
{:else}
	<div class={containerClass()} {...rest}>
		{@render content()}
	</div>
{/if}

{#snippet content(info?: RoomInfo)}
	<p class="text-lg">{name}</p>
	<div class="flex -space-x-2">
		{#if info}
			{#each info.clients as client}
				<Avatar size="sm" username={client.username} avatarUrl={client.avatarUrl ?? undefined} />
			{/each}
		{:else}
			<div class="size-9.5 rounded-full border border-pastel-red"></div>
			<div class="size-9.5 rounded-full border border-pastel-green"></div>
			<div class="size-9.5 rounded-full border border-pastel-blue"></div>
		{/if}
	</div>
	{#if info === undefined || info.settings.game === 'wordBomb'}
		<Bomb class="absolute right-3 bottom-3" />
	{:else if info.settings.game === 'anagrams'}
		{unreachable(info.settings.game)}
	{:else if info.settings.game satisfies never}
		{unreachable(info.settings.game)}
	{/if}
{/snippet}
