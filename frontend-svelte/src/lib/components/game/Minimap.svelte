<script lang="ts">
  import { gameState } from "$lib/game/state/gameState.svelte";
  import { mapState } from "$lib/game/state/mapState.svelte";
  import { getTileAt } from "$lib/game/engine/assetLoader";

  const MAP_SIZE = 140;
  const BORDER = 1;
  const PLAYER_DOT = 3;
  const ENTITY_DOT = 2;

  let canvas = $state<HTMLCanvasElement>();
  let ctx: CanvasRenderingContext2D | null = null;
  let collapsed = $state(false);
  let rafId: number | undefined;

  const TILE_COLORS: Record<string, string> = {
    water: "#1e3a5f",
    grass: "#1a3a1a",
    dirt: "#3b2f1e",
    stone: "#3a3a3a",
    sand: "#5a4b30",
  };

  function classifyTile(layer1Grh: number): string {
    if (layer1Grh >= 1505 && layer1Grh <= 1568) return "water";
    if (layer1Grh >= 7000 && layer1Grh <= 7100) return "water";
    if (layer1Grh >= 681 && layer1Grh <= 744) return "water";
    if (layer1Grh >= 1 && layer1Grh <= 6) return "grass";
    if (layer1Grh >= 200 && layer1Grh <= 300) return "dirt";
    if (layer1Grh >= 400 && layer1Grh <= 500) return "stone";
    return "grass";
  }

  function draw() {
    if (!ctx || !canvas) return;
    const mp = mapState.mapParsed;
    if (!mp) {
      ctx.fillStyle = "#111";
      ctx.fillRect(0, 0, MAP_SIZE, MAP_SIZE);
      return;
    }

    const { x: px, y: py } = gameState.hud.pos;
    if (px <= 0 || py <= 0) return;

    const mapW = mp.w || 100;
    const mapH = mp.h || 100;
    const scaleX = MAP_SIZE / mapW;
    const scaleY = MAP_SIZE / mapH;

    ctx.fillStyle = "#0a0f0a";
    ctx.fillRect(0, 0, MAP_SIZE, MAP_SIZE);

    const step = Math.max(1, Math.floor(1 / Math.min(scaleX, scaleY)));
    for (let y = 1; y <= mapH; y += step) {
      for (let x = 1; x <= mapW; x += step) {
        const tileData = getTileAt(mp, x, y);
        let grh = 0;
        if (tileData) {
          grh = tileData.graphics["1"] ?? 0;
        }
        const cls = classifyTile(grh);
        ctx.fillStyle = TILE_COLORS[cls] || TILE_COLORS["grass"]!;

        const drawW = Math.max(1, Math.ceil(step * scaleX));
        const drawH = Math.max(1, Math.ceil(step * scaleY));
        ctx.fillRect(
          Math.floor((x - 1) * scaleX),
          Math.floor((y - 1) * scaleY),
          drawW,
          drawH,
        );

        if (tileData?.blocked) {
          ctx.fillStyle = "rgba(0,0,0,0.3)";
          ctx.fillRect(
            Math.floor((x - 1) * scaleX),
            Math.floor((y - 1) * scaleY),
            drawW,
            drawH,
          );
        }
      }
    }

    for (const [, npc] of gameState.remoteNpcs) {
      if (npc.dead) continue;
      const nx = Math.floor((npc.x - 1) * scaleX);
      const ny = Math.floor((npc.y - 1) * scaleY);
      ctx.fillStyle = "#ef4444";
      ctx.fillRect(nx, ny, ENTITY_DOT, ENTITY_DOT);
    }

    for (const [, e] of gameState.remoteEntities) {
      if (e.dead) continue;
      const ex = Math.floor((e.x - 1) * scaleX);
      const ey = Math.floor((e.y - 1) * scaleY);
      ctx.fillStyle = "#60a5fa";
      ctx.fillRect(ex, ey, ENTITY_DOT, ENTITY_DOT);
    }

    const ppx = Math.floor((px - 1) * scaleX);
    const ppy = Math.floor((py - 1) * scaleY);
    ctx.fillStyle = "#fbbf24";
    ctx.beginPath();
    ctx.arc(ppx + PLAYER_DOT / 2, ppy + PLAYER_DOT / 2, PLAYER_DOT, 0, Math.PI * 2);
    ctx.fill();

    ctx.strokeStyle = "rgba(255,255,255,0.15)";
    ctx.lineWidth = 1;
    ctx.strokeRect(0, 0, MAP_SIZE, MAP_SIZE);
  }

  function loop() {
    draw();
    rafId = requestAnimationFrame(loop);
  }

  $effect(() => {
    if (canvas) {
      ctx = canvas.getContext("2d");
      if (rafId === undefined) {
        loop();
      }
      return () => {
        if (rafId !== undefined) {
          cancelAnimationFrame(rafId);
          rafId = undefined;
        }
        ctx = null;
      };
    }
  });
</script>

<div
  class="absolute bottom-2 right-2 z-30 select-none"
  style="width: {collapsed ? 28 : MAP_SIZE + BORDER * 2}px"
>
  <button
    onclick={() => { collapsed = !collapsed; }}
    class="absolute -top-5 right-0 text-[9px] text-stone-500 hover:text-stone-300 transition cursor-pointer bg-black/50 rounded px-1"
    title={collapsed ? "Mostrar minimapa" : "Ocultar minimapa"}
  >
    {collapsed ? "▢" : "▽"}
  </button>

  {#if !collapsed}
    <div class="rounded border border-white/10 bg-black/70 backdrop-blur-sm overflow-hidden shadow-lg" style="width: {MAP_SIZE + BORDER * 2}px; height: {MAP_SIZE + BORDER * 2}px">
      <canvas
        bind:this={canvas}
        width={MAP_SIZE}
        height={MAP_SIZE}
        class="block"
        style="width: {MAP_SIZE}px; height: {MAP_SIZE}px"
      ></canvas>
    </div>
    <div class="mt-0.5 flex items-center justify-center gap-3 text-[8px] text-stone-500">
      <span class="flex items-center gap-1"><span class="inline-block w-2 h-2 rounded-full bg-amber-400"></span>Tu</span>
      <span class="flex items-center gap-1"><span class="inline-block w-2 h-2 bg-blue-400"></span>Jugador</span>
      <span class="flex items-center gap-1"><span class="inline-block w-2 h-2 bg-red-500"></span>NPC</span>
    </div>
  {/if}
</div>
