type SimpleGraphic = [fileNum: number, sX: number, sY: number, width: number, height: number];
type CompactExtended = { f?: number; r?: number[]; s?: number; n?: number; x?: number; y?: number; w?: number; h?: number; o?: [number, number] };
type GraphicEntry = SimpleGraphic | CompactExtended;
type GraphicsDB = Record<string, GraphicEntry>;

export interface TileData {
  graphics: Record<string, number>;
  blocked: boolean;
  trigger?: number;
  tileExit?: { map: number; x: number; y: number };
}

export interface MapParsed {
  id: number;
  w: number;
  h: number;
  tiles: Record<number, Record<number, TileData>>;
}

interface MapJSON {
  id: number;
  w: number;
  h: number;
  d?: number[];
  cx?: Array<Record<string, unknown>>;
  tiles?: unknown[];
  patterns?: unknown[];
  data?: number[];
}

let graphicsDB: GraphicsDB | null = null;
const textureCache = new Map<string, HTMLImageElement>();
const imageLoadPromises = new Map<string, Promise<HTMLImageElement>>();

export async function loadGraphicsDB(): Promise<GraphicsDB> {
  if (graphicsDB) return graphicsDB;
  const resp = await fetch("/init/graficos_optimized.json");
  graphicsDB = (await resp.json()) as GraphicsDB;
  return graphicsDB;
}

export function getGraphicsDB(): GraphicsDB | null {
  return graphicsDB;
}

export async function loadMapJSON(mapId: number): Promise<MapJSON> {
  const candidates = [
    `/maps_optimized/mapa_${mapId}.json`,
    `/maps/mapa_${mapId}.json`,
  ];
  for (const url of candidates) {
    try {
      const resp = await fetch(url);
      if (resp.ok) return (await resp.json()) as MapJSON;
    } catch { /* try next */ }
  }
  throw new Error(`Map ${mapId} not found`);
}

function isSimple(entry: GraphicEntry): entry is SimpleGraphic {
  return Array.isArray(entry);
}

export interface GraphicInfo {
  fileNum: number;
  sX: number;
  sY: number;
  w: number;
  h: number;
  offsetX: number;
  offsetY: number;
}

export interface AnimationInfo {
  frames: GraphicInfo[];
  speed: number;
  frameCount: number;
}

export function getAnimationInfo(grhId: number): AnimationInfo | null {
  if (!graphicsDB || grhId === 0) return null;
  const entry = graphicsDB[String(grhId)];
  if (!entry) return null;

  if (!isSimple(entry) && entry.r && entry.r.length > 0) {
    const frames: GraphicInfo[] = [];
    for (const fid of entry.r) {
      const fe = graphicsDB[String(fid)];
      if (fe && isSimple(fe)) {
        frames.push({ fileNum: fe[0], sX: fe[1], sY: fe[2], w: fe[3], h: fe[4], offsetX: 0, offsetY: 0 });
      }
    }
    if (frames.length > 0) {
      return { frames, speed: entry.s ?? 100, frameCount: frames.length };
    }
  }

  const single = getGraphicInfo(grhId);
  if (single) {
    return { frames: [single], speed: 0, frameCount: 1 };
  }
  return null;
}

export function getGraphicInfo(grhId: number): GraphicInfo | null {
  if (!graphicsDB || grhId === 0) return null;
  const strId = String(grhId);
  const entry = graphicsDB[strId];
  if (!entry) return null;

  if (isSimple(entry)) {
    return { fileNum: entry[0], sX: entry[1], sY: entry[2], w: entry[3], h: entry[4], offsetX: 0, offsetY: 0 };
  }

  if (entry.n !== undefined && entry.x !== undefined && entry.y !== undefined && entry.w !== undefined && entry.h !== undefined) {
    return {
      fileNum: entry.n,
      sX: entry.x,
      sY: entry.y,
      w: entry.w,
      h: entry.h,
      offsetX: entry.o?.[0] ?? 0,
      offsetY: entry.o?.[1] ?? 0,
    };
  }

  if (entry.r && entry.r.length > 0) {
    const firstFrame = String(entry.r[0]);
    const frameEntry = graphicsDB[firstFrame];
    if (frameEntry && isSimple(frameEntry)) {
      return { fileNum: frameEntry[0], sX: frameEntry[1], sY: frameEntry[2], w: frameEntry[3], h: frameEntry[4], offsetX: 0, offsetY: 0 };
    }
  }

  return null;
}

