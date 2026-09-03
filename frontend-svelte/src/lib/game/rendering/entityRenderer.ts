import type {
  Container as PixiContainer,
  Sprite as PixiSprite,
} from "pixi.js";
import { getGraphicInfo, getAnimationInfo, type GraphicInfo } from "$lib/game/engine/assetLoader";
import { assetStore } from "$lib/game/state/assetStore.svelte";
import { TILE_SIZE, type SpriteFactory } from "./tileRenderer";
import type { TextureSource as PixiTextureSource } from "pixi.js";

export type TextureSourceCache = Map<number, PixiTextureSource>;

/**
 * Animation constants ported from the original Engine.ts:
 * - BODY_ANIMATION_CYCLE_MS: One full walk cycle takes 400ms
 * - WALK_DURATION_MS: How long a single tile-step takes (~300ms)
 * Entities are considered "moving" for WALK_DURATION_MS after their
 * last MOVE_ENTITY packet.  While moving, body/head frames cycle
 * over BODY_ANIMATION_CYCLE_MS.
 */
const BODY_ANIMATION_CYCLE_MS = 400;
const WALK_DURATION_MS = 300;

export type EntityRenderState = {
  /** The base body/head IDs (heading-independent). Used to detect sprite rebuild. */
  bodyId: number;
  headId: number;
  bodyGrh: number;
  headGrh: number;
  heading: number;
  dead: boolean;
  nameColor: number;
  bodyAnimFrames?: GraphicInfo[];
  bodyFrameCount: number;
  headAnimFrames?: GraphicInfo[];
  headFrameCount: number;
  /** Timestamp when the current walk animation started (performance.now) */
  animStartTime: number;
  /** Last rendered body frame index (0-based) */
  lastFrameIdx: number;
  /** Whether entity was moving on the last render tick */
  wasMoving: boolean;
  /** Last known visual position — used to detect movement for animation */
  lastVisualX: number;
  lastVisualY: number;
};

const NAME_COLORS: Record<number, number> = {
  0: 0xffffff,
  1: 0xff4444,
  2: 0x3333ff,
  3: 0xff3333,
  4: 0x44bb00,
};

export function nameColor(e: { dead: boolean; nameColor?: number }): number {
  if (e.dead) return 0x991b1b;
  return NAME_COLORS[e.nameColor ?? 0] ?? 0xffffff;
}

function getBodySpritePosition(w: number, h: number) {
  return {
    x: 16 - Math.floor((w * 16) / 32),
    y: 32 - Math.floor((h * 32) / 32),
  };
}

export function buildEntityContainer(
  PIXI: typeof import("pixi.js"),
  bodyGrhId: number,
  headGrhId: number,
  bodyId: number,
  name: string,
  color: number,
  createSprite: SpriteFactory,
): { container: PixiContainer; hasRealSprite: boolean } {
  const c = new PIXI.Container();
  c.sortableChildren = true;
  let hasRealSprite = false;

  const bodyInfo = bodyGrhId > 0 ? getGraphicInfo(bodyGrhId) : null;
  if (bodyInfo) {
    const bodySprite = createSprite(bodyInfo);
    if (bodySprite) {
      hasRealSprite = true;
      const pos = getBodySpritePosition(bodyInfo.w, bodyInfo.h);
      bodySprite.x = pos.x;
      bodySprite.y = pos.y;
      bodySprite.zIndex = 0.2;
      (bodySprite as any)._isBody = true;
      c.addChild(bodySprite);

      if (headGrhId > 0) {
        const headInfo = getGraphicInfo(headGrhId);
        if (headInfo) {
          const headSprite = createSprite(headInfo);
          if (headSprite) {
            const bodyData =
              bodyId > 0 ? assetStore.getBodyData(bodyId) : null;
            const hx =
              pos.x +
              bodyInfo.w / 2 -
              headInfo.w / 2 +
              (bodyData?.headOffsetX ?? 0);
            const hy =
              pos.y +
              bodyInfo.h -
              50 +
              (bodyData?.headOffsetY ?? 0);
            headSprite.x = Math.round(hx);
            headSprite.y = Math.round(hy);
            headSprite.zIndex = 0.1;
            (headSprite as any)._isHead = true;
            c.addChild(headSprite);
          }
        }
      }
    }
  }

  if (!bodyInfo) {
    const g = new PIXI.Graphics();
    const r = TILE_SIZE * 0.3;
    g.circle(0, 0, r);
    g.fill({ color: 0xcccccc });
    c.addChild(g);
  }

  const t = new PIXI.Text({
    text: name,
    style: {
      fontSize: 11,
      fill: color,
      fontFamily: "sans-serif",
      fontWeight: "bold",
      dropShadow: { alpha: 0.9, angle: 0, blur: 2, color: 0x000000, distance: 0 },
    },
  });
  t.anchor.set(0.5, 0);
  t.zIndex = 0.6;
  const bodyInfo2 = bodyGrhId > 0 ? getGraphicInfo(bodyGrhId) : null;
  if (bodyInfo2 && hasRealSprite) {
    const pos = getBodySpritePosition(bodyInfo2.w, bodyInfo2.h);
    t.x = Math.round(pos.x + bodyInfo2.w / 2);
    t.y = Math.round(pos.y + bodyInfo2.h + 2);
  } else {
    t.y = TILE_SIZE * 0.5;
  }
  c.addChild(t);
  return { container: c, hasRealSprite };
}

