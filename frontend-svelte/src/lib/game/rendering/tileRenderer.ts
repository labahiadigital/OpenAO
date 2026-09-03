import type {
  Container as PixiContainer,
  Sprite as PixiSprite,
  TextureSource as PixiTextureSource,
} from "pixi.js";
import {
  getGraphicInfo,
  getImageSync,
  loadImage,
  getTileAt,
  type GraphicInfo,
} from "$lib/game/engine/assetLoader";

export const TILE_SIZE = 32;
const TREE_FADE_ALPHA = 0.25;

export type TileState = {
  builtTiles: Set<string>;
  tileSprites: Map<string, PixiSprite[]>;
  roofSprites: Map<string, PixiSprite[]>;
  roofTiles: Set<string>;
  treeSpriteEntries: Array<{ x: number; y: number; sprite: PixiSprite }>;
  lastRoofHidden: boolean | null;
};

export function createTileState(): TileState {
  return {
    builtTiles: new Set(),
    tileSprites: new Map(),
    roofSprites: new Map(),
    roofTiles: new Set(),
    treeSpriteEntries: [],
    lastRoofHidden: null,
  };
}

export function clearTileState(
  state: TileState,
  groundLayer: PixiContainer | undefined,
  belowLayer: PixiContainer | undefined,
  aboveLayer: PixiContainer | undefined,
  roofLayer: PixiContainer | undefined,
) {
  for (const sprites of state.tileSprites.values())
    for (const s of sprites) s.destroy();
  state.tileSprites.clear();
  state.roofSprites.clear();
  state.roofTiles.clear();
  state.treeSpriteEntries = [];
  state.builtTiles.clear();
  state.lastRoofHidden = null;
  groundLayer?.removeChildren();
  belowLayer?.removeChildren();
  aboveLayer?.removeChildren();
  roofLayer?.removeChildren();
}

export type SpriteFactory = (info: GraphicInfo) => PixiSprite | null;

function getBottomAnchoredPosition(w: number, h: number) {
  return {
    x: 16 - Math.floor((w * 16) / 32),
    y: 32 - Math.floor((h * 16) / 16),
  };
}

export function buildTile(
  state: TileState,
  mapParsed: any,
  x: number,
  y: number,
  layers: {
    groundLayer: PixiContainer;
    belowLayer: PixiContainer;
    aboveLayer: PixiContainer;
    roofLayer: PixiContainer;
  },
  createSprite: SpriteFactory,
) {
  const key = `${x},${y}`;
  if (state.builtTiles.has(key)) return;
  state.builtTiles.add(key);

  const tile = getTileAt(mapParsed, x, y);
  if (!tile || Object.keys(tile.graphics).length === 0) return;

  const wx = (x - 1) * TILE_SIZE;
  const wy = (y - 1) * TILE_SIZE;
  const sprites: PixiSprite[] = [];

  const layer1 = tile.graphics["1"];
  if (layer1) {
    const info = getGraphicInfo(layer1);
    if (info) {
      const sprite = createSprite(info);
      if (sprite) {
        sprite.x = wx;
        sprite.y = wy;
        sprite.width = TILE_SIZE;
        sprite.height = TILE_SIZE;
        layers.groundLayer.addChild(sprite);
        sprites.push(sprite);
      }
    }
  }

  const layer2 = tile.graphics["2"];
  if (layer2) {
    const info = getGraphicInfo(layer2);
    if (info) {
      const sprite = createSprite(info);
      if (sprite) {
        const pos = getBottomAnchoredPosition(info.w, info.h);
        sprite.x = wx + pos.x;
        sprite.y = wy + pos.y;
        layers.belowLayer.addChild(sprite);
        sprites.push(sprite);
      }
    }
  }

  const layer3 = tile.graphics["3"];
  if (layer3) {
    const info = getGraphicInfo(layer3);
    if (info) {
      const sprite = createSprite(info);
      if (sprite) {
        const pos = getBottomAnchoredPosition(info.w, info.h);
        sprite.x = wx + pos.x;
        sprite.y = wy + pos.y;
        sprite.zIndex = y * 10 + 7;
        layers.aboveLayer.addChild(sprite);
        sprites.push(sprite);
        state.treeSpriteEntries.push({ x, y, sprite });
      }
    }
  }

  const layer4 = tile.graphics["4"];
  if (layer4) {
    const info = getGraphicInfo(layer4);
    if (info) {
      const sprite = createSprite(info);
      if (sprite) {
        const pos = getBottomAnchoredPosition(info.w, info.h);
        sprite.x = wx + pos.x;
        sprite.y = wy + pos.y;
        sprite.zIndex = y * 10 + 9;
        layers.roofLayer.addChild(sprite);
        sprites.push(sprite);
        if (!state.roofSprites.has(key)) state.roofSprites.set(key, []);
        state.roofSprites.get(key)!.push(sprite);
        state.roofTiles.add(key);
      }
    }
  }

  if (sprites.length > 0) state.tileSprites.set(key, sprites);
}

