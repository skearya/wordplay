<script>
	import GreenSettings from '$lib/icons/GreenSettings.svelte';
	import Logo from '$lib/icons/Logo.svelte';
</script>

<nav class="flex items-center justify-between px-5 py-4">
	<Logo />
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
				class="border-dark-green size-12 content-center rounded-full border-2 bg-black text-center"
			>
				+2
			</div>
		</div>
		<GreenSettings />
	</div>
</nav>
