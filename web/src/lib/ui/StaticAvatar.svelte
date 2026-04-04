<script lang="ts">
	import type { HTMLImgAttributes } from 'svelte/elements';

	let {
		size = 'md',
		username,
		avatarUrl,
		class: className,
		...rest
	}: {
		size?: 'sm' | 'md' | 'lg';
		username: string;
		avatarUrl?: string;
	} & HTMLImgAttributes = $props();
</script>

<img
	src={avatarUrl ? avatarUrl : `https://avatar.vercel.sh/${username}`}
	alt={username}
	width="120"
	height="120"
	class={[
		size === 'sm' ? 'size-10' : size === 'md' ? 'size-24' : 'size-36',
		'rounded-full transition-opacity',
		className
	]}
	{...rest}
/>
