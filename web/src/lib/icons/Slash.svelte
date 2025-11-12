<script lang="ts">
	const {
		width = 23,
		height = 77,
		class: className
	}: { width?: number; height?: number; class?: string } = $props();
</script>

<svg
	{width}
	{height}
	viewBox="0 0 23 77"
	fill="none"
	xmlns="http://www.w3.org/2000/svg"
	class={className}
>
	<path d="M1.25098 76.7498L21.749 0.25" stroke="#E9B8FF" stroke-width="2" />
</svg>