export type EntityContainerState = {
  entityContainers: Map<number, PixiContainer>;
  entityRenderStates: Map<number, EntityRenderState>;
  npcContainers: Map<number, PixiContainer>;
  npcRenderStates: Map<number, EntityRenderState>;
  playerContainer: PixiContainer | undefined;
  playerRenderState: EntityRenderState | null;
  /** Tracks the last time each entity/npc moved (performance.now ms) */
  entityMoveTimes: Map<number, number>;
  /** Track when the local player last moved */
  playerLastMoveTime: number;
};

export function createEntityContainerState(): EntityContainerState {
  return {
    entityContainers: new Map(),
    entityRenderStates: new Map(),
    npcContainers: new Map(),
    npcRenderStates: new Map(),
    playerContainer: undefined,
    playerRenderState: null,
    entityMoveTimes: new Map(),
    playerLastMoveTime: 0,
  };
}

export function clearEntityState(state: EntityContainerState) {
  for (const c of state.entityContainers.values())
    c.destroy({ children: true });
  state.entityContainers.clear();
  state.entityRenderStates.clear();
  for (const c of state.npcContainers.values())
    c.destroy({ children: true });
  state.npcContainers.clear();
  state.npcRenderStates.clear();
  state.entityMoveTimes.clear();
}

function buildRenderState(bodyId: number, headId: number, bodyGrhId: number, headGrhId: number, heading: number, dead: boolean, nc: number): EntityRenderState {
  const bodyAnim = bodyGrhId > 0 ? getAnimationInfo(bodyGrhId) : null;
  const headAnim = headGrhId > 0 ? getAnimationInfo(headGrhId) : null;
  return {
    bodyId,
    headId,
    bodyGrh: bodyGrhId,
    headGrh: headGrhId,
    heading,
    dead,
    nameColor: nc,
    bodyAnimFrames: bodyAnim?.frames,
    bodyFrameCount: bodyAnim?.frameCount ?? 1,
    headAnimFrames: headAnim?.frames,
    headFrameCount: headAnim?.frameCount ?? 1,
    animStartTime: 0,
    lastFrameIdx: 0,
    wasMoving: false,
    lastVisualX: -1,
    lastVisualY: -1,
  };
}

/**
 * When the heading changes but the body/head IDs stay the same,
 * update only the animation frames (cheap) instead of destroying
 * and rebuilding the entire Pixi container (expensive).
 */
function updateRenderStateHeading(rs: EntityRenderState, bodyGrhId: number, headGrhId: number, heading: number) {
  rs.heading = heading;
  rs.bodyGrh = bodyGrhId;
  rs.headGrh = headGrhId;
  const bodyAnim = bodyGrhId > 0 ? getAnimationInfo(bodyGrhId) : null;
  const headAnim = headGrhId > 0 ? getAnimationInfo(headGrhId) : null;
  rs.bodyAnimFrames = bodyAnim?.frames;
  rs.bodyFrameCount = bodyAnim?.frameCount ?? 1;
  rs.headAnimFrames = headAnim?.frames;
  rs.headFrameCount = headAnim?.frameCount ?? 1;
  rs.lastFrameIdx = -1; // force frame refresh on next animate
}

