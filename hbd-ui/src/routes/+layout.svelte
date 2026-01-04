<script lang="ts">
	import '../app.css';
	import { Tooltip } from 'bits-ui';

	let { children } = $props();
</script>

<svelte:head>
	<title>hbd-ui | Task Graph Visualizer</title>
</svelte:head>

<Tooltip.Provider delayDuration={200}>
	{@render children()}
</Tooltip.Provider>
