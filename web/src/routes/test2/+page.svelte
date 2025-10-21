<script lang="ts">
	import type { PageProps } from './$types';
	import type { SVGAttributes } from 'svelte/elements';
	import { getAbortSignal, onMount } from 'svelte';
	import { flip } from 'svelte/animate';
	import { blur, fade, fly, scale, slide } from 'svelte/transition';
	import crownSvg from '$lib/assets/crown.svg';
	import grid2Svg from '$lib/assets/grid2.svg';
	import slashSvg from '$lib/assets/slash.svg';
	import { lerp } from '$lib/utils';

	let { data }: PageProps = $props();

	let activeOutlineElement: HTMLElement;
	let playersContainer: HTMLElement;
	let bombElement: HTMLElement;
	let playerElements = $state<Record<string, HTMLElement>>({});
	let arrowElements = $state<HTMLElement[]>([]);

	let unreadMessages = $state(0);
	let readyPlayers = $state<string[]>([crypto.randomUUID(), crypto.randomUUID()]);
	let activePlayer = $state(0);

	onMount(() => {
		const intervalId = setInterval(() => {
			unreadMessages += 1;
		}, 1500);

		document.addEventListener(
			'keydown',
			(e) => {
				if (e.key === '=') {
					readyPlayers.push(crypto.randomUUID());
				} else if (e.key === '-') {
					readyPlayers.pop();
				} else if (e.key === 'n') {
					activePlayer++;
					activePlayer %= readyPlayers.length;
				}
			},
			{ signal: getAbortSignal() }
		);

		return () => clearInterval(intervalId);
	});

	$effect(() => {
		const playerUUID = readyPlayers[activePlayer];
		const playerBBox = playerElements[playerUUID].getBoundingClientRect();
		const outlineBBox = activeOutlineElement.getBoundingClientRect();

		const playerX = playerBBox.left + playerBBox.width / 2;
		const playerY = playerBBox.top + playerBBox.height / 2;

		const outlineX = outlineBBox.left + outlineBBox.width / 2;
		const outlineY = outlineBBox.top + outlineBBox.height / 2;

		const playerDistanceX = playerX - window.innerWidth / 2;
		const playerDistanceY = playerY - window.innerHeight / 2;

		console.log(playersContainer.style.translate);

		const [prevContainerTransformX, prevContainerTransformY] = (
			playersContainer.style.translate || '0px 0px'
		)
			.split(' ')
			.map((value) => value.substring(0, value.length - 2))
			.map((value) => parseFloat(value));

		const containerTransformX = -playerDistanceX * 0.008;
		const containerTransformY = -playerDistanceY * 0.008;

		const bombTransformX = -playerDistanceX * 0.005;
		const bombTransformY = -playerDistanceY * 0.005;

		playersContainer.style.translate = `${containerTransformX}px ${containerTransformY}px`;
		bombElement.style.translate = `calc(-50% + ${bombTransformX}px) calc(-50% + ${bombTransformY}px)`;

		const newPlayerX = playerX + containerTransformX - prevContainerTransformX;
		const newPlayerY = playerY + containerTransformY - prevContainerTransformY;

		const toX1 = lerp(outlineX, playerX, 0.1);
		const toY1 = lerp(outlineY, playerY, 0.1);

		activeOutlineElement
			.animate(
				{
					transform: `translateX(calc(-50% + ${toX1}px)) translateY(calc(-50% + ${toY1}px))`,
					filter: 'blur(2px)',
					opacity: '0%'
				},
				{
					fill: 'forwards',
					easing: 'cubic-bezier(0.25, 1, 0.5, 1)',
					duration: 150
				}
			)
			.finished.then(() => {
				const fromX2 = lerp(outlineX, newPlayerX, 0.9);
				const fromY2 = lerp(outlineY, newPlayerY, 0.9);

				const toX3 = newPlayerX;
				const toY3 = newPlayerY;

				activeOutlineElement.animate(
					{
						transform: [
							`translateX(calc(-50% + ${fromX2}px)) translateY(calc(-50% + ${fromY2}px))`,
							`translateX(calc(-50% + ${toX3}px)) translateY(calc(-50% + ${toY3}px))`
						],
						filter: 'blur(0px)',
						opacity: '100%'
					},
					{
						fill: 'forwards',
						easing: 'cubic-bezier(0.61, 1, 0.88, 1)',
						duration: 200
					}
				);
			});

		const arrowElement =
			arrowElements[activePlayer - 1 === -1 ? readyPlayers.length - 1 : activePlayer - 1];

		(arrowElement.firstChild as HTMLElement).animate(
			{
				translate: `0px 32px`,
				opacity: ['100%', '0%']
			},
			{
				easing: 'ease-out',
				duration: 450
			}
		);
	});