export function renderRemoteEntities(
  PIXI: typeof import("pixi.js"),
  entityLayer: PixiContainer,
  state: EntityContainerState,
  remoteEntities: Map<number, any>,
  interpolationBuffers: Map<number, any>,
  estimatedServerTick: number,
  createSprite: SpriteFactory,
) {
  const active = new Set<number>();
  for (const [id, e] of remoteEntities) {
    active.add(id);
    const nc = nameColor(e);
    const eBodyId = e.bodyGrh ?? 0;
    const eHeadId = e.headGrh ?? 0;
    const bodyGrhId = eBodyId > 0
      ? assetStore.getBodyGrhId(eBodyId, e.heading)
      : 0;
    const headGrhId = eHeadId > 0
      ? assetStore.getHeadGrhId(eHeadId, e.heading)
      : 0;

    const prev = state.entityRenderStates.get(id);
    // Only rebuild the container when the base identity changes (body/head
    // ID, dead state, name color).  A mere heading change is handled by
    // swapping animation frames — orders of magnitude cheaper.
    const needsRebuild =
      !prev ||
      prev.bodyId !== eBodyId ||
      prev.headId !== eHeadId ||
      prev.dead !== e.dead ||
      prev.nameColor !== nc;

    let c = state.entityContainers.get(id);
    if (needsRebuild) {
      if (c) {
        c.destroy({ children: true });
        state.entityContainers.delete(id);
      }
      const built = buildEntityContainer(
        PIXI,
        bodyGrhId,
        headGrhId,
        eBodyId,
        e.name,
        nc,
        createSprite,
      );
      c = built.container;
      entityLayer.addChild(c);
      state.entityContainers.set(id, c);
      if (built.hasRealSprite || bodyGrhId === 0) {
        state.entityRenderStates.set(id, buildRenderState(eBodyId, eHeadId, bodyGrhId, headGrhId, e.heading, e.dead, nc));
      }
    } else if (prev && prev.heading !== e.heading) {
      // Heading changed but body/head identity is the same — just update
      // the animation frame set (no container rebuild, no Text recreation).
      updateRenderStateHeading(prev, bodyGrhId, headGrhId, e.heading);
    }

    if (c) {
      let visualX = e.x;
      let visualY = e.y;
      const buf = interpolationBuffers.get(id);
      if (buf && !buf.isEmpty) {
        const sample = buf.sample(estimatedServerTick);
        if (sample) {
          const prev = sample.previous as { x: number; y: number };
          const next = sample.next as { x: number; y: number };
          visualX = prev.x + (next.x - prev.x) * sample.alpha;
          visualY = prev.y + (next.y - prev.y) * sample.alpha;
        }
      }
      c.x = Math.floor((visualX - 1) * TILE_SIZE);
      c.y = Math.floor((visualY - 1) * TILE_SIZE);

      // Only update zIndex when the rounded Y changes — avoids
      // triggering Pixi's sortableChildren sort every frame.
      const newZ = Math.round(visualY) * 10 + 5;
      if (c.zIndex !== newZ) c.zIndex = newZ;

      c.visible = true;
      c.alpha = e.dead ? 0.45 : 1;

      const rs = state.entityRenderStates.get(id);
      if (rs) {
        if (rs.lastVisualX >= 0 && (Math.abs(visualX - rs.lastVisualX) > 0.01 || Math.abs(visualY - rs.lastVisualY) > 0.01)) {
          state.entityMoveTimes.set(id, performance.now());
        }
        rs.lastVisualX = visualX;
        rs.lastVisualY = visualY;
      }
    }
  }
  for (const [id, c] of state.entityContainers) {
    if (!active.has(id)) {
      c.destroy({ children: true });
      state.entityContainers.delete(id);
      state.entityRenderStates.delete(id);
      state.entityMoveTimes.delete(id);
    }
  }
}

