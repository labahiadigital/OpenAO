import type {
  Container as PixiContainer,
  Sprite as PixiSprite,
} from "pixi.js";
import { getGraphicInfo } from "$lib/game/engine/assetLoader";
import { assetStore } from "$lib/game/state/assetStore.svelte";
import { TILE_SIZE, type SpriteFactory } from "./tileRenderer";

export type EntityRenderState = {
  bodyGrh: number;
  headGrh: number;
  heading: number;
  dead: boolean;
  nameColor: number;
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
};

export function createEntityContainerState(): EntityContainerState {
  return {
    entityContainers: new Map(),
    entityRenderStates: new Map(),
    npcContainers: new Map(),
    npcRenderStates: new Map(),
    playerContainer: undefined,
    playerRenderState: null,
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
    const bodyGrhId = e.bodyGrh
      ? assetStore.getBodyGrhId(e.bodyGrh, e.heading)
      : 0;
    const headGrhId = e.headGrh
      ? assetStore.getHeadGrhId(e.headGrh, e.heading)
      : 0;

    const prev = state.entityRenderStates.get(id);
    const needsRebuild =
      !prev ||
      prev.bodyGrh !== bodyGrhId ||
      prev.headGrh !== headGrhId ||
      prev.heading !== e.heading ||
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
        e.bodyGrh ?? 0,
        e.name,
        nc,
        createSprite,
      );
      c = built.container;
      entityLayer.addChild(c);
      state.entityContainers.set(id, c);
      if (built.hasRealSprite || bodyGrhId === 0) {
        state.entityRenderStates.set(id, {
          bodyGrh: bodyGrhId,
          headGrh: headGrhId,
          heading: e.heading,
          dead: e.dead,
          nameColor: nc,
        });
      }
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
      c.x = (visualX - 1) * TILE_SIZE;
      c.y = (visualY - 1) * TILE_SIZE;
      c.zIndex = Math.round(visualY) * 10 + 5;
      c.visible = true;
      c.alpha = e.dead ? 0.45 : 1;
    }
  }
  for (const [id, c] of state.entityContainers) {
    if (!active.has(id)) {
      c.destroy({ children: true });
      state.entityContainers.delete(id);
      state.entityRenderStates.delete(id);
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
    const needsRebuild =
      !prev ||
      prev.bodyGrh !== bodyGrhId ||
      prev.headGrh !== headGrhId ||
      prev.heading !== npc.heading ||
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
        state.npcRenderStates.set(id, {
          bodyGrh: bodyGrhId,
          headGrh: headGrhId,
          heading: npc.heading,
          dead: npc.dead,
          nameColor: 0,
        });
      }
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
      c.x = (visualX - 1) * TILE_SIZE;
      c.y = (visualY - 1) * TILE_SIZE;
      c.zIndex = Math.round(visualY) * 10 + 5;
      c.visible = true;
      c.alpha = npc.dead ? 0.45 : 1;
    }
  }
  for (const [id, c] of state.npcContainers) {
    if (!active.has(id)) {
      c.destroy({ children: true });
      state.npcContainers.delete(id);
      state.npcRenderStates.delete(id);
    }
  }
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

  const idBody = hud.idBody ?? 1;
  const idHead = hud.idHead ?? 1;
  const heading = hud.heading;

  const bodyGrhId = assetStore.getBodyGrhId(idBody, heading);
  const headGrhId = idHead > 0 ? assetStore.getHeadGrhId(idHead, heading) : 0;

  const prev = state.playerRenderState;
  const nc = nameColor({ dead: hud.dead, nameColor: hud.nameColor });
  const needsRebuild =
    !prev ||
    prev.bodyGrh !== bodyGrhId ||
    prev.headGrh !== headGrhId ||
    prev.heading !== heading ||
    prev.dead !== hud.dead ||
    prev.nameColor !== nc;

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

    state.playerRenderState = {
      bodyGrh: bodyGrhId,
      headGrh: headGrhId,
      heading,
      dead: hud.dead,
      nameColor: nc,
    };
  }

  pc.x = (px - 1) * TILE_SIZE;
  pc.y = (py - 1) * TILE_SIZE;
  pc.zIndex = py * 10 + 5;
  pc.alpha = hud.dead ? 0.45 : 1;
}
