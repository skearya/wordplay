<script lang="ts" module>
	let showSettings = $state(false);

	export function openSettings() {
		showSettings = true;
	}
</script>

<script lang="ts">
	import type { Props } from '$lib/context';
	import { onMount } from 'svelte';
	import { fly } from 'svelte/transition';
	import Avatar from '$lib/components/Avatar.svelte';
	import { generalEmitter } from '$lib/events';
	import Logo from '$lib/icons/Logo.svelte';
	import Settings from '$lib/icons/Settings.svelte';
	import { camelCaseToWords, onClickOutside } from '$lib/utils';

	let { ctx = $bindable(), sendMsg }: Omit<Props<never>, 'state'> = $props();

	let innerNavElement: HTMLElement;
	let settingsElement: HTMLFormElement | undefined = $state();

	let ping = $state(10);

	const PING_INTERVAL = 5000;

	onMount(() => {
		const pingFn = () =>
			sendMsg({
				kind: 'general',
				data: { kind: 'ping', timestamp: Date.now() }
			});

		pingFn();
		const intervalId = setInterval(pingFn, PING_INTERVAL);

		const generalCleanup = generalEmitter.handle({
			pong: ({ timestamp }) => {
				ping = Date.now() - timestamp;
			}
		});

		return () => {
			generalCleanup();
			clearInterval(intervalId);
		};
	});

	const navOnFocus = () => {
		if (ctx.state.kind === 'lobby') return;
		innerNavElement.classList.remove('-translate-y-full');
	};

	const navOnBlur = () => {
		if (ctx.state.kind === 'lobby') return;
		innerNavElement.classList.add('-translate-y-full');
	};

	let settings = $derived(ctx.settings);
	let disabled = $derived(ctx.uuid !== ctx.settings.owner);

	const updateSettings = () => {
		sendMsg({ kind: 'general', data: { kind: 'settings', ...$state.snapshot(settings) } });
	};
</script>

<nav
	onmouseover={() => navOnFocus()}
	onfocus={() => navOnFocus()}
	onmouseout={() => navOnBlur()}
	onblur={() => navOnBlur()}
	class={['z-40', ctx.state.kind !== 'lobby' ? 'fixed top-0 left-0 w-full' : null]}
>
	<div
		bind:this={innerNavElement}
		class={[
			'relative flex items-center justify-between px-5 py-4 transition-transform',
			ctx.state.kind === 'lobby'
				? 'translate-y-0'
				: '-translate-y-full border-b border-b-foreground/25 bg-background'
		]}
	>
		<Logo />
		<button
			class={[
				'group absolute top-0 right-1/2 flex translate-x-1/2 items-center gap-x-2.5 justify-self-center border border-t-0 border-pastel-light-red bg-pastel-light-red/25 px-8 transition-all ease-out',
				showSettings ? 'py-4' : 'py-2'
			]}
			onclick={(e) => {
				if (!settingsElement?.contains(e.target as Node)) {
					showSettings = !showSettings;
				}
			}}
			{@attach showSettings && onClickOutside(() => (showSettings = false))}
		>
			<Settings
				width={32}
				height={32}
				class={`text-pastel-light-red transition-transform duration-300 ease-out group-hover:rotate-360 ${showSettings ? 'rotate-360' : 'rotate-0'}`}
			/>
			<p>Settings</p>
			{#if showSettings}
				<form
					bind:this={settingsElement}
					transition:fly={{ y: -8, duration: 300 }}
					class={[
						'absolute top-full right-1/2 mt-4 w-80 translate-x-1/2 space-y-2.5 border border-foreground/25 p-2.5 text-left backdrop-blur-3xl backdrop-brightness-50',
						ctx.settings.owner !== ctx.uuid && 'cursor-not-allowed grayscale-100'
					]}
					onchange={() => updateSettings()}
				>
					<p
						style="border-image: linear-gradient(to right, var(--color-light-green), transparent) 1;"
						class="border-b border-foreground/50 pb-1 font-serif text-lg"
					>
						General
					</p>
					<p class="mb-0.5">Public</p>
					<input type="checkbox" bind:checked={settings.public} {disabled} />
					<p class="mb-0.5">Room Size</p>
					<input
						type="number"
						bind:value={settings.size}
						{disabled}
						min="2"
						max="24"
						class="no-spinner border border-pastel-green/25 bg-dark-dark-green px-1 py-0.5 text-sm focus:border-green focus:ring-green"
					/>
					<p class="mb-0.5">Owner</p>
					<select
						bind:value={settings.owner}
						{disabled}
						class="border border-pastel-green/25 bg-dark-dark-green p-1 text-sm"
					>
						{#each Object.entries(ctx.clients) as [uuid, client]}
							<option value={uuid}>
								{client.username}
							</option>
						{/each}
					</select>
					<p class="mb-0.5">Game</p>
					<select
						bind:value={settings.game}
						{disabled}
						class="border border-pastel-green/25 bg-dark-dark-green p-1 text-sm"
					>
						{#each ['wordBomb', 'anagrams'] as game}
							<option value={game}>
								{camelCaseToWords(game)}
							</option>
						{/each}
					</select>
					<p
						style="border-image: linear-gradient(to right, var(--color-light-green), transparent) 1;"
						class="mt-1.5 border-b border-foreground/50 pb-1 font-serif text-lg"
					>
						Word Bomb
					</p>
					<p class="mb-0.5">Minimum WPP</p>
					<div class="flex items-center gap-x-2">
						<span class="border border-pastel-green/25 bg-dark-dark-green p-1 text-sm">
							{settings.wordBomb.minWpp}
						</span>
						<input
							type="range"
							bind:value={settings.wordBomb.minWpp}
							{disabled}
							min="50"
							max="1000"
							class="flex-1"
						/>
					</div>
				</form>
			{/if}
		</button>
		<div class="flex items-center gap-x-4 justify-self-end">
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
			<div
				class="bg-bright-green px-2 py-2 font-mono text-sm text-black"
				style="clip-path: polygon(0 0, calc(100% - 8px) 0, 100% 8px, 100% 100%, 8px 100%, 0 calc(100% - 8px));"
			>
				{ping}ms
			</div>
		</div>
	</div>
</nav>
