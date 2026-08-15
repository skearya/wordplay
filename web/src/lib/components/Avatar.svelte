<script lang="ts">
	import type { Context } from '$lib/context';
	import type { ComponentProps } from 'svelte';
	import Avatar from '$lib/ui/Avatar.svelte';

	type UIAvatarProps = ComponentProps<typeof Avatar>;

	let {
		ctx,
		uuid,
		size = 'md',
		class: className,
		...rest
	}: {
		ctx: Context;
		uuid: string;
	} & Partial<UIAvatarProps> = $props();

	let user = $derived(ctx.clients[uuid]);
</script>

<Avatar
	{size}
	username={user.username}
	avatarUrl={user.avatarUrl ?? undefined}
	title={user.connected ? user.username : `${user.username} (disconnected)`}
	class={[!user.connected && 'animate-pulse', className]}
	{...rest}
/>
