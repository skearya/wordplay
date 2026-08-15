<script lang="ts">
	const {
		width = 48,
		height = 48,
		class: className
	}: { width?: number; height?: number; class?: string } = $props();
</script>

<svg
	{width}
	{height}
	viewBox="0 0 48 48"
	fill="none"
	filter="url(#slightNoiseFilter)"
	xmlns="http://www.w3.org/2000/svg"
	class={className}
>
	<g clip-path="url(#clip0_148_131)">
		<mask
			id="mask0_148_131"
			style="mask-type: luminance"
			maskUnits="userSpaceOnUse"
			x="0"
			y="0"
			width="48"
			height="48"
		>
			<path d="M48 48V0L0 0V48H48Z" fill="white" />
		</mask>
		<g mask="url(#mask0_148_131)">
			<path
				d="M0.5 46.7988V1.20117L23.4512 24L0.5 46.7988ZM24.3389 46.7988L24.3389 1.20117L47.291 24L24.3389 46.7988Z"
				fill="#040605"
				stroke="#F3ACFF"
			/>
		</g>
	</g>
	<defs>
		<clipPath id="clip0_148_131">
			<rect width="48" height="48" fill="white" transform="matrix(0 1 -1 0 48 0)" />
		</clipPath>
		<filter id="slightNoiseFilter">
			<feTurbulence
				type="turbulence"
				baseFrequency="0.01"
				numOctaves="1"
				seed="1"
				result="turbulence"
			>
				<animate
					attributeName="seed"
					values="1;2;3;4;5;6;7;8;9;10;11;12;13;14;15;16;17;18;19;20;"
					dur="2000ms"
					repeatCount="indefinite"
				/>
			</feTurbulence>
			<feComposite operator="in" in="turbulence" in2="SourceAlpha" result="composite" />
			<feBlend in="SourceGraphic" in2="composite" mode="normal" result="blended" />
			<feDisplacementMap in="blended" in2="turbulence" scale="5" />
		</filter>
	</defs>
</svg>
