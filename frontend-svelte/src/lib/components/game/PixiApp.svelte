<script lang="ts">
  import { onMount } from "svelte";
  import { gameState } from "$lib/game/state/gameState.svelte";
  import { assetStore } from "$lib/game/state/assetStore.svelte";
  import { mapState } from "$lib/game/state/mapState.svelte";
  import { gameSession } from "$lib/game/session/gameSession.svelte";
  import { TILE_SIZE } from "$lib/game/lib/viewport";
  import { pollMovement } from "$lib/game/input/movementPoller";
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
    resetCamera,
    getCameraPosition,
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
      // Teleport the LERP camera to the new position immediately so it
      // doesn't slowly pan from the old map's coordinates.
      resetCamera(
        (px - 1) * TILE_SIZE + TILE_SIZE / 2,
        (py - 1) * TILE_SIZE + TILE_SIZE / 2,
      );
    }

    // ── Sub-tile interpolation offset ─────────────────────────────
    // The logical pos (px,py) has already snapped to the destination
    // tile.  We compute a pixel offset that slides from −TILE_SIZE →0
    // over `durationMs`, giving the visual "glide" between tiles.
    let camOffsetPx = 0;
    let camOffsetPy = 0;
    let isAnimating = false;
    const anim = gameState.playerMoveAnim;
    if (anim) {
      const elapsed = performance.now() - anim.startedAt;
      const t = Math.min(1, elapsed / anim.durationMs); // 0→1
      camOffsetPx = -TILE_SIZE * anim.dx * (1 - t);
      camOffsetPy = -TILE_SIZE * anim.dy * (1 - t);
      isAnimating = t < 1;
      // Don't null the anim at t>=1: the next doMove() will replace it.
      // This prevents a 1-16ms gap (setTimeout jitter) where offset
      // snaps to 0 → visible micro-freeze between walk steps.
    }

    // ── Camera: smooth LERP follow ─────────────────────────────────
    // updateCamera internally LERPs toward the target, absorbing any
    // micro-irregularities from the step timer.  Math.floor is applied
    // inside to the *amortised* position, guaranteeing integer-pixel
    // tile boundaries → no nearest-neighbour shimmer.
    updateCamera(app, worldContainer, px, py, camOffsetPx, camOffsetPy);
    const bounds = computeViewBounds(app, px, py, mp.w, mp.h);

    for (let y = bounds.minY; y <= bounds.maxY; y++) {
      for (let x = bounds.minX; x <= bounds.maxX; x++) {
        buildTile(tileState, mp, x, y, { groundLayer, belowLayer, aboveLayer, roofLayer }, createSprite);
      }
    }

    updateRoofVisibility(tileState, mp, px, py);
    updateTreeTransparency(tileState, px, py);

    renderPlayer(PIXI, entityState, gameState.hud, px, py, createSprite);

    // Position the player container in world space.
    // We use the raw target position (not the LERP'd camera), but
    // Math.floor it so the player sprite sits on integer pixels.
    // The LERP camera is close enough that the ±0.5px difference is
    // invisible — and both values are floor'd to integers, so they
    // never oscillate relative to each other.
    if (entityState.playerContainer) {
      entityState.playerContainer.x = Math.floor((px - 1) * TILE_SIZE + camOffsetPx);
      entityState.playerContainer.y = Math.floor((py - 1) * TILE_SIZE + camOffsetPy);
    }

    // Keep the player marked as "moving" while sliding so the walk cycle
    // plays the full animation instead of snapping back to idle.
    if (isAnimating) {
      entityState.playerLastMoveTime = performance.now();
    }

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

      // Match the device's physical pixel density so the browser does not
      // resample the canvas via CSS scaling (which causes blur / pixel-crawl
      // on Windows with 125%/150% display scaling).
      // roundPixels: true snaps every sprite draw to the nearest integer
      // pixel *within the back-buffer*, preventing sub-pixel shimmer on
      // tile edges with nearest-neighbour filtering.
      // The LERP camera ensures the worldContainer.x/y values fed to
      // Pixi are already smoothly varying integers (via Math.floor),
      // so roundPixels won't cause 2px/3px alternation.
      const dpr = window.devicePixelRatio || 1;
      await app.init({
        resizeTo: container,
        background: "#0a0f0a",
        antialias: false,
        resolution: dpr,
        autoDensity: true,
        preference: "webgpu",
        roundPixels: true,
      });
      if (destroyed) { app.destroy(true); app = undefined; return; }

      const canvas = app.canvas as HTMLCanvasElement;
      // Pixel-art rendering: prevent the browser from applying bilinear
      // filtering when the CSS size differs from the back-buffer size.
      canvas.style.imageRendering = "pixelated";
      container.appendChild(canvas);

      // Default texture sampling = nearest-neighbour for crisp pixel art.
      PIXI.TextureStyle.defaultOptions.scaleMode = "nearest";

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
          // Poll movement inputs every frame — decoupled from OS key repeat.
          pollMovement();
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
