<script lang="ts">
	import FilledDoubleRightArrow from '$lib/icons/FilledDoubleRightArrow.svelte';
	import HeartIcon from '$lib/icons/HeartIcon.svelte';
	import LogoFilled from '$lib/icons/LogoFilled.svelte';

	const {
		room,
		connection,
		onJoin
	}: {
		room: string;
		connection: 'awaiting' | 'connecting' | 'connected';
		onJoin: (username: string) => void;
	} = $props();

	let username = $state('');
</script>

<form
	style="background: radial-gradient(at top left, var(--color-background) 0%, rgba(233, 184, 255, 0.1) 100%), var(--color-background); border-image: conic-gradient(from -112deg, rgba(121, 120, 150, 0.5), rgba(203, 201, 252, 1)) 1;"
	class="absolute left-1/2 top-28 flex h-64 max-w-xl -translate-x-1/2 flex-col border"
	onsubmit={(e) => {
		e.preventDefault();

		if (1 <= username.length && username.length <= 32) {
			onJoin(username);
		}
	}}
>
	<div class="bg-pink absolute left-0 top-0 w-min text-nowrap rounded-br-2xl px-3 py-1 text-black">
		<p>Joining Game <code>'{room}'</code></p>
	</div>
	<div class="flex flex-1 items-center">
		<div class="flex flex-1 flex-col justify-center p-4">
			<label for="username" class="mb-1.5 block text-sm font-medium">Username</label>
			<input
				type="text"
				id="username"
				name="username"
				class="bg-background border-green focus:ring-pasteborder-green focus:border-green shadow-xs border px-3 py-2.5 text-sm"
				placeholder="wordplayer"
				required
				minlength="1"
				maxlength="32"
				disabled={connection !== 'awaiting'}
				bind:value={username}
			/>
		</div>
		<button
			style="clip-path: polygon(35% 0, 100% 0, 100% 100%, 0% 100%);"
			class="bg-pink relative h-full pl-12 pr-6"
		>
			<p class="text-background bg-pink font-serif text-2xl">Enter!</p>
			<FilledDoubleRightArrow class="absolute bottom-2.5 right-2" />
		</button>
	</div>
	<div class="border-pastel-pink/20 bg-pastel-pink/5 overflow-hidden border-t font-mono">
		<div class="marquee -mb-1 py-0.5">
			{#each { length: 2 }}
				<p class="text-nowrap pl-32 text-sm text-white/75">
					The room is currently in a game of Word Bomb with 4 players for 3:39 minutes with 95 words
					used. Consider supporting Wordplay! <HeartIcon
						width={19.5}
						height={16.5}
						class="inline-block"
					/>
				</p>
			{/each}
		</div>
	</div>
	<LogoFilled
		class="absolute bottom-4 left-4 -translate-x-1/2 translate-y-1/2 rotate-3 scale-125"
	/>
</form>
