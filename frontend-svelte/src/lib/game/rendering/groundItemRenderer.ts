import type { Container as PixiContainer } from "pixi.js";
import { getGraphicInfo } from "$lib/game/engine/assetLoader";
import { TILE_SIZE, type SpriteFactory } from "./tileRenderer";

export type GroundItemContainerState = {
  containers: Map<string, PixiContainer>;
};

export function createGroundItemState(): GroundItemContainerState {
  return { containers: new Map() };
}

export function clearGroundItemState(state: GroundItemContainerState) {
  for (const c of state.containers.values()) c.destroy({ children: true });
  state.containers.clear();
}

export function renderGroundItems(
  PIXI: typeof import("pixi.js"),
  entityLayer: PixiContainer,
  state: GroundItemContainerState,
  groundItems: Map<string, { x: number; y: number; grhIndex?: number }>,
  createSprite: SpriteFactory,
) {
  const active = new Set<string>();
  for (const [key, item] of groundItems) {
    active.add(key);
    let c = state.containers.get(key);
    if (!c) {
      c = new PIXI.Container();
      let ok = false;
      if (item.grhIndex) {
        const info = getGraphicInfo(item.grhIndex);
        if (info) {
          const sprite = createSprite(info);
          if (sprite) {
            const scale = Math.min(1, 24 / Math.max(info.w, info.h, 1));
            sprite.anchor.set(0.5, 0.5);
            sprite.scale.set(scale);
            c.addChild(sprite);
            ok = true;
          }
        }
      }
      if (!ok) {
        const g = new PIXI.Graphics();
        g.roundRect(-6, -6, 12, 12, 2);
        g.fill({ color: 0x22c55e, alpha: 0.8 });
        c.addChild(g);
      }
      entityLayer.addChild(c);
      state.containers.set(key, c);
    }
    c.x = (item.x - 1) * TILE_SIZE + TILE_SIZE / 2;
    c.y = (item.y - 1) * TILE_SIZE + TILE_SIZE / 2;
    c.zIndex = item.y * 10 + 3;
    c.visible = true;
  }
  for (const [key, c] of state.containers) {
    if (!active.has(key)) {
      c.destroy({ children: true });
      state.containers.delete(key);
    }
  }
}