</script>

<svg height="0">
	<filter id="noiseFilter">
		<feTurbulence type="turbulence" baseFrequency="0.2" numOctaves="1" seed="1" result="turbulence">
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
</svg>

<main
	style="background: linear-gradient(180deg, rgba(0, 0, 0, 0) 0%, rgba(246, 245, 180, 0.08) 100%), #040605"
	class="relative flex h-screen overflow-hidden"
>
	<nav class="hidden items-center justify-between px-5 py-4">
		{@render wordplayLogo()}
		<div class="flex items-center gap-x-4">
			<div class="flex -space-x-6">
				{#each { length: 3 }}
					<img
						src="https://avatar.vercel.sh/s"
						alt="avatar"
						width="120"
						height="120"
						class="size-12 rounded-full border-2 border-black"
					/>
				{/each}
				<div
					class="size-12 content-center rounded-full border-2 border-[#1D1F1E] bg-black text-center"
				>
					+2
				</div>
			</div>
			{@render settingsIcon()}
		</div>
	</nav>
	<div class="flex flex-1 items-center justify-center gap-4 overflow-hidden p-4 pt-0">
		<div
			bind:this={bombElement}
			class="timing-function-0 absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 transition-transform duration-[400ms]"
		>
			{@render bombIcon()}
			<div class="absolute right-0 top-0 -translate-y-[4.8rem] translate-x-[4.8rem]">
				{@render bombWireIcon()}
			</div>
			<div
				style="font-family: 'PP Editorial New';"
				class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-[40%] text-7xl"
			>
				VEN
			</div>
		</div>
		<div bind:this={activeOutlineElement} class="absolute left-0 top-0">
			<div
				style={`scale: ${100 - Math.log2(readyPlayers.length) * 8}%;`}
				class="rotating-border size-48"
			></div>
		</div>
		<div
			bind:this={playersContainer}
			class="timing-function-0 relative size-full transition-transform duration-[400ms]"
		>
			{#each readyPlayers as uuid, i (uuid)}
				<div
					bind:this={playerElements[uuid]}
					style={`--angle-between: 2 * pi / ${readyPlayers.length};
							--angle: ${i} * var(--angle-between);
							--dist: min(100vw, 100vh) * 0.35;
							translate: calc(-50% + cos(var(--angle)) * var(--dist)) calc(-50% - sin(var(--angle)) * var(--dist));
							scale: ${100 - Math.log2(readyPlayers.length) * 8}%;`}
					class="timing-function-0 absolute left-1/2 top-1/2 flex flex-col items-center p-2 transition-transform duration-[400ms]"
				>
					<div class="relative mb-2">
						<img
							src={`https://avatar.vercel.sh/${i}`}
							alt="avatar"
							width="120"
							height="120"
							class="size-24 rounded-full"
						/>
						<div class="absolute bottom-0 left-0 flex flex-col gap-y-1.5 mix-blend-plus-lighter">
							{@render heartIcon()}
							{@render heartIcon()}
						</div>
						{#if i === 0}
							<div class="absolute right-0 top-0 -translate-y-1/2 translate-x-1/2">
								{@render starIcon()}
								<span
									class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-[40%] tracking-tight text-black"
								>
									x<span class="text-2xl font-semibold">10</span>
								</span>
							</div>
						{/if}
					</div>
					<p>skeary</p>
					<p>ovens</p>
				</div>
			{/each}
			{#each readyPlayers as uuid, i (uuid)}
				{@const angleBetween = (2 * Math.PI) / readyPlayers.length}
				{@const angle = i * angleBetween + angleBetween / 2}
				<div
					bind:this={arrowElements[i]}
					style={`--angle-between: 2 * pi / ${readyPlayers.length};
							--angle: ${i} * var(--angle-between) + var(--angle-between) / 2;
							--dist: min(100vw, 100vh) * 0.35;
							translate: calc(-50% + cos(var(--angle)) * var(--dist)) calc(-50% - sin(var(--angle)) * var(--dist));
							scale: ${100 - Math.log2(readyPlayers.length) * 8}%;
							rotate: ${Math.PI - angle}rad;`}
					class="timing-function-0 absolute left-1/2 top-1/2 text-transparent transition-transform duration-[400ms]"
				>
					<div class="opacity-0">
						{#each { length: 3 }}
							{@render downArrowIcon()}
						{/each}
					</div>
				</div>
			{/each}
		</div>
	</div>
</main>

{#snippet heartIcon()}
	<svg width="26" height="22" viewBox="0 0 25 21" fill="none" xmlns="http://www.w3.org/2000/svg">
		<path
			d="M0 7.13885C0 13.0709 5.02429 16.2321 8.70216 19.0615C10 20.06 11.25 21 12.5 21C13.75 21 15 20.06 16.2979 19.0615C19.9757 16.2321 25 13.0709 25 7.13885C25 1.20675 18.1248 -3.00018 12.5 2.70287C6.8752 -3.00018 0 1.20675 0 7.13885Z"
			fill="#DA5858"
		/>
	</svg>
{/snippet}

{#snippet starIcon()}
	<svg width="76" height="76" viewBox="0 0 76 76" fill="none" xmlns="http://www.w3.org/2000/svg">
		<path
			d="M27.3244 13.2814C32.0743 4.76049 34.4491 0.5 38 0.5C41.5509 0.5 43.9258 4.76045 48.6755 13.2814L49.9044 15.4859C51.2544 17.9072 51.929 19.118 52.9816 19.9168C54.0339 20.7156 55.3441 21.0121 57.9654 21.6052L60.3519 22.1451C69.5754 24.2321 74.1875 25.2756 75.2848 28.804C76.382 32.3323 73.238 36.0091 66.9496 43.3621L65.3229 45.2645C63.536 47.354 62.6424 48.3988 62.2404 49.6914C61.8387 50.984 61.9738 52.3779 62.2438 55.166L62.4897 57.704C63.4404 67.5147 63.9159 72.4201 61.0434 74.6008C58.1705 76.7814 53.8524 74.7931 45.2161 70.817L42.9819 69.788C40.5279 68.6581 39.3009 68.093 38 68.093C36.6991 68.093 35.4721 68.6581 33.0181 69.788L30.7839 70.817C22.1476 74.7931 17.8294 76.7814 14.9568 74.6008C12.0842 72.4201 12.5596 67.5147 13.5103 57.704L13.7562 55.166C14.0264 52.3779 14.1614 50.984 13.7595 49.6914C13.3576 48.3988 12.4641 47.354 10.6772 45.2645L9.05038 43.3621C2.76219 36.0091 -0.381918 32.3323 0.715332 28.804C1.81258 25.2756 6.42448 24.2321 15.6483 22.1451L18.0346 21.6052C20.6557 21.0121 21.9662 20.7156 23.0185 19.9168C24.0708 19.118 24.7457 17.9073 26.0955 15.4859L27.3244 13.2814Z"
			fill="#F6F5B4"
		/>
	</svg>
{/snippet}

{#snippet wordplayLogo()}
	<svg width="96" height="61" viewBox="0 0 96 61" fill="none" xmlns="http://www.w3.org/2000/svg">
		<path
			d="M85.0191 58.6036L92.1336 18.5848L51.7866 11.5283L49.0137 27.1268L51.6351 42.339L46.1291 43.3531L44.6724 51.5471L85.0191 58.6036Z"
			stroke="white"
			stroke-width="2"
		/>
		<path
			d="M63.1709 30.3682L65.2981 30.7402L64.9042 32.9555L65.0106 32.9741C65.5183 32.1747 66.1449 31.6315 66.8904 31.345C67.6567 31.0439 68.5185 30.9771 69.4758 31.1445C70.3266 31.2933 71.068 31.5952 71.7 32.0502C72.3319 32.5052 72.8325 33.0822 73.2018 33.7813C73.5887 34.4835 73.8281 35.2958 73.92 36.2184C74.0296 37.144 73.9875 38.1518 73.7937 39.242C73.5999 40.3321 73.292 41.2934 72.8697 42.1261C72.4653 42.9618 71.9602 43.6439 71.3545 44.1725C70.7664 44.7042 70.0971 45.0766 69.3464 45.2897C68.5956 45.5029 67.7949 45.5351 66.944 45.3863C65.0827 45.0608 63.8943 43.9827 63.379 42.1522L63.2727 42.1336L61.9411 49.6237L59.814 49.2517L63.1709 30.3682ZM66.6966 43.385C67.902 43.5959 68.916 43.3925 69.7386 42.775C70.5643 42.1398 71.085 41.2157 71.3007 40.0024L71.7133 37.6816C71.9289 36.4684 71.7568 35.4321 71.1967 34.5728C70.6398 33.6958 69.7586 33.1519 68.5532 32.9411C68.0746 32.8574 67.6012 32.8471 67.1331 32.9102C66.6857 32.959 66.2822 33.0696 65.9225 33.2425C65.5628 33.4152 65.2557 33.6515 65.0013 33.9515C64.7679 34.237 64.6182 34.5644 64.5526 34.9336L63.6524 39.9973C63.5743 40.4369 63.5926 40.848 63.7074 41.2306C63.843 41.5988 64.0484 41.9338 64.3237 42.2359C64.6022 42.5202 64.9431 42.7611 65.3467 42.9586C65.768 43.1592 66.218 43.3014 66.6966 43.385Z"
			fill="white"
		/>
		<path
			d="M44.2131 2.34717L51.3273 42.366L10.9805 49.4224L3.86621 9.40354L44.2131 2.34717Z"
			stroke="white"
			stroke-width="2"
		/>
		<path
			d="M17.2817 23.6443L19.3558 23.2816L21.8637 28.9071L24.3857 34.6117L24.4389 34.6024L25.0545 28.349L25.7373 22.1654L27.6252 21.8353L30.4523 27.405L33.2136 33.0678L33.2668 33.0585L33.6164 26.8516L34.0599 20.7099L36.0542 20.3611L34.8306 34.607L32.1982 35.0674L29.121 28.8616L27.1883 24.9301L27.1352 24.9394L26.7014 29.2847L25.9496 36.1602L23.3704 36.6113L17.2817 23.6443Z"
			fill="white"
		/>
	</svg>
{/snippet}

{#snippet settingsIcon()}
	<svg width="48" height="49" viewBox="0 0 48 49" fill="none" xmlns="http://www.w3.org/2000/svg">
		<path
			d="M24 30.5C27.3137 30.5 30 27.8137 30 24.5C30 21.1863 27.3137 18.5 24 18.5C20.6863 18.5 18 21.1863 18 24.5C18 27.8137 20.6863 30.5 24 30.5Z"
			stroke="#8BA698"
			stroke-width="1.5"
		/>
		<path
			d="M27.5308 4.80448C26.7956 4.5 25.8638 4.5 24 4.5C22.1362 4.5 21.2044 4.5 20.4692 4.80448C19.4891 5.21046 18.7104 5.98916 18.3044 6.96926C18.1191 7.41668 18.0466 7.937 18.0182 8.69598C17.9765 9.81136 17.4045 10.8438 16.4379 11.4019C15.4713 11.9599 14.2912 11.9391 13.3044 11.4175C12.6329 11.0626 12.146 10.8652 11.6659 10.802C10.6141 10.6636 9.55035 10.9486 8.70871 11.5944C8.07747 12.0788 7.61153 12.8858 6.67965 14.4999C5.74777 16.1139 5.28183 16.921 5.17797 17.7098C5.03951 18.7616 5.32453 19.8253 5.97035 20.667C6.26511 21.0512 6.67939 21.374 7.32237 21.778C8.26759 22.372 8.87577 23.3838 8.87571 24.5C8.87565 25.6162 8.26749 26.6278 7.32235 27.2216C6.67929 27.6258 6.26495 27.9488 5.97015 28.333C5.32433 29.1746 5.03931 30.2382 5.17779 31.29C5.28163 32.0788 5.74757 32.886 6.67945 34.5C7.61135 36.114 8.07729 36.9212 8.70851 37.4054C9.55015 38.0512 10.6139 38.3362 11.6657 38.1978C12.1458 38.1346 12.6326 37.9372 13.3041 37.5824C14.2909 37.0608 15.4711 37.04 16.4378 37.598C17.4045 38.1562 17.9765 39.1886 18.0182 40.3042C18.0466 41.063 18.1191 41.5834 18.3044 42.0308C18.7104 43.0108 19.4891 43.7896 20.4692 44.1956C21.2044 44.5 22.1362 44.5 24 44.5C25.8638 44.5 26.7956 44.5 27.5308 44.1956C28.5108 43.7896 29.2896 43.0108 29.6954 42.0308C29.8808 41.5834 29.9534 41.063 29.9818 40.304C30.0234 39.1886 30.5954 38.1562 31.562 37.598C32.5286 37.0398 33.7088 37.0608 34.6958 37.5824C35.3672 37.9372 35.854 38.1344 36.334 38.1976C37.3858 38.3362 38.4496 38.0512 39.2912 37.4054C39.9224 36.921 40.3884 36.114 41.3202 34.4998C42.2522 32.8858 42.7182 32.0788 42.822 31.29C42.9604 30.2382 42.6754 29.1744 42.0296 28.3328C41.7348 27.9486 41.3204 27.6256 40.6774 27.2216C39.7324 26.6278 39.1242 25.616 39.1242 24.4998C39.1242 23.3836 39.7324 22.3722 40.6774 21.7784C41.3206 21.3742 41.735 21.0514 42.0298 20.667C42.6756 19.8255 42.9606 18.7617 42.8222 17.7099C42.7184 16.9211 42.2524 16.1141 41.3204 14.5C40.3886 12.8859 39.9226 12.0789 39.2914 11.5945C38.4498 10.9487 37.386 10.6637 36.3342 10.8022C35.8542 10.8654 35.3674 11.0627 34.6958 11.4176C33.709 11.9392 32.5288 11.96 31.5622 11.4019C30.5954 10.8438 30.0234 9.81132 29.9818 8.69588C29.9534 7.93696 29.8808 7.41666 29.6954 6.96926C29.2896 5.98916 28.5108 5.21046 27.5308 4.80448Z"
			stroke="#8BA698"
			stroke-width="1.5"
		/>
	</svg>
{/snippet}

{#snippet doubleRightArrowIcon()}
	<svg width="48" height="48" viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg">
		<g clip-path="url(#clip0_148_131)">
			<mask
				id="mask0_148_131"
				style="mask-type:luminance"
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
					stroke="#475D50"
					fill="currentColor"
				/>
			</g>
		</g>
		<defs>
			<clipPath id="clip0_148_131">
				<rect width="48" height="48" fill="white" transform="matrix(0 1 -1 0 48 0)" />
			</clipPath>
		</defs>
	</svg>
{/snippet}

{#snippet downArrowIcon(props: SVGAttributes<SVGSVGElement> = {})}
	<svg
		width="24"
		height="24"
		viewBox="0 0 24 24"
		fill="none"
		xmlns="http://www.w3.org/2000/svg"
		{...props}
	>
		<path d="M21.1695 6L22.5 7.182L12 18L1.5 7.182L2.8305 6L12 15.447L21.1695 6Z" fill="#FAE1DD" />
	</svg>
{/snippet}

{#snippet wordBombIcon(props: SVGAttributes<SVGSVGElement> = {})}
	<svg
		width="901"
		height="916"
		viewBox="0 0 901 916"
		fill="none"
		xmlns="http://www.w3.org/2000/svg"
		{...props}
	>
		<g style="mix-blend-mode:color-dodge">
			<path
				d="M169.563 275.312C219.15 246.154 276.72 229.465 338.126 229.465C524.314 229.465 675.251 382.898 675.251 572.163C675.251 761.428 524.314 914.861 338.126 914.861C151.936 914.861 1 761.428 1 572.163C1 509.742 17.4171 451.223 46.1016 400.814"
				stroke="url(#paint0_linear_148_156)"
				stroke-width="1.5"
				stroke-linecap="round"
			/>
			<path
				d="M675.252 229.465L585.352 320.851"
				stroke="url(#paint1_linear_148_156)"
				stroke-width="1.5"
				stroke-linecap="round"
			/>
			<path
				d="M719.351 17.137C727.698 -4.37899 757.653 -4.37899 766 17.137L795.416 92.9536C797.964 99.5224 803.08 104.722 809.544 107.313L884.125 137.215C905.292 145.7 905.292 176.151 884.125 184.637L809.544 214.538C803.08 217.129 797.964 222.329 795.416 228.898L766 304.714C757.653 326.23 727.698 326.23 719.351 304.714L689.936 228.898C687.387 222.329 682.272 217.129 675.808 214.538L601.227 184.637C580.06 176.151 580.06 145.7 601.227 137.215L675.808 107.313C682.272 104.722 687.387 99.5224 689.936 92.9536L719.351 17.137Z"
				stroke="url(#paint2_linear_148_156)"
				stroke-width="1.5"
			/>
		</g>
		<defs>
			<linearGradient
				id="paint0_linear_148_156"
				x1="338.126"
				y1="229.465"
				x2="338.126"
				y2="914.861"
				gradientUnits="userSpaceOnUse"
			>
				<stop stop-color="#FAE1DD" />
				<stop offset="1" stop-color="#E9B8FF" />
			</linearGradient>
			<linearGradient
				id="paint1_linear_148_156"
				x1="630.302"
				y1="229.465"
				x2="630.302"
				y2="320.851"
				gradientUnits="userSpaceOnUse"
			>
				<stop stop-color="#FAE1DD" />
				<stop offset="1" stop-color="#E9B8FF" />
			</linearGradient>
			<linearGradient
				id="paint2_linear_148_156"
				x1="742.676"
				y1="1"
				x2="742.676"
				y2="320.851"
				gradientUnits="userSpaceOnUse"
			>
				<stop stop-color="#FAE1DD" />
				<stop offset="1" stop-color="#E9B8FF" />
			</linearGradient>
		</defs>
	</svg>
{/snippet}

{#snippet bombIcon()}
	<svg
		width="224"
		height="228"
		viewBox="0 0 224 228"
		fill="none"
		xmlns="http://www.w3.org/2000/svg"
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
{/snippet}

{#snippet bombWireIcon()}
	<svg
		width="112"
		height="114"
		viewBox="0 0 112 114"
		fill="none"
		xmlns="http://www.w3.org/2000/svg"
		filter="url(#noiseFilter)"
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
		</defs>
	</svg>
{/snippet}

<!-- {#snippet chatMessageSvg()}
	<svg width="44" height="44" viewBox="0 0 44 44" fill="none" xmlns="http://www.w3.org/2000/svg">
		<path
			d="M22.0002 38.5C25.118 38.4991 28.1708 37.6087 30.8002 35.9333L38.5002 38.5L35.9335 30.8C37.8832 27.7051 38.7485 24.0498 38.3932 20.4093C38.0379 16.7688 36.4822 13.3498 33.9709 10.6903C31.4596 8.03085 28.1353 6.28187 24.5211 5.71864C20.907 5.15542 17.2081 5.80992 14.0067 7.57914C10.8052 9.34836 8.28292 12.1319 6.83672 15.4916C5.39051 18.8514 5.10249 22.5966 6.01799 26.138C6.93349 29.6794 9.00054 32.8158 11.8938 35.0537C14.7871 37.2916 18.3424 38.504 22.0002 38.5ZM14.6669 15.5833H29.3335C29.8198 15.5833 30.2861 15.7765 30.6299 16.1203C30.9737 16.4641 31.1669 16.9304 31.1669 17.4167C31.1669 17.9029 30.9737 18.3692 30.6299 18.713C30.2861 19.0568 29.8198 19.25 29.3335 19.25H14.6669C14.1806 19.25 13.7143 19.0568 13.3705 18.713C13.0267 18.3692 12.8335 17.9029 12.8335 17.4167C12.8335 16.9304 13.0267 16.4641 13.3705 16.1203C13.7143 15.7765 14.1806 15.5833 14.6669 15.5833ZM14.6669 24.75H22.0002C22.4864 24.75 22.9528 24.9431 23.2966 25.287C23.6404 25.6308 23.8335 26.0971 23.8335 26.5833C23.8335 27.0695 23.6404 27.5359 23.2966 27.8797C22.9528 28.2235 22.4864 28.4166 22.0002 28.4166H14.6669C14.1806 28.4166 13.7143 28.2235 13.3705 27.8797C13.0267 27.5359 12.8335 26.5833 12.8335 26.5833C12.8335 26.5833 13.0267 25.6308 13.3705 25.287C13.7143 24.9431 14.1806 24.75 14.6669 24.75Z"
			fill="#161A15"
		/>
	</svg>
{/snippet}

<button class="fixed bottom-0 left-0 flex items-center justify-center bg-[#ACAAFF] p-1.5">
	{@render chatMessageSvg()}

	{#if unreadMessages !== 0}
		<div
			transition:scale
			class="absolute right-0 top-0 flex size-7 -translate-y-1/2 translate-x-1/2 items-center justify-center rounded-full bg-[#ED765E]"
		>
			<p>{unreadMessages > 99 ? '...' : unreadMessages}</p>
		</div>
	{/if}
</button>

{#if unreadMessages === -1}
	<button class="fixed bottom-0 left-0 flex items-center gap-2.5 bg-[#CBC9FC]/20 p-2.5 text-white">
		<img src={avatarImageSrc} alt="avatar" width="96" height="96" class="size-12" />
		<div class="text-left leading-snug">
			<p class="font-medium">skeary</p>
			<p class="max-w-64 truncate">{'Twifaohis is a decently short sentence'}</p>
		</div>
	</button>
{/if} -->