export function updateRoofVisibility(
  state: TileState,
  mapParsed: any,
  px: number,
  py: number,
) {
  const playerTile = getTileAt(mapParsed, px, py);
  const shouldHide =
    playerTile?.trigger === 1 || Boolean(playerTile?.graphics["4"]);
  if (shouldHide === state.lastRoofHidden) return;
  state.lastRoofHidden = shouldHide;
  const targetAlpha = shouldHide ? 0 : 1;
  for (const sprites of state.roofSprites.values()) {
    for (const sprite of sprites) {
      sprite.alpha = targetAlpha;
      sprite.visible = targetAlpha > 0.05;
    }
  }
}

export function updateTreeTransparency(
  state: TileState,
  px: number,
  py: number,
) {
  for (const entry of state.treeSpriteEntries) {
    const dx = Math.abs(px - entry.x);
    const dy = Math.abs(py - entry.y);
    const shouldFade = dx <= 3 && dy < 12 && py < entry.y;
    entry.sprite.alpha = shouldFade ? TREE_FADE_ALPHA : 1;
  }
}

export function cullOffscreenTiles(
  state: TileState,
  px: number,
  py: number,
  vw: number,
  vh: number,
) {
  for (const [key, sprites] of state.tileSprites) {
    const parts = key.split(",");
    const tx = Number(parts[0]);
    const ty = Number(parts[1]);
    const visible =
      Math.abs(tx - px) <= vw + 1 && Math.abs(ty - py) <= vh + 1;
    for (const s of sprites) s.visible = visible;
  }

  for (const [key, sprites] of state.roofSprites) {
    if (state.lastRoofHidden) continue;
    const parts = key.split(",");
    const tx = Number(parts[0]);
    const ty = Number(parts[1]);
    const visible =
      Math.abs(tx - px) <= vw + 1 && Math.abs(ty - py) <= vh + 1;
    for (const s of sprites) s.visible = visible;
  }
}

export type TextureSourceCache = Map<number, PixiTextureSource>;

export function getTextureSource(
  PIXI: typeof import("pixi.js"),
  cache: TextureSourceCache,
  pendingLoads: Set<number>,
  fileNum: number,
): PixiTextureSource | null {
  const cached = cache.get(fileNum);
  if (cached) return cached;
  const img = getImageSync(fileNum);
  if (!img) {
    if (!pendingLoads.has(fileNum)) {
      pendingLoads.add(fileNum);
      loadImage(fileNum)
        .catch(() => {})
        .finally(() => pendingLoads.delete(fileNum));
    }
    return null;
  }
  try {
    const src = new PIXI.ImageSource({ resource: img });
    cache.set(fileNum, src);
    return src;
  } catch {
    return null;
  }
}

export function createSpriteFromInfo(
  PIXI: typeof import("pixi.js"),
  cache: TextureSourceCache,
  pendingLoads: Set<number>,
  info: GraphicInfo,
): PixiSprite | null {
  const src = getTextureSource(PIXI, cache, pendingLoads, info.fileNum);
  if (!src) return null;
  try {
    const needsFrame =
      info.sX !== 0 ||
      info.sY !== 0 ||
      info.w !== src.width ||
      info.h !== src.height;
    let texture;
    if (needsFrame) {
      const fw = Math.min(info.w, src.width - info.sX);
      const fh = Math.min(info.h, src.height - info.sY);
      if (fw <= 0 || fh <= 0) return null;
      texture = new PIXI.Texture({
        source: src,
        frame: new PIXI.Rectangle(info.sX, info.sY, fw, fh),
      });
    } else {
      texture = new PIXI.Texture({ source: src });
    }
    return new PIXI.Sprite(texture);
  } catch {
    return null;
  }
}
