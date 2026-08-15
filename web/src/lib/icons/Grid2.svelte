<script lang="ts">
	const {
		width = 200,
		height = 200,
		class: className
	}: { width?: number; height?: number; class?: string } = $props();
</script>

<svg
	{width}
	{height}
	viewBox="0 0 200 200"
	fill="none"
	xmlns="http://www.w3.org/2000/svg"
	class={className}
>
	<g clip-path="url(#clip0_106_773)">
		<mask
			id="mask0_106_773"
			style="mask-type:luminance"
			maskUnits="userSpaceOnUse"
			x="0"
			y="0"
			width="200"
			height="200"
		>
			<path d="M200 0H0V200H200V0Z" fill="white" />
		</mask>
		<g mask="url(#mask0_106_773)">
			<path
				fill-rule="evenodd"
				clip-rule="evenodd"
				d="M0 0H50V50H0V0ZM100 50H50V100H0V150H50V200H100V150H150V200H200V150H150V100H200V50H150V0H100V50ZM100 100H150V50H100V100ZM100 100V150H50V100H100Z"
				fill="white"
				fill-opacity="20%"
			/>
		</g>
	</g>
	<defs>
		<clipPath id="clip0_106_773">
			<rect width="200" height="200" fill="white" />
		</clipPath>
	</defs>
</svg>
