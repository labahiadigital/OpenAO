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

  // Track the last anim we've re-based so we only do it once per slide.
  let lastRebasedAnimStart = 0;

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

    // ── Smooth camera / player slide ──────────────────────────────
    // While the player slides between tiles the logical pos has already
    // snapped to the destination.  We compute a *pixel* offset that
    // decreases linearly from −TILE_SIZE*delta → 0 over `durationMs`.
    //
    // CRITICAL: we use the Pixi ticker's own timestamp (monotonic,
    // aligned to rAF) rather than a raw `performance.now()` call so
    // the interpolation factor is perfectly in-phase with the frame
    // that will be presented.  This avoids micro-jitter caused by
    // `performance.now()` being sampled at an arbitrary point inside
    // the frame budget.
    const tickerNow = app.ticker.lastTime;   // ms since Pixi init, rAF-aligned
    let camOffsetPx = 0;
    let camOffsetPy = 0;
    let isAnimating = false;
    const anim = gameState.playerMoveAnim;
    if (anim) {
      // Re-base the animation start from performance.now() space into
      // Pixi-ticker space the first time we see a new anim.  This is a
      // one-time conversion: GameView records `performance.now()` but
      // the render loop uses `app.ticker.lastTime`.
      if (anim.startedAt !== lastRebasedAnimStart) {
        const perfNow = performance.now();
        const drift = perfNow - anim.startedAt;      // how long ago it started
        anim.startedAt = tickerNow - drift;           // convert to ticker space
        lastRebasedAnimStart = anim.startedAt;
      }
      const elapsed = tickerNow - anim.startedAt;
      const t = Math.min(1, elapsed / anim.durationMs); // 0→1
      camOffsetPx = -TILE_SIZE * anim.dx * (1 - t);
      camOffsetPy = -TILE_SIZE * anim.dy * (1 - t);
      isAnimating = t < 1;

      // Do NOT null-out playerMoveAnim when t >= 1.
      // If the player is walking continuously, the next doMove() call
      // will overwrite this anim with the next step.  If we null it
      // here, there is a 1-16ms window (setTimeout jitter) where no
      // anim is active → offset snaps to 0 → visible micro-freeze.
      // When t >= 1 the offset is already 0, so keeping the stale
      // anim is harmless.  It will be replaced or naturally expire
      // once the player stops and a grace period (WALK_STEP_MS * 1.5)
      // passes — see below.
      if (t >= 1 && (tickerNow - anim.startedAt) > anim.durationMs * 1.5) {
        // Player has been idle for >50% longer than a step duration.
        // Safe to clean up.
        gameState.playerMoveAnim = null;
      }
    }

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

    // Single source of truth for the player container world position.
    // NO rounding — must use the exact same float precision as the camera
    // so the player is always perfectly centred on screen with zero
    // oscillation.  If we Math.round here but the camera uses floats
    // (or vice-versa), the ±0.5px discrepancy flips each frame and
    // the entire world jitters relative to the character.
    if (entityState.playerContainer) {
      entityState.playerContainer.x = (px - 1) * TILE_SIZE + camOffsetPx;
      entityState.playerContainer.y = (py - 1) * TILE_SIZE + camOffsetPy;
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

      // resolution: 1 keeps a 1:1 mapping between canvas CSS pixels and
      // back-buffer pixels.  This avoids the double-rounding trap where
      // roundPixels snaps sprite-local coords to integers but a non-1x
      // resolution then multiplies them by a fractional DPR, re-introducing
      // sub-pixel offsets and causing tile-edge shimmer.
      //
      // roundPixels: false — we deliberately allow sub-pixel positioning
      // so the camera can scroll at non-integer speeds without the 2px/3px
      // temporal aliasing that Math.round causes.  Tile edges stay crisp
      // thanks to scaleMode "nearest" set below.
      await app.init({
        resizeTo: container,
        background: "#0a0f0a",
        antialias: false,
        resolution: 1,
        autoDensity: false,
        preference: "webgpu",
        roundPixels: false,
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
