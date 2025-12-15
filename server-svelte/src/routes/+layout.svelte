<script lang="ts">
	import "../app.css";

	let { children } = $props();
</script>

<svelte:head>
	<title>Driftwatch - Guard Your Performance</title>
	<meta name="description" content="Automatically track and visualize your Criterion benchmarks. Get alerted when performance regresses." />
</svelte:head>

{@render children()}
