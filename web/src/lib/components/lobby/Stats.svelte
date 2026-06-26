<script lang="ts">
	import type { PostGameInfo } from '@bindings/PostGameInfo';
	import type { Props } from '$lib/context';
	import grid2Svg from '$lib/assets/grid2.svg';
	import Avatar from '$lib/components/Avatar.svelte';
	import Crown from '$lib/icons/Crown.svelte';

	let { ctx, info }: Omit<Props<never>, 'state' | 'sendMsg'> & { info: PostGameInfo } = $props();

	const { winner, headlines, leaderboards } = stats();

	function stats(): {
		winner: string;
		headlines: Array<[name: string, value: string]>;
		leaderboards: Array<[name: string, values: Array<[uuid: string, value: string]>]>;
	} {
		switch (info.kind) {
			case 'wordBomb':
				return {
					winner: info.leaderboard[0],
					headlines: [
						['Minutes Elapsed', `${info.minsElapsed.toFixed(2)}`],
						['Words Used', `${info.wordsUsed}`]
					],
					leaderboards: [
						[
							'Fastest Guesses',
							info.fastestGuesses.map(([uuid, value]) => [uuid, `${value.toFixed(2)}sec`])
						],
						['Longest Words', info.longestWords],
						['Missed Prompts', info.missedPrompts]
					]
				};
			case 'anagrams':
				return {
					winner: info.leaderboard[0][0],
					headlines: [],
					leaderboards: [
						['Leaderboard', info.leaderboard.map(([uuid, value]) => [uuid, `${value}`])]
					]
				};
			default:
				info satisfies never;
				throw new Error();
		}
	}

	let currentIndex = $state(0);
</script>

<section class="flex h-full flex-col overflow-y-hidden border border-pink bg-pink/15">
	<div
		style={`background: url("${grid2Svg}"), var(--color-pink);`}
		class="background-scroll p-4 pt-16 text-black"
	>
		<div class="relative mb-2 inline-block rounded-full">
			<Crown class="absolute -top-4 -right-4" />
			<Avatar {ctx} uuid={winner} />
		</div>
		<p class="text-xl"><b>{ctx.clients[winner].username}</b> won the game!</p>
	</div>
	<div class="flex flex-1 flex-col overflow-y-hidden p-4">
		<div class="mb-4 flex items-center justify-between gap-x-8 px-8">
			{#each headlines as [title, value], i}
				<div class="text-center">
					<p class="font-serif text-4xl text-yellow">{value}</p>
					<p>{title}</p>
				</div>
				{#if i < headlines.length - 1}
					<div class="w-[1px] rotate-12 self-stretch bg-pastel-pink"></div>
				{/if}
			{/each}
		</div>
		<div class="flex flex-1 flex-col overflow-y-hidden">
			<div class="flex items-center -space-x-1">
				{#each leaderboards as [name], i}
					<button
						style={`z-index: ${i === currentIndex ? leaderboards.length : leaderboards.length - i};`}
						class={[
							i === currentIndex
								? 'border-pastel-pink bg-pastel-pink text-background'
								: 'border-pink bg-background',
							'rounded-t-xl border border-b-0 px-2.5 py-1.5 text-sm'
						]}
						onclick={() => (currentIndex = i)}
					>
						{name}
					</button>
				{/each}
			</div>
			<div
				style="scrollbar-width: thin;"
				class="flex-1 divide-y divide-pastel-pink/50 overflow-y-auto border border-pastel-pink"
			>
				{#each leaderboards[currentIndex][1] as [uuid, value], i}
					<div class="relative flex items-center space-x-2.5 p-2">
						<div
							class="absolute top-0 left-0 content-center rounded-br bg-pastel-pink px-1 py-0.5 text-center font-mono text-xs text-background tabular-nums"
						>
							<p>{i + 1}</p>
						</div>
						<Avatar size="sm" {ctx} {uuid} />
						<p class="text-sm">{ctx.clients[uuid].username}</p>
						<p class="ml-auto text-sm">{value}</p>
					</div>
				{/each}
			</div>
		</div>
	</div>
</section>
