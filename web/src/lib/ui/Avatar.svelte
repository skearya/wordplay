<!-- https://github.com/vercel/avatar/blob/master/utils/gradient.ts -->

<script lang="ts" module>
	import color from 'tinycolor2';

	function djb2(str: string): number {
		let hash = 5381;
		for (let i = 0; i < str.length; i++) {
			hash = (hash << 5) + hash + str.charCodeAt(i);
		}
		return hash;
	}

	type Gradient = { fromColor: string; toColor: string };

	const avatarGradientCache: { [username: string]: Gradient } = {};

	export function generateGradient(username: string): Gradient {
		if (avatarGradientCache[username]) return avatarGradientCache[username];

		const first = color({ h: djb2(username) % 360, s: 0.95, l: 0.5 });
		const second = first.triad()[1];

		return (avatarGradientCache[username] = {
			fromColor: first.toHexString(),
			toColor: second.toHexString()
		});
	}
</script>

<script lang="ts">
	import type { SvelteHTMLElements } from 'svelte/elements';
	import { keepAlphanumeric } from '$lib/utils';

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
	} & SvelteHTMLElements['img'] &
		SvelteHTMLElements['svg'] = $props();

	const containerClass = () => [
		'rounded-full',
		size === 'sm' ? 'size-10' : size === 'md' ? 'size-24' : 'size-36',
		className
	];
</script>

{#if avatarUrl}
	<img src={avatarUrl} alt={username} width="120" height="120" class={containerClass()} {...rest} />
{:else}
	{@const { fromColor, toColor } = generateGradient(username)}
	<svg
		width="120"
		height="120"
		viewBox="0 0 120 120"
		version="1.1"
		xmlns="http://www.w3.org/2000/svg"
		class={containerClass()}
		{...rest}
	>
		<g>
			<defs>
				<linearGradient id={keepAlphanumeric(username)} x1="0" y1="0" x2="1" y2="1">
					<stop offset="0%" stop-color={fromColor} />
					<stop offset="100%" stop-color={toColor} />
				</linearGradient>
			</defs>
			<rect fill={`url(#${keepAlphanumeric(username)})`} x="0" y="0" width="120" height="120" />
		</g>
	</svg>
{/if}
