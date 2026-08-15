<script lang="ts">
	const {
		width = 20,
		height = 21,
		class: className
	}: { width?: number; height?: number; class?: string } = $props();
</script>

<svg
	{width}
	{height}
	viewBox="0 0 20 21"
	fill="none"
	xmlns="http://www.w3.org/2000/svg"
	class={className}
>
	<path
		d="M13.7955 14.3111L19 19.5M16 9C16 13.1421 12.6421 16.5 8.5 16.5C4.35786 16.5 1 13.1421 1 9C1 4.85786 4.35786 1.5 8.5 1.5C12.6421 1.5 16 4.85786 16 9Z"
		stroke="#B0B0B0"
		stroke-width="2"
		stroke-linecap="round"
		stroke-linejoin="round"
	/>
</svg>
