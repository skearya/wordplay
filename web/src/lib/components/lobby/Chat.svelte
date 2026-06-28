<script lang="ts" module>
	type Message = { key: string; author: string; content: string };

	let messages: Message[] = $state([]);
</script>

<script lang="ts">
	import type { Props } from '$lib/context';
	import type { TransitionConfig } from 'svelte/transition';
	import { onMount } from 'svelte';
	import { cubicOut } from 'svelte/easing';
	import Avatar from '$lib/components/Avatar.svelte';
	import { generalEmitter } from '$lib/events';
	import Me from '$lib/icons/Me.svelte';
	import Tag from '$lib/ui/Tag.svelte';

	let { ctx = $bindable(), sendMsg }: Omit<Props<never>, 'state'> = $props();

	let messagesContainer: HTMLElement;

	onMount(() =>
		generalEmitter.handle({
			chat: ({ author, content }) => {
				messages.push({ key: crypto.randomUUID(), author, content });
			},
			error: ({ message }) => {
				messages.push({ key: crypto.randomUUID(), author: 'Server', content: message });
			}
		})
	);

	$effect(() => {
		messages.length;

		messagesContainer.scrollTo({
			top: messagesContainer.scrollHeight,
			behavior: 'smooth'
		});
	});

	function messageIn(_node: HTMLElement): TransitionConfig {
		const delay = 0;
		const duration = 400;
		const easing = cubicOut;

		return {
			delay,
			duration,
			easing,
			css: (t) =>
				`background-color: color-mix(in srgb, var(--color-pastel-pink) ${50 - t * 50}%, transparent);`
		};
	}
</script>

<div class="relative flex h-full flex-col border border-pastel-pink bg-pastel-pink/10">
	<Tag class="bg-pastel-pink">Chat</Tag>
	<div bind:this={messagesContainer} class="flex-1 overflow-x-hidden overflow-y-auto pb-1">
		<div
			style="--stripe-color: var(--color-pastel-pink);"
			class="striped h-16 w-full border-b border-pastel-pink opacity-25"
		></div>
		{#each ['Welcome to Wordplay!', 'Please leave issues or feedback on <a href="https://github.com/skearya/wordplay" target="_blank" class="text-gray-200 underline">GitHub</a>.'] as content, i}
			{#await new Promise((resolve) => setTimeout(resolve, 500 + 2000 * i)) then}
				{@render messageSnippet({ key: `${i}`, author: 'Server', content }, true)}
			{/await}
		{/each}
		{#each messages as message (message.key)}
			{@render messageSnippet(message, false)}
		{/each}
	</div>
	<input
		maxlength="250"
		placeholder="Send a message..."
		class="border-t border-pink/25 px-3.5 py-3 transition-colors duration-75 focus:border-pink focus:ring-0 focus:outline-none disabled:opacity-50"
		onkeyup={(e) => {
			if (e.key === 'Enter' && e.currentTarget.value != '') {
				sendMsg({
					kind: 'general',
					data: { kind: 'chat', content: e.currentTarget.value }
				});

				e.currentTarget.value = '';
			}
		}}
	/>
</div>

{#snippet messageSnippet({ author, content }: Message, trusted: boolean)}
	<div in:messageIn class="flex items-center gap-2 px-2 py-1.5">
		{#if author === 'Server'}
			<Me width={147 * 0.275} height={152 * 0.275} class="flex-none" />
		{:else}
			<Avatar {ctx} uuid={author} size="sm" class="flex-none" />
		{/if}
		<div class="text-sm leading-snug">
			<p class="font-medium">{author === 'Server' ? author : ctx.clients[author].username}</p>
			{#if trusted}{@html content}{:else}{content}{/if}
		</div>
	</div>
{/snippet}