export function renderNpcs(
  PIXI: typeof import("pixi.js"),
  entityLayer: PixiContainer,
  state: EntityContainerState,
  remoteNpcs: Map<number, any>,
  interpolationBuffers: Map<number, any>,
  estimatedServerTick: number,
  createSprite: SpriteFactory,
) {
  const active = new Set<number>();
  for (const [id, npc] of remoteNpcs) {
    active.add(id);
    const npcData = assetStore.getNpcBodyHead(npc.npcType);
    const idBody = npcData?.idBody ?? 0;
    const idHead = npcData?.idHead ?? 0;
    const bodyGrhId =
      idBody > 0 ? assetStore.getBodyGrhId(idBody, npc.heading) : 0;
    const headGrhId =
      idHead > 0 ? assetStore.getHeadGrhId(idHead, npc.heading) : 0;

    const prev = state.npcRenderStates.get(id);
    // Only rebuild when the NPC's visual identity changes — NOT on heading.
    const needsRebuild =
      !prev ||
      prev.bodyId !== idBody ||
      prev.headId !== idHead ||
      prev.dead !== npc.dead;

    let c = state.npcContainers.get(id);
    if (needsRebuild) {
      if (c) {
        c.destroy({ children: true });
        state.npcContainers.delete(id);
      }
      const npcName = assetStore.getNpcName(npc.npcType);
      const built = buildEntityContainer(
        PIXI,
        bodyGrhId,
        headGrhId,
        idBody,
        npcName,
        0xfca5a5,
        createSprite,
      );
      c = built.container;
      entityLayer.addChild(c);
      state.npcContainers.set(id, c);
      if (built.hasRealSprite || bodyGrhId === 0) {
        state.npcRenderStates.set(id, buildRenderState(idBody, idHead, bodyGrhId, headGrhId, npc.heading, npc.dead, 0));
      }
    } else if (prev && prev.heading !== npc.heading) {
      updateRenderStateHeading(prev, bodyGrhId, headGrhId, npc.heading);
    }

    if (c) {
      let visualX = npc.x;
      let visualY = npc.y;
      const buf = interpolationBuffers.get(id);
      if (buf && !buf.isEmpty) {
        const sample = buf.sample(estimatedServerTick);
        if (sample) {
          const prev = sample.previous as { x: number; y: number };
          const next = sample.next as { x: number; y: number };
          visualX = prev.x + (next.x - prev.x) * sample.alpha;
          visualY = prev.y + (next.y - prev.y) * sample.alpha;
        }
      }
      c.x = Math.floor((visualX - 1) * TILE_SIZE);
      c.y = Math.floor((visualY - 1) * TILE_SIZE);

      const newZ = Math.round(visualY) * 10 + 5;
      if (c.zIndex !== newZ) c.zIndex = newZ;

      c.visible = true;
      c.alpha = npc.dead ? 0.45 : 1;

      const rs = state.npcRenderStates.get(id);
      if (rs) {
        if (rs.lastVisualX >= 0 && (Math.abs(visualX - rs.lastVisualX) > 0.01 || Math.abs(visualY - rs.lastVisualY) > 0.01)) {
          state.entityMoveTimes.set(id, performance.now());
        }
        rs.lastVisualX = visualX;
        rs.lastVisualY = visualY;
      }
    }
  }
  for (const [id, c] of state.npcContainers) {
    if (!active.has(id)) {
      c.destroy({ children: true });
      state.npcContainers.delete(id);
      state.npcRenderStates.delete(id);
      state.entityMoveTimes.delete(id);
    }
  }
}

let _getTexSrcFn: ((PIXI: any, cache: any, pending: any, fileNum: number) => any) | null = null;
export function setGetTextureSourceFn(fn: (PIXI: any, cache: any, pending: any, fileNum: number) => any) {
  _getTexSrcFn = fn;
}

/**
 * Called from PixiApp ticker every frame.
 * Drives the walk-cycle animation for all entities, NPCs, and the player.
 *
 * Animation logic ported from the original Engine.ts:
 * - An entity is "moving" for WALK_DURATION_MS after its last position change.
 * - While moving, the body frame cycles over BODY_ANIMATION_CYCLE_MS.
 * - When idle, frame resets to 0 (the standing/idle pose).
 */
export function animateEntitySprites(
  PIXI: typeof import("pixi.js"),
  state: EntityContainerState,
  cache: TextureSourceCache,
  pendingLoads: Set<number>,
  now: number,
) {
  const allSets: [Map<number, PixiContainer>, Map<number, EntityRenderState>][] = [
    [state.entityContainers, state.entityRenderStates],
    [state.npcContainers, state.npcRenderStates],
  ];

  if (state.playerContainer && state.playerRenderState) {
    const isMoving = (now - state.playerLastMoveTime) < WALK_DURATION_MS;
    updateContainerAnim(PIXI, state.playerContainer, state.playerRenderState, cache, pendingLoads, now, isMoving);
  }

  for (const [containers, renderStates] of allSets) {
    for (const [id, c] of containers) {
      const rs = renderStates.get(id);
      if (!rs) continue;
      const lastMove = state.entityMoveTimes.get(id) ?? 0;
      const isMoving = (now - lastMove) < WALK_DURATION_MS;
      updateContainerAnim(PIXI, c, rs, cache, pendingLoads, now, isMoving);
    }
  }
}

