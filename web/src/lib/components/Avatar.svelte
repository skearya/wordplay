<script lang="ts">
	import type { Props } from '$lib/context';
	import type { HTMLImgAttributes } from 'svelte/elements';

	let {
		ctx,
		uuid,
		size = 'md',
		class: className,
		...rest
	}: Omit<Props<never>, 'state' | 'sendMsg'> & {
		uuid: string;
		size?: 'sm' | 'md' | 'lg';
	} & HTMLImgAttributes = $props();

	let user = $derived(ctx.clients[uuid]!);
</script>

<img
	src={user.avatarUrl ? user.avatarUrl : `https://avatar.vercel.sh/${user.username}`}
	alt={user.username}
	title={user.connected ? user.username : `${user.username} (disconnected)`}
	width="120"
	height="120"
	class={[
		size === 'sm' ? 'size-10' : size === 'md' ? 'size-24' : 'size-36',
		user.connected ? 'opacity-100' : 'animate-pulse',
		'rounded-full transition-opacity',
		className
	]}
	{...rest}
/>
