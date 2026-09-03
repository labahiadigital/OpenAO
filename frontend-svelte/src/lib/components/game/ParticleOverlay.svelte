<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { particleEngine } from "$lib/game/rendering/particleSystem";

  let canvas = $state<HTMLCanvasElement>();

  function resize() {
    if (!canvas) return;
    canvas.width = canvas.clientWidth;
    canvas.height = canvas.clientHeight;
  }

  onMount(() => {
    if (canvas) {
      resize();
      particleEngine.attach(canvas);
      window.addEventListener("resize", resize);
    }
  });

  onDestroy(() => {
    particleEngine.detach();
    window.removeEventListener("resize", resize);
  });
</script>

<canvas
  bind:this={canvas}
  class="absolute inset-0 w-full h-full pointer-events-none z-20"
></canvas>
