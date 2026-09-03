<script lang="ts">
  import type { GraphicData } from "$lib/game/types/game";

  let { graphicData, name, size = 32 }: { graphicData?: GraphicData | null; name: string; size?: number } = $props();

  let scale = $derived(
    graphicData ? Math.min(1, (size - 4) / Math.max(graphicData.width, graphicData.height, 1)) : 1,
  );
</script>

{#if graphicData?.numFile}
  <div
    class="relative overflow-hidden rounded-sm"
    style="width: {size}px; height: {size}px;"
  >
    <div
      aria-label={name}
      class="absolute left-1/2 top-1/2 bg-no-repeat"
      style="
        width: {graphicData.width}px;
        height: {graphicData.height}px;
        background-image: url('/graphics/{graphicData.numFile}.png');
        background-position: -{graphicData.sX}px -{graphicData.sY}px;
        transform: translate(-50%, -50%) scale({scale});
        transform-origin: center;
      "
    ></div>
  </div>
{:else}
  <div
    class="rounded-md bg-black/20"
    style="width: {size}px; height: {size}px;"
  ></div>
{/if}
