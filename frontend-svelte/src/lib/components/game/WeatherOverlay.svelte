<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { weatherSystem } from '$lib/game/rendering/weatherSystem';
	import { dayNightCycle } from '$lib/game/rendering/dayNightCycle';

	let weatherCanvas: HTMLCanvasElement;
	let dayNightCanvas: HTMLCanvasElement;

	onMount(() => {
		weatherSystem.attach(weatherCanvas);
		weatherSystem.start();

		dayNightCycle.attach(dayNightCanvas);
		dayNightCycle.start();

		const resize = () => {
			const w = window.innerWidth;
			const h = window.innerHeight;
			weatherCanvas.width = w;
			weatherCanvas.height = h;
			dayNightCanvas.width = w;
			dayNightCanvas.height = h;
		};
		resize();
		window.addEventListener('resize', resize);

		return () => {
			window.removeEventListener('resize', resize);
		};
	});

	onDestroy(() => {
		weatherSystem.destroy();
		dayNightCycle.destroy();
	});
</script>

<canvas
	bind:this={dayNightCanvas}
	class="pointer-events-none fixed inset-0 z-30"
	style="mix-blend-mode: multiply;"
></canvas>
<canvas
	bind:this={weatherCanvas}
	class="pointer-events-none fixed inset-0 z-31"
></canvas>
