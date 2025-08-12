<script lang="ts">
	import type { PageProps } from './$types';
	import { onMount } from 'svelte';
	import { greet } from 'state';

	let { data }: PageProps = $props();

	onMount(() => {
		greet('someone');
	});
</script>