function updateContainerAnim(
  PIXI: typeof import("pixi.js"),
  container: PixiContainer,
  rs: EntityRenderState,
  cache: TextureSourceCache,
  _pendingLoads: Set<number>,
  now: number,
  isMoving: boolean,
) {
  if (!rs.bodyAnimFrames || rs.bodyFrameCount <= 1) return;

  if (isMoving) {
    if (!rs.wasMoving) {
      rs.animStartTime = now;
      rs.wasMoving = true;
    }
    const elapsed = now - rs.animStartTime;
    const msPerFrame = Math.max(Math.round(BODY_ANIMATION_CYCLE_MS / rs.bodyFrameCount), 1);
    const frameIdx = Math.floor(elapsed / msPerFrame) % rs.bodyFrameCount;

    if (frameIdx !== rs.lastFrameIdx) {
      rs.lastFrameIdx = frameIdx;
      applyBodyFrame(PIXI, container, rs.bodyAnimFrames, frameIdx, cache);
      if (rs.headAnimFrames && rs.headFrameCount > 1) {
        const headFrameIdx = Math.floor(elapsed / msPerFrame) % rs.headFrameCount;
        applyHeadFrame(PIXI, container, rs.headAnimFrames, headFrameIdx, cache);
      }
    }
  } else {
    if (rs.wasMoving || rs.lastFrameIdx !== 0) {
      rs.lastFrameIdx = 0;
      rs.wasMoving = false;
      applyBodyFrame(PIXI, container, rs.bodyAnimFrames, 0, cache);
      if (rs.headAnimFrames && rs.headFrameCount > 1) {
        applyHeadFrame(PIXI, container, rs.headAnimFrames, 0, cache);
      }
    }
  }
}

function applyBodyFrame(
  PIXI: typeof import("pixi.js"),
  container: PixiContainer,
  frames: GraphicInfo[],
  frameIdx: number,
  cache: TextureSourceCache,
) {
  const frameInfo = frames[frameIdx];
  if (!frameInfo) return;
  for (const child of container.children) {
    if ((child as any)._isBody) {
      const src = cache.get(frameInfo.fileNum);
      if (!src) continue;
      try {
        const fw = Math.min(frameInfo.w, src.width - frameInfo.sX);
        const fh = Math.min(frameInfo.h, src.height - frameInfo.sY);
        if (fw > 0 && fh > 0) {
          (child as PixiSprite).texture = new PIXI.Texture({
            source: src,
            frame: new PIXI.Rectangle(frameInfo.sX, frameInfo.sY, fw, fh),
          });
        }
      } catch { /* keep current frame */ }
    }
  }
}

function applyHeadFrame(
  PIXI: typeof import("pixi.js"),
  container: PixiContainer,
  frames: GraphicInfo[],
  frameIdx: number,
  cache: TextureSourceCache,
) {
  const headFrame = frames[frameIdx];
  if (!headFrame) return;
  for (const child of container.children) {
    if ((child as any)._isHead) {
      const src = cache.get(headFrame.fileNum);
      if (!src) continue;
      try {
        const fw = Math.min(headFrame.w, src.width - headFrame.sX);
        const fh = Math.min(headFrame.h, src.height - headFrame.sY);
        if (fw > 0 && fh > 0) {
          (child as PixiSprite).texture = new PIXI.Texture({
            source: src,
            frame: new PIXI.Rectangle(headFrame.sX, headFrame.sY, fw, fh),
          });
        }
      } catch { /* keep current frame */ }
    }
  }
}

/**
 * Called externally when an entity position changes (from moveEntity handler).
 * Records the timestamp so `animateEntitySprites` knows the entity is walking.
 */
export function notifyEntityMoved(state: EntityContainerState, entityId: number) {
  state.entityMoveTimes.set(entityId, performance.now());
}

/**
 * Called externally when the local player moves.
 */
export function notifyPlayerMoved(state: EntityContainerState) {
  state.playerLastMoveTime = performance.now();
}

