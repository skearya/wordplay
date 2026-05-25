<script lang="ts">
	import grid2Svg from '$lib/assets/grid2.svg';
	import Crown from '$lib/icons/Crown.svelte';
</script>

<section class="flex h-full max-w-md flex-col overflow-y-hidden border border-pink bg-pink/15">
	<div
		style={`background: url("${grid2Svg}"), var(--color-pink);`}
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
			<Crown class="absolute -top-4 -right-4" />
		</div>
		<h1 class="text-xl"><b>skeary</b> won the game!</h1>
	</div>
	<div class="flex flex-1 flex-col overflow-y-hidden p-4">
		<div class="mb-4 flex items-center justify-between gap-x-8 px-8">
			<div class="text-center">
				<h1 class="font-serif text-4xl text-yellow">21</h1>
				<p>Minutes Elapsed</p>
			</div>
			<div class="w-[1px] rotate-12 self-stretch bg-pastel-pink"></div>
			<div class="text-center">
				<h1 class="font-serif text-4xl text-yellow">21</h1>
				<p>Words Used</p>
			</div>
		</div>
		<div class="flex flex-1 flex-col overflow-y-hidden">
			<div class="flex items-center -space-x-1 text-nowrap">
				{#each ['Fastest Guess', 'Longest Word', 'Word Length', 'WPM'] as text, i}
					<button
						style={`z-index: ${4 - i};`}
						class={[
							i === 0 ? 'border-pastel-pink bg-pastel-pink text-black' : 'border-pink bg-[#2B212C]',
							'rounded-t-xl border border-b-0 px-2.5 py-1 text-sm'
						]}
					>
						{text}
					</button>
				{/each}
			</div>
			<div
				style="scrollbar-width: thin;"
				class="flex-1 divide-y overflow-y-auto border border-pastel-pink"
			>
				{#each { length: 99 }, i}
					<div class="relative flex items-center space-x-2.5 border-pastel-pink p-2.5">
						<div
							class="absolute top-0 left-0 content-center bg-pastel-pink px-1 py-0.5 text-center text-xs text-black"
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
