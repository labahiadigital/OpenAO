<script lang="ts">
  import { onMount } from "svelte";
  import { gameState } from "$lib/game/state/gameState.svelte";
  import { assetStore } from "$lib/game/state/assetStore.svelte";
  import { mapState } from "$lib/game/state/mapState.svelte";
  import { gameSession } from "$lib/game/session/gameSession.svelte";
  import { TILE_SIZE } from "$lib/game/lib/viewport";
  import type { GraphicInfo } from "$lib/game/engine/assetLoader";
  import type {
    Application,
    Container as PixiContainer,
    Sprite as PixiSprite,
  } from "pixi.js";
  import {
    type TileState,
    type TextureSourceCache,
    createTileState,
    clearTileState,
    buildTile,
    updateRoofVisibility,
    updateTreeTransparency,
    cullOffscreenTiles,
    createSpriteFromInfo,
    getTextureSource,
  } from "$lib/game/rendering/tileRenderer";
  import {
    type EntityContainerState,
    createEntityContainerState,
    clearEntityState,
    renderRemoteEntities,
    renderNpcs,
    renderPlayer,
    animateEntitySprites,
    setGetTextureSourceFn,
  } from "$lib/game/rendering/entityRenderer";
  import {
    type GroundItemContainerState,
    createGroundItemState,
    clearGroundItemState,
    renderGroundItems,
  } from "$lib/game/rendering/groundItemRenderer";
  import {
    updateCamera,
    computeViewBounds,
    handleCanvasClick,
  } from "$lib/game/rendering/cameraController";

  let container = $state<HTMLDivElement>();
  let app: Application | undefined;
  let worldContainer: PixiContainer | undefined;
  let groundLayer: PixiContainer | undefined;
  let belowLayer: PixiContainer | undefined;
  let aboveLayer: PixiContainer | undefined;
  let entityLayer: PixiContainer | undefined;
  let roofLayer: PixiContainer | undefined;
  let PIXI: typeof import("pixi.js") | undefined;

  let tileState: TileState = createTileState();
  let entityState: EntityContainerState = createEntityContainerState();
  let groundItemState: GroundItemContainerState = createGroundItemState();
  let initError = $state<string | null>(null);
  let lastRenderedMapId = 0;

  const textureSourceCache: TextureSourceCache = new Map();
  const pendingLoads = new Set<number>();

  function createSprite(info: GraphicInfo): PixiSprite | null {
    if (!PIXI) return null;
    return createSpriteFromInfo(PIXI, textureSourceCache, pendingLoads, info);
  }

  function clearAll() {
    clearTileState(tileState, groundLayer, belowLayer, aboveLayer, roofLayer);
    clearEntityState(entityState);
    clearGroundItemState(groundItemState);
  }

  function renderScene() {
    if (!PIXI || !worldContainer || !app || !groundLayer || !belowLayer || !aboveLayer || !roofLayer || !entityLayer) return;
    const mp = mapState.mapParsed;
    if (!mp) return;

    const { x: px, y: py } = gameState.hud.pos;
    if (px <= 0 || py <= 0) return;

    if (mapState.currentMapId !== lastRenderedMapId) {
      clearAll();
      lastRenderedMapId = mapState.currentMapId;
    }

    updateCamera(app, worldContainer, px, py);
    const bounds = computeViewBounds(app, px, py, mp.w, mp.h);

    for (let y = bounds.minY; y <= bounds.maxY; y++) {
      for (let x = bounds.minX; x <= bounds.maxX; x++) {
        buildTile(tileState, mp, x, y, { groundLayer, belowLayer, aboveLayer, roofLayer }, createSprite);
      }
    }

    updateRoofVisibility(tileState, mp, px, py);
    updateTreeTransparency(tileState, px, py);

    renderPlayer(PIXI, entityState, gameState.hud, px, py, createSprite);
    renderRemoteEntities(
      PIXI, entityLayer, entityState,
      gameState.remoteEntities,
      gameState.interpolationBuffers,
      gameSession.tickSync.estimatedServerTick,
      createSprite,
    );
    renderNpcs(
      PIXI, entityLayer, entityState,
      gameState.remoteNpcs,
      gameState.interpolationBuffers,
      gameSession.tickSync.estimatedServerTick,
      createSprite,
    );
    renderGroundItems(PIXI, entityLayer, groundItemState, gameState.groundItems, createSprite);

    animateEntitySprites(PIXI, entityState, textureSourceCache, pendingLoads, performance.now());

    cullOffscreenTiles(tileState, px, py, bounds.viewW, bounds.viewH);
  }

  function onCanvasClick(e: MouseEvent) {
    if (!app) return;
    handleCanvasClick(app, gameState.hud.pos.x, gameState.hud.pos.y, e);
  }

  $effect(() => {
    if (!app) return;
    const canvas = app.canvas as HTMLCanvasElement;
    canvas.style.cursor = gameState.pendingSpellSlot !== null ? "crosshair" : "default";
  });

  onMount(() => {
    let destroyed = false;

    (async () => {
      PIXI = await import("pixi.js");
      if (destroyed || !container) return;
      app = new PIXI.Application();
      await app.init({
        resizeTo: container,
        background: "#0a0f0a",
        antialias: false,
        resolution: 1,
        autoDensity: false,
        preference: "webgpu",
      });
      if (destroyed) { app.destroy(true); app = undefined; return; }

      const canvas = app.canvas as HTMLCanvasElement;
      container.appendChild(canvas);

      worldContainer = new PIXI.Container();
      app.stage.addChild(worldContainer);
      groundLayer = new PIXI.Container(); worldContainer.addChild(groundLayer);
      belowLayer = new PIXI.Container(); worldContainer.addChild(belowLayer);
      aboveLayer = new PIXI.Container(); aboveLayer.sortableChildren = true; worldContainer.addChild(aboveLayer);
      entityLayer = new PIXI.Container(); entityLayer.sortableChildren = true; worldContainer.addChild(entityLayer);
      roofLayer = new PIXI.Container(); roofLayer.sortableChildren = true; worldContainer.addChild(roofLayer);

      entityState.playerContainer = new PIXI.Container();
      entityState.playerContainer.sortableChildren = true;
      entityLayer.addChild(entityState.playerContainer);

      canvas.addEventListener("click", onCanvasClick);

      setGetTextureSourceFn((pixiRef: any, cache: any, pending: any, fileNum: number) => {
        return getTextureSource(pixiRef, cache, pending, fileNum);
      });

      app.ticker.add(() => {
        try {
          const mapId = gameState.hud.map;
          if (mapId > 0 && (mapId !== mapState.currentMapId) && !mapState.loading) {
            mapState.load(mapId);
          }
          if (mapState.mapParsed) renderScene();
        } catch (err) { console.error("[PixiApp] tick error:", err); }
      });
    })().catch((err) => {
      console.error("[PixiApp] init error:", err);
      initError = err instanceof Error ? err.message : String(err);
    });

    return () => {
      destroyed = true;
      clearAll();
      textureSourceCache.clear();
      app?.destroy(true);
      app = undefined;
    };
  });
</script>

{#if initError}
  <div class="absolute inset-0 z-50 flex items-center justify-center bg-black text-red-400 p-4">
    <p>Error: {initError}</p>
  </div>
{/if}
<div bind:this={container} class="absolute inset-0 overflow-hidden bg-black"></div>
