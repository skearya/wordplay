<script lang="ts">
	import type { LobbyState } from '@bindings/LobbyState';
	import type { Props } from '$lib/context';
	import { fly, slide } from 'svelte/transition';
	import Avatar from '$lib/components/Avatar.svelte';
	import Countdown from '$lib/components/lobby/Countdown.svelte';
	import DoubleRightArrow from '$lib/icons/DoubleRightArrow.svelte';
	import DownArrow from '$lib/icons/DownArrow.svelte';
	import WordBomb from '$lib/icons/WordBomb.svelte';
	import Button from '$lib/ui/Button.svelte';
	import Tag from '$lib/ui/Tag.svelte';
	import { camelCaseToWords } from '$lib/utils';
	import { openSettings } from '../Nav.svelte';

	let { ctx = $bindable(), state: lobby = $bindable(), sendMsg }: Props<LobbyState> = $props();
</script>

<section class="relative flex h-full flex-col overflow-hidden border border-green bg-green/20">
	<Tag class="bg-green text-foreground">
		{#if lobby.timerStart}
			<Countdown timerStart={lobby.timerStart} />
		{:else}
			Ready Players
		{/if}
	</Tag>
	<div class="pointer-events-none absolute -right-6 -bottom-6 opacity-50">
		<WordBomb class="aspect-[901/916] w-[calc(min(45vw,60vh))] mix-blend-color-dodge" />
	</div>
	<div class="absolute top-3 right-3 text-bright-green">
		<p>{ctx.settings.size - lobby.ready.length} slots left</p>
	</div>
	<div class="flex flex-1 items-center justify-center gap-x-20 px-12">
		<div class="relative size-[304px]">
			<div
				class={[
					'absolute top-1/2 left-1/2 flex -translate-x-1/2 -translate-y-1/2 flex-col items-center gap-y-4 text-[#B1C1AE] transition-opacity',
					lobby.ready.length === 0 ? 'opacity-100 delay-400' : 'opacity-0'
				]}
			>
				<div
					class="size-24 content-center rounded-full border border-dashed text-center opacity-80"
				>
					?
				</div>
				<p class="loading animate-pulse text-nowrap">Waiting for someone to ready</p>
			</div>
			{#each lobby.ready as uuid, i (uuid)}
				{@const angleBetween = (2 * Math.PI) / lobby.ready.length}
				{@const angle = i * angleBetween + Math.PI / 2}
				{@const dist = 80 + Math.log2(lobby.ready.length) * 20}
				{@const x = Math.cos(angle) * dist}
				{@const y = Math.sin(angle) * dist - 32}
				<div
					in:fly={{ x: 48, y: 48, duration: 400 }}
					out:fly={{ x: 48, y: -48, duration: 400 }}
					style={lobby.ready.length === 1
						? `translate: -50% -50%;`
						: `translate: calc(-50% + ${x}px) calc(-50% + ${-y}px);` +
							`scale: ${100 - Math.log2(lobby.ready.length) * 8}%;`}
					class="ease-out-cubic absolute top-1/2 left-1/2 flex flex-col items-center gap-y-2 transition-transform duration-[400ms]"
				>
					<Avatar {ctx} {uuid} />
					<p>{ctx.clients[uuid]!.username}</p>
				</div>
			{/each}
		</div>
		<DoubleRightArrow />
		<button
			style="border-image: linear-gradient(to right, var(--color-green), var(--color-light-green)) 1;"
			class="z-10 flex w-72 flex-col border bg-dark-dark-green/85 transition-colors hover:bg-[color-mix(in_oklch,var(--color-dark-dark-green)_85%,var(--color-light-green))]"
			onclick={() => openSettings()}
		>
			<div class="flex items-center justify-between px-4 pt-5 pb-3 font-serif text-4xl text-yellow">
				<p>{camelCaseToWords(ctx.settings.game)}</p>
				<DownArrow />
			</div>
			<div class="mb-3 h-[1px] bg-green"></div>
			<div class="grid grid-cols-2 justify-between p-4 pt-0 text-sm text-light-green">
				<p class="text-left font-medium">Difficulty</p>
				<p class="text-right">Easy</p>
				<p class="text-left font-medium">Starting Lives</p>
				<p class="text-right">2</p>
			</div>
		</button>
	</div>
	<div class="z-10 flex gap-x-4 p-4">
		<Button
			color="pastel-pink"
			class="flex-1"
			onclick={() => {
				sendMsg({
					kind: 'lobby',
					data: { kind: lobby.ready.includes(ctx.uuid) ? 'unready' : 'ready' }
				});
			}}
		>
			{lobby.ready.includes(ctx.uuid) ? 'Unready' : 'Ready'}
		</Button>
		<Button
			disabled={ctx.settings.owner !== ctx.uuid || lobby.timerStart === null}
			color="pastel-blue"
			class="flex-1"
			onclick={() => {
				sendMsg({ kind: 'lobby', data: { kind: 'startEarly' } });
			}}
		>
			Start Early
		</Button>
	</div>
</section>
