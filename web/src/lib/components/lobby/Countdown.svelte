<script lang="ts">
	let { timerStart }: { timerStart: number } = $props();

	let countdown = $derived(10 - Math.floor((Date.now() - timerStart) / 1000));

	$effect(() => {
		let interval: number | undefined;

		const timeout = setTimeout(
			() => {
				interval = setInterval(() => {
					countdown -= 1;
				}, 1000);
			},
			(Date.now() - timerStart) % 1000
		);

		return () => {
			if (interval) {
				clearInterval(interval);
			}

			clearTimeout(timeout);
		};
	});
</script>

{countdown} seconds left
