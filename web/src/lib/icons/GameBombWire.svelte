<script lang="ts">
	const {
		width = 112,
		height = 114,
		class: className
	}: { width?: number; height?: number; class?: string } = $props();
</script>

<svg
	{width}
	{height}
	viewBox="0 0 112 114"
	fill="none"
	xmlns="http://www.w3.org/2000/svg"
	filter="url(#noiseFilter)"
	class={className}
>
	<path
		d="M34.5003 78.7314L4.7002 109.024L34.5003 78.7314Z"
		fill="url(#paint0_linear_0_1)"
		fill-opacity="0.3"
	/>
	<path
		d="M36.6389 80.8353C37.8008 79.6542 37.7853 77.7547 36.6041 76.5928C35.423 75.4309 33.5236 75.4464 32.3616 76.6276L34.5003 78.7314L36.6389 80.8353ZM2.56156 106.92L0.457697 109.059L4.73497 113.267L6.83883 111.128L4.7002 109.024L2.56156 106.92ZM34.5003 78.7314L32.3616 76.6276L2.56156 106.92L4.7002 109.024L6.83883 111.128L36.6389 80.8353L34.5003 78.7314Z"
		fill="url(#paint1_linear_0_1)"
		fill-opacity="0.7"
	/>
	<path
		d="M49.1183 8.34907C51.8852 1.21698 61.8146 1.21698 64.5816 8.34907L74.3321 33.4807C75.177 35.6582 76.8726 37.3818 79.0152 38.2406L103.737 48.1523C110.754 50.9652 110.754 61.059 103.737 63.8718L79.0152 73.7835C76.8726 74.6423 75.177 76.366 74.3321 78.5434L64.5816 103.675C61.8146 110.807 51.8852 110.807 49.1183 103.675L39.3677 78.5434C38.5229 76.366 36.8273 74.6423 34.6846 73.7835L9.96251 63.8718C2.94609 61.059 2.94609 50.9652 9.96251 48.1523L34.6846 38.2406C36.8273 37.3818 38.5229 35.6582 39.3677 33.4807L49.1183 8.34907Z"
		fill="url(#paint2_linear_0_1)"
		fill-opacity="0.3"
		stroke="url(#paint3_linear_0_1)"
		stroke-opacity="0.8"
		stroke-width="6"
	/>
	<defs>
		<linearGradient
			id="paint0_linear_0_1"
			x1="19.6002"
			y1="78.7314"
			x2="19.6002"
			y2="109.024"
			gradientUnits="userSpaceOnUse"
		>
			<stop stop-color="#F6F5B4" />
			<stop offset="1" stop-color="#DF4A2D" />
		</linearGradient>
		<linearGradient
			id="paint1_linear_0_1"
			x1="19.6002"
			y1="78.7314"
			x2="19.6002"
			y2="109.024"
			gradientUnits="userSpaceOnUse"
		>
			<stop stop-color="#F6F5B4" />
			<stop offset="1" stop-color="#DF4A2D" />
		</linearGradient>
		<linearGradient
			id="paint2_linear_0_1"
			x1="56.8499"
			y1="3"
			x2="56.8499"
			y2="109.024"
			gradientUnits="userSpaceOnUse"
		>
			<stop stop-color="#F6F5B4" />
			<stop offset="1" stop-color="#DF4A2D" />
		</linearGradient>
		<linearGradient
			id="paint3_linear_0_1"
			x1="56.8499"
			y1="3"
			x2="56.8499"
			y2="109.024"
			gradientUnits="userSpaceOnUse"
		>
			<stop stop-color="#F6F5B4" />
			<stop offset="1" stop-color="#DF4A2D" />
		</linearGradient>
		<filter id="noiseFilter">
			<feTurbulence
				type="turbulence"
				baseFrequency="0.2"
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
			<feDisplacementMap in="blended" in2="turbulence" scale="10" />
		</filter>
	</defs>
</svg>
