/**
 * Asset Preloader — optimizes map transitions by preloading adjacent map data
 * and cleaning up unused texture entries when switching scenes.
 *
 * Uses `requestIdleCallback` to avoid blocking the game loop during preloads.
 */

import { loadMapData } from "./gameLoader";
import { collectAdjacentMapNumbers } from "../assets/scenePreload";
import type { MapData } from "../types/game";

const preloadedMaps = new Set<number>();
const PRELOAD_BATCH_SIZE = 3;

/** Max maps to keep cached; beyond this, LRU eviction triggers. */
const MAX_CACHED_MAPS = 30;

/** Tracks access order for LRU eviction (most recent at end). */
const mapAccessOrder: number[] = [];

function touchMap(mapNum: number): void {
	const idx = mapAccessOrder.indexOf(mapNum);
	if (idx !== -1) mapAccessOrder.splice(idx, 1);
	mapAccessOrder.push(mapNum);
}

/**
 * Preloads adjacent maps discovered via tile exits.
 * Uses idle callback to avoid frame drops during gameplay.
 */
export function preloadAdjacentMaps(
	mapData: MapData,
	currentMapNumber: number,
): void {
	touchMap(currentMapNumber);

	const adjacentMaps = collectAdjacentMapNumbers(mapData, currentMapNumber);

	const newMaps = adjacentMaps.filter((m) => !preloadedMaps.has(m));
	if (newMaps.length === 0) return;

	const batches: number[][] = [];
	for (let i = 0; i < newMaps.length; i += PRELOAD_BATCH_SIZE) {
		batches.push(newMaps.slice(i, i + PRELOAD_BATCH_SIZE));
	}

	let batchIndex = 0;

	function processNextBatch() {
		if (batchIndex >= batches.length) return;

		const batch = batches[batchIndex++];
		if (!batch) return;

		for (const mapNum of batch) {
			preloadedMaps.add(mapNum);
			touchMap(mapNum);
			loadMapData(mapNum).catch(() => {
				preloadedMaps.delete(mapNum);
			});
		}

		if (typeof requestIdleCallback === "function") {
			requestIdleCallback(processNextBatch, { timeout: 2000 });
		} else {
			setTimeout(processNextBatch, 100);
		}
	}

	if (typeof requestIdleCallback === "function") {
		requestIdleCallback(processNextBatch, { timeout: 3000 });
	} else {
		setTimeout(processNextBatch, 200);
	}
}

/**
 * Evicts the least-recently-used cached maps when the cache grows beyond
 * MAX_CACHED_MAPS. Called during preloadAdjacentMaps after new entries are added.
 * Only removes entries from the preloaded tracking set — the underlying
 * gameLoader jsonValueCache / mapValueCache hold weak references that the GC
 * can reclaim once nothing else holds them.
 */
export function evictDistantMaps(): void {
	while (mapAccessOrder.length > MAX_CACHED_MAPS) {
		const oldest = mapAccessOrder.shift();
		if (oldest !== undefined) {
			preloadedMaps.delete(oldest);
		}
	}
}

/**
 * Clears the preload tracking set (e.g. on disconnect).
 */
export function resetPreloadState(): void {
	preloadedMaps.clear();
	mapAccessOrder.length = 0;
}

/**
 * Returns a set of map numbers that have been preloaded.
 */
export function getPreloadedMaps(): ReadonlySet<number> {
	return preloadedMaps;
}

/**
 * Loads multiple game databases in parallel. Useful for the initial boot
 * sequence to overlap network latency instead of sequential awaits.
 */
export async function loadInitialAssetsParallel<T extends readonly Promise<unknown>[]>(
	...loaders: T
): Promise<{ -readonly [K in keyof T]: Awaited<T[K]> }> {
	return Promise.all(loaders) as Promise<{ -readonly [K in keyof T]: Awaited<T[K]> }>;
}