export function loadImage(fileNum: number): Promise<HTMLImageElement> {
  const key = String(fileNum);
  const cached = imageLoadPromises.get(key);
  if (cached) return cached;

  const promise = new Promise<HTMLImageElement>((resolve, reject) => {
    const existing = textureCache.get(key);
    if (existing) { resolve(existing); return; }
    const img = new Image();
    img.crossOrigin = "anonymous";
    img.onload = () => { textureCache.set(key, img); resolve(img); };
    img.onerror = () => reject(new Error(`Failed to load /graphics/${fileNum}.png`));
    img.src = `/graphics/${fileNum}.png`;
  });

  imageLoadPromises.set(key, promise);
  return promise;
}

export function getImageSync(fileNum: number): HTMLImageElement | undefined {
  return textureCache.get(String(fileNum));
}

function parseGraphicsField(g: unknown): Record<string, number> {
  const graphics: Record<string, number> = {};
  if (typeof g === "number") {
    graphics["1"] = g;
  } else if (Array.isArray(g)) {
    for (let i = 0; i < g.length; i++) {
      if (g[i] !== null && g[i] !== undefined) {
        graphics[(i + 1).toString()] = g[i] as number;
      }
    }
  } else if (g && typeof g === "object") {
    const gObj = g as Record<string, number>;
    for (const k of Object.keys(gObj)) {
      if (gObj[k] !== undefined) graphics[k] = gObj[k];
    }
  }
  return graphics;
}

/**
 * Decodes map data matching the original frontend format exactly.
 * The .d array iterates: for outerIdx=1..w { for innerIdx=1..h }.
 * The original stores as mapData[outerIdx][innerIdx] but accesses
 * with getTileAt(mapData, mapNumber, x, y) = mapData[mapNumber][y][x].
 * So outerIdx corresponds to y (row) and innerIdx to x (column).
 * We replicate: tiles[y][x] — accessed via getTileAt(map, x, y).
 */
export function decodeMap(mapData: MapJSON): MapParsed {
  const { w, h, id } = mapData;
  const tiles: Record<number, Record<number, TileData>> = {};

  if (mapData.d) {
    let index = 0;
    for (let outerIdx = 1; outerIdx <= w; outerIdx++) {
      if (!tiles[outerIdx]) tiles[outerIdx] = {};
      for (let innerIdx = 1; innerIdx <= h; innerIdx++) {
        const value = mapData.d[index++];
        if (value === undefined || value === 0) continue;

        const tile: TileData = { graphics: {}, blocked: false };

        if (value > 100000) {
          tile.blocked = true;
          tile.graphics = { "1": value - 100000 };
        } else if (value > 0) {
          tile.graphics = { "1": value };
        } else if (value < 0 && mapData.cx) {
          const complexIndex = -value - 1;
          const cx = mapData.cx[complexIndex] as Record<string, unknown> | undefined;
          if (!cx) continue;

          if (cx.b) tile.blocked = true;
          if (cx.g !== undefined) tile.graphics = parseGraphicsField(cx.g);
          if (cx.e) {
            const e = cx.e as { m: number; x: number; y: number };
            tile.tileExit = { map: e.m, x: e.x, y: e.y };
          }
          if (cx.t !== undefined) tile.trigger = cx.t as number;
        }

        const row = tiles[outerIdx];
        if (row) row[innerIdx] = tile;
      }
    }
  }

  return { id, w, h, tiles };
}

/**
 * Access tile at (x, y) — matches original: mapData[mapNumber][y][x]
 */
export function getTileAt(map: MapParsed, x: number, y: number): TileData | undefined {
  return map.tiles[y]?.[x];
}

export async function preloadMapGraphics(map: MapParsed): Promise<Set<number>> {
  const fileNums = new Set<number>();

  for (const rowKey of Object.keys(map.tiles)) {
    const row = map.tiles[Number(rowKey)];
    if (!row) continue;
    for (const colKey of Object.keys(row)) {
      const tile = row[Number(colKey)];
      if (!tile) continue;
      for (const layerKey of Object.keys(tile.graphics)) {
        const grhId = tile.graphics[layerKey];
        if (!grhId) continue;
        const info = getGraphicInfo(grhId);
        if (info) fileNums.add(info.fileNum);
      }
    }
  }

  const batchSize = 50;
  const fileNumArr = [...fileNums];
  for (let i = 0; i < fileNumArr.length; i += batchSize) {
    const batch = fileNumArr.slice(i, i + batchSize);
    await Promise.all(batch.map((fn) => loadImage(fn).catch(() => null)));
  }

  return fileNums;
}
