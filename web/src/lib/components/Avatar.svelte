<script lang="ts">
	import type { Props } from '$lib/context';

	let { user, ctx }: { user: string } & Props<never> = $props();
</script>

<img
	src={`https://avatar.vercel.sh/${ctx.clients[user]!.username}`}
	alt="avatar"
	width="120"
	height="120"
	class="size-24 rounded-full"
/>
