<script lang="ts">
	const {
		width = 81,
		height = 15,
		class: className
	}: { width?: number; height?: number; class?: string } = $props();
</script>

<svg
	{width}
	{height}
	viewBox="0 0 81 15"
	fill="none"
	xmlns="http://www.w3.org/2000/svg"
	class={className}
>
	<path
		d="M80.7071 8.07088C81.0976 7.68035 81.0976 7.04719 80.7071 6.65666L74.3432 0.292702C73.9526 -0.0978222 73.3195 -0.0978222 72.9289 0.292702C72.5384 0.683227 72.5384 1.31639 72.9289 1.70692L78.5858 7.36377L72.9289 13.0206C72.5384 13.4111 72.5384 14.0443 72.9289 14.4348C73.3195 14.8254 73.9526 14.8254 74.3432 14.4348L80.7071 8.07088ZM0 7.36377L0 8.36377H80V7.36377V6.36377H0L0 7.36377Z"
		fill="url(#paint0_linear_408_36)"
	/>
	<defs>
		<linearGradient
			id="paint0_linear_408_36"
			x1="80"
			y1="7.86377"
			x2="0"
			y2="7.86377"
			gradientUnits="userSpaceOnUse"
		>
			<stop stop-color="white" />
			<stop offset="1" stop-color="#999999" stop-opacity="0" />
		</linearGradient>
	</defs>
</svg>
