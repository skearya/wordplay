<script lang="ts">
	let { timerStart }: { timerStart: bigint } = $props();

	let countdown = $derived(10 - Math.floor((Date.now() - Number(timerStart)) / 1000));

	$effect(() => {
		let interval: number | undefined;

		const timeout = setTimeout(
			() => {
				interval = setInterval(() => {
					countdown -= 1;
				}, 1000);
			},
			(Date.now() - Number(timerStart)) % 1000
		);

		return () => {
			if (interval) {
				clearInterval(interval);
			}

			clearTimeout(timeout);
		};
	});
</script>

<p>{countdown} seconds left</p>
