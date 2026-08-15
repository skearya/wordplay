<!-- https://github.com/vercel/avatar/blob/master/utils/gradient.ts -->

<script lang="ts" module>
	import color from 'tinycolor2';
	import { memoize } from '$lib/utils';

	function hash(str: string) {
		let hash = 0;

		for (let i = 0, len = str.length; i < len; i++) {
			let chr = str.charCodeAt(i);
			hash = (hash << 5) - hash + chr;
			hash |= 0;
		}

		return hash;
	}

	export const generateGradient = memoize((username: string) => {
		const first = color({ h: hash(username) % 360, s: 0.95, l: 0.5 });
		const second = first.triad()[1];

		return {
			fromColor: first.toHexString(),
			toColor: second.toHexString()
		};
	});
</script>

<script lang="ts">
	import type { SvelteHTMLElements } from 'svelte/elements';

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
	<img src={avatarUrl} alt={username} class={containerClass()} {...rest} />
{:else}
	{@const { fromColor, toColor } = generateGradient(username)}
	{@const id = crypto.randomUUID()}

	<svg
		viewBox="0 0 120 120"
		version="1.1"
		xmlns="http://www.w3.org/2000/svg"
		class={containerClass()}
		{...rest}
	>
		<g>
			<defs>
				<linearGradient {id} x1="0" y1="0" x2="1" y2="1">
					<stop offset="0%" stop-color={fromColor} />
					<stop offset="100%" stop-color={toColor} />
				</linearGradient>
			</defs>
			<rect fill={`url(#${id})`} x="0" y="0" width="120" height="120" />
		</g>
	</svg>
{/if}
