<script lang="ts">
	const {
		width = 24,
		height = 24,
		class: className
	}: { width?: number; height?: number; class?: string } = $props();
</script>

<svg
	{width}
	{height}
	viewBox="0 0 24 24"
	fill="none"
	xmlns="http://www.w3.org/2000/svg"
	class={className}
>
	<path
		d="M21.1695 6L22.5 7.182L12 18L1.5 7.182L2.8305 6L12 15.447L21.1695 6Z"
		fill="currentColor"
	/>
</svg>
