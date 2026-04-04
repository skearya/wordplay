<script lang="ts">
	import type { Context, Props } from '$lib/context';
	import type { ComponentProps } from 'svelte';
	import StaticAvatar from '$lib/ui/StaticAvatar.svelte';

	type StaticAvatarProps = ComponentProps<typeof StaticAvatar>;

	let {
		ctx,
		uuid,
		size = 'md',
		class: className,
		...rest
	}: {
		ctx: Context;
		uuid: string;
	} & Partial<StaticAvatarProps> = $props();

	let user = $derived(ctx.clients[uuid]!);
</script>

<StaticAvatar
	{size}
	username={user.username}
	avatarUrl={user.avatarUrl ?? undefined}
	title={user.connected ? user.username : `${user.username} (disconnected)`}
	class={[!user.connected && 'animate-pulse', className]}
	{...rest}
/>
