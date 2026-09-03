import {
  decodeMap,
  getTileAt,
  loadGraphicsDB,
  loadMapJSON,
  preloadMapGraphics,
  type MapParsed,
} from "$lib/game/engine/assetLoader";

class MapState {
  mapParsed: MapParsed | null = $state(null);
  currentMapId = $state(0);
  loading = $state(false);
  private failedMapIds = new Set<number>();

  async load(mapId: number) {
    if (mapId === this.currentMapId && this.mapParsed) return;
    if (this.loading) return;
    if (this.failedMapIds.has(mapId)) return;
    this.loading = true;
    try {
      await loadGraphicsDB();
      const raw = await loadMapJSON(mapId);
      const parsed = decodeMap(raw);
      await preloadMapGraphics(parsed);
      this.mapParsed = parsed;
      this.currentMapId = mapId;
      this.failedMapIds.delete(mapId);
    } catch (e) {
      console.error("[mapState] load error:", e);
      this.failedMapIds.add(mapId);
    } finally {
      this.loading = false;
    }
  }

  isTileBlocked(x: number, y: number): boolean {
    if (!this.mapParsed) return false;
    if (x < 1 || x > this.mapParsed.w || y < 1 || y > this.mapParsed.h) return true;
    const tile = getTileAt(this.mapParsed, x, y);
    return tile?.blocked === true;
  }

  getTile(x: number, y: number) {
    if (!this.mapParsed) return undefined;
    return getTileAt(this.mapParsed, x, y);
  }

  clear() {
    this.mapParsed = null;
    this.currentMapId = 0;
  }
}

export const mapState = new MapState();
