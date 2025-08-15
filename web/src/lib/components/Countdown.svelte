<script lang="ts">
	const { timerStart }: { timerStart: bigint } = $props();

	let countdown = $derived(Date.now() - Number(timerStart));

	$effect(() => {
		let interval: number | undefined;

		const timeout = setTimeout(() => {
			setInterval(() => {
				countdown -= 1;
			}, 1000);
		}, countdown);

		return () => {
			if (interval) {
				clearInterval(interval);
			}

			clearTimeout(timeout);
		};
	});
</script>
