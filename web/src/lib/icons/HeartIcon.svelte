<script lang="ts">
	const { width = 26, height = 22, class: className }: { width?: number; height?: number, class?: string } = $props();
</script>

<svg {width} {height} viewBox="0 0 25 21" fill="none" xmlns="http://www.w3.org/2000/svg" class={className}>
	<path
		d="M0 7.13885C0 13.0709 5.02429 16.2321 8.70216 19.0615C10 20.06 11.25 21 12.5 21C13.75 21 15 20.06 16.2979 19.0615C19.9757 16.2321 25 13.0709 25 7.13885C25 1.20675 18.1248 -3.00018 12.5 2.70287C6.8752 -3.00018 0 1.20675 0 7.13885Z"
		fill="#DA5858"
	/>
</svg>
