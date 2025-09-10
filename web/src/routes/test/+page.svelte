<script lang="ts">
	import type { PageProps } from './$types';
	import { fade, scale, slide } from 'svelte/transition';
	import crownSvg from '$lib/assets/crown.svg';
	import grid2Svg from '$lib/assets/grid2.svg';
	import slashSvg from '$lib/assets/slash.svg';

	let { data }: PageProps = $props();

	let unreadMessages = $state(0);

	$effect(() => {
		setInterval(() => {
			unreadMessages += 1;
		}, 2000);
	});
</script>

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

<main
	style="background: linear-gradient(180deg, rgba(0, 0, 0, 0) 0%, rgba(246, 245, 180, 0.08) 100%), #040605"
	class="flex h-screen flex-col"
>
	<nav class="flex items-center justify-between p-6">
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
	<div class="flex h-full items-stretch gap-4 p-4 pt-0">
		<section class="flex-[30%] border border-[#F3ACFF] bg-[#F3ACFF]/15">
			<div
				style={`background: url("${grid2Svg}"), #F3ACFF;`}
				class="background-scroll p-4 pt-16 text-black"
			>
				<div class="relative mb-2.5 size-20">
					<img
						src="https://avatar.vercel.sh/s"
						alt="avatar"
						width="120"
						height="120"
						class="h-full w-full rounded-full"
					/>
					<img src={crownSvg} alt="crown" class="absolute -right-4 -top-4" />
				</div>
				<h1 class="text-xl"><b>skeary</b> won the game!</h1>
			</div>
			<div class="p-4">
				<div class="mb-4 flex items-center justify-between gap-x-8 px-8">
					<div class="text-center">
						<h1 style="font-family: 'PP Editorial New';" class="text-4xl text-[#F6F5B4]">21</h1>
						<p>Minutes Elapsed</p>
					</div>
					<div class="w-[1px] rotate-12 self-stretch bg-[#E9B8FF]"></div>
					<div class="text-center">
						<h1 style="font-family: 'PP Editorial New';" class="text-4xl text-[#F6F5B4]">21</h1>
						<p>Words Used</p>
					</div>
				</div>
				<div>
					<div class="flex items-center -space-x-1 text-nowrap">
						{#each ['Fastest Guess', 'Longest Word', 'Word Length', 'WPM'] as text, i}
							<button
								style={`z-index: ${3 - i};`}
								class={[
									i === 0
										? 'border-[#E9B8FF] bg-[#E9B8FF] text-black'
										: 'border-[#F3ACFF] bg-[#2B212C]',
									'rounded-t-xl border border-b-0 px-2.5 py-1 text-sm'
								]}
							>
								{text}
							</button>
						{/each}
					</div>
					<div class="max-h-64 divide-y overflow-y-auto border border-[#E9B8FF]">
						{#each { length: 99 } as _, i}
							<div class="relative flex items-center space-x-2.5 border-[#E9B8FF] p-2.5">
								<div
									class="absolute left-0 top-0 size-4 content-center bg-[#E9B8FF] text-center text-xs text-black"
								>
									<p>{i + 1}</p>
								</div>
								<img
									src="https://avatar.vercel.sh/s"
									alt="avatar"
									width="120"
									height="120"
									class="size-12 rounded-full"
								/>
								<p>skeary</p>
								<p class="ml-auto">999ms</p>
							</div>
						{/each}
					</div>
				</div>
			</div>
		</section>
		<section class="flex-[70%] border border-[#475D50] bg-[#475D50]/15">w</section>
	</div>
</main>

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
