<script lang="ts">
	const {
		width = 224,
		height = 228,
		class: className
	}: { width?: number; height?: number; class?: string } = $props();
</script>

<svg
	{width}
	{height}
	viewBox="0 0 224 228"
	fill="none"
	xmlns="http://www.w3.org/2000/svg"
	class={className}
>
	<path
		d="M223.501 114.329C223.501 51.5912 173.467 0.731445 111.749 0.731445C50.0296 0.731445 0 51.5912 0 114.329C0 177.066 50.0323 227.926 111.75 227.926C173.468 227.926 223.501 177.066 223.501 114.329Z"
		fill="url(#paint0_linear_280_112)"
		fill-opacity="0.1"
	/>
	<path
		d="M111.749 3.73145C171.763 3.73145 220.501 53.2019 220.501 114.329C220.501 175.456 171.764 224.926 111.75 224.926C51.7353 224.926 3.00019 175.456 3 114.329C3 53.2019 51.7326 3.7315 111.749 3.73145Z"
		stroke="url(#paint1_linear_280_112)"
		stroke-opacity="0.7"
		stroke-width="6"
		stroke-linecap="round"
	/>
	<defs>
		<linearGradient
			id="paint0_linear_280_112"
			x1="111.75"
			y1="0.731445"
			x2="111.75"
			y2="227.926"
			gradientUnits="userSpaceOnUse"
		>
			<stop stop-color="#F6F5B4" />
			<stop offset="1" stop-color="#DF4A2D" />
		</linearGradient>
		<linearGradient
			id="paint1_linear_280_112"
			x1="111.75"
			y1="0.731445"
			x2="111.75"
			y2="227.926"
			gradientUnits="userSpaceOnUse"
		>
			<stop stop-color="#F6F5B4" />
			<stop offset="1" stop-color="#DF4A2D" />
		</linearGradient>
	</defs>
</svg>