export function renderPlayer(
  PIXI: typeof import("pixi.js"),
  state: EntityContainerState,
  hud: {
    idBody?: number;
    idHead?: number;
    heading: number;
    dead: boolean;
    nameColor?: number;
    name?: string;
  },
  px: number,
  py: number,
  createSprite: SpriteFactory,
) {
  const pc = state.playerContainer;
  if (!pc) return;

  const idBody = hud.idBody || 1;
  const idHead = hud.idHead || 1;
  const heading = hud.heading;

  const bodyGrhId = assetStore.getBodyGrhId(idBody, heading);
  const headGrhId = idHead > 0 ? assetStore.getHeadGrhId(idHead, heading) : 0;

  const prev = state.playerRenderState;
  const nc = nameColor({ dead: hud.dead, nameColor: hud.nameColor });
  // Only rebuild the player container when the base identity changes.
  // Heading-only changes swap animation frames (fast path).
  const needsRebuild =
    !prev ||
    prev.bodyId !== idBody ||
    prev.headId !== idHead ||
    prev.dead !== hud.dead ||
    prev.nameColor !== nc;

  if (!needsRebuild && prev && prev.heading !== heading) {
    updateRenderStateHeading(prev, bodyGrhId, headGrhId, heading);
  }

  if (needsRebuild) {
    while (pc.children.length > 0) {
      const child = pc.children[0];
      if (child) {
        pc.removeChild(child);
        child.destroy();
      }
    }

    const bodyInfo = bodyGrhId > 0 ? getGraphicInfo(bodyGrhId) : null;
    if (bodyInfo) {
      const bodySprite = createSprite(bodyInfo);
      if (bodySprite) {
        const pos = getBodySpritePosition(bodyInfo.w, bodyInfo.h);
        bodySprite.x = pos.x;
        bodySprite.y = pos.y;
        bodySprite.zIndex = 0.2;
        (bodySprite as any)._isBody = true;
        pc.addChild(bodySprite);

        if (headGrhId > 0) {
          const headInfo = getGraphicInfo(headGrhId);
          if (headInfo) {
            const headSprite = createSprite(headInfo);
            if (headSprite) {
              const bodyData = assetStore.getBodyData(idBody);
              const hx =
                pos.x +
                bodyInfo.w / 2 -
                headInfo.w / 2 +
                (bodyData?.headOffsetX ?? 0);
              const hy =
                pos.y +
                bodyInfo.h -
                50 +
                (bodyData?.headOffsetY ?? 0);
              headSprite.x = Math.round(hx);
              headSprite.y = Math.round(hy);
              headSprite.zIndex = 0.1;
              (headSprite as any)._isHead = true;
              pc.addChild(headSprite);
            }
          }
        }

        const t = new PIXI.Text({
          text: hud.name || "",
          style: {
            fontSize: 11,
            fill: nc,
            fontFamily: "sans-serif",
            fontWeight: "bold",
            dropShadow: {
              alpha: 0.9,
              angle: 0,
              blur: 2,
              color: 0x000000,
              distance: 0,
            },
          },
        });
        t.anchor.set(0.5, 0);
        t.x = Math.round(pos.x + bodyInfo.w / 2);
        t.y = Math.round(pos.y + bodyInfo.h + 2);
        t.zIndex = 0.6;
        pc.addChild(t);
      }
    } else {
      const g = new PIXI.Graphics();
      g.circle(0, 0, TILE_SIZE * 0.4);
      g.fill({ color: hud.dead ? 0x991b1b : 0xfbbf24 });
      g.stroke({ color: 0xfcd34d, width: 2 });
      pc.addChild(g);
      const t = new PIXI.Text({
        text: hud.name || "",
        style: {
          fontSize: 11,
          fill: 0xfcd34d,
          fontFamily: "sans-serif",
          fontWeight: "bold",
          dropShadow: {
            alpha: 0.9,
            angle: 0,
            blur: 2,
            color: 0x000000,
            distance: 0,
          },
        },
      });
      t.anchor.set(0.5, 1);
      t.y = -TILE_SIZE * 0.55;
      pc.addChild(t);
    }

    state.playerRenderState = buildRenderState(idBody, idHead, bodyGrhId, headGrhId, heading, hud.dead, nc);
  }

  // NOTE: pc.x / pc.y are NOT set here.  The caller (PixiApp.svelte)
  // is the single owner of the player container's world position
  // because it needs to apply the sub-tile interpolation offset.
  // Setting it here would cause a 1-frame snap before the offset is
  // applied, producing visible jitter.
  pc.zIndex = py * 10 + 5;
  pc.alpha = hud.dead ? 0.45 : 1;

  const prs = state.playerRenderState;
  if (prs) {
    if (prs.lastVisualX >= 0 && (Math.abs(px - prs.lastVisualX) > 0.01 || Math.abs(py - prs.lastVisualY) > 0.01)) {
      state.playerLastMoveTime = performance.now();
    }
    prs.lastVisualX = px;
    prs.lastVisualY = py;
  }
}
