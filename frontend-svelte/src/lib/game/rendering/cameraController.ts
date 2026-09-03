import type { Container as PixiContainer, Application } from "pixi.js";
import { TILE_SIZE } from "$lib/game/lib/viewport";
import { sendClick, sendAttackSpell } from "$lib/game/session/outgoingRequests";
import { gameState } from "$lib/game/state/gameState.svelte";

export type ViewBounds = {
  minX: number;
  maxX: number;
  minY: number;
  maxY: number;
  viewW: number;
  viewH: number;
};

// ── Smooth-follow camera (LERP) ──────────────────────────────────────
//
// Instead of rigidly copying the player position every frame
// (`worldContainer.x = cw/2 - player.x`), we maintain an internal
// "current" camera position that chases a "target" via linear
// interpolation.  This absorbs micro-freezes between walk steps and
// produces a buttery-smooth scroll.
//
// The final worldContainer position is Math.floor'd so that every
// sprite sits on integer pixel boundaries → no nearest-neighbour
// shimmer on tile edges.

let currentCamX = -1;
let currentCamY = -1;

/** How quickly the camera catches up to the target (0–1 per frame). */
const LERP_FACTOR = 0.18;

/** Distance (px) under which we snap instead of interpolating. */
const SNAP_THRESHOLD = 0.5;

/**
 * Teleport the camera to a specific world position immediately
 * (no interpolation).  Call this on map change or connect.
 */
export function resetCamera(targetX: number, targetY: number): void {
  currentCamX = targetX;
  currentCamY = targetY;
}

/**
 * Called once per Pixi ticker frame.  Computes the target camera
 * focus point from the player's tile + sub-tile offset, LERPs the
 * internal camera toward it, and positions the worldContainer.
 */
export function updateCamera(
  app: Application,
  worldContainer: PixiContainer,
  px: number,
  py: number,
  offsetPx = 0,
  offsetPy = 0,
): void {
  const cw = app.screen.width;
  const ch = app.screen.height;

  // Target = exact world-pixel position of the player's visual centre.
  const targetX = (px - 1) * TILE_SIZE + TILE_SIZE / 2 + offsetPx;
  const targetY = (py - 1) * TILE_SIZE + TILE_SIZE / 2 + offsetPy;

  // First frame or after a teleport / map change → snap immediately.
  if (currentCamX < 0) {
    currentCamX = targetX;
    currentCamY = targetY;
  }

  // LERP toward the target.
  const dx = targetX - currentCamX;
  const dy = targetY - currentCamY;
  if (Math.abs(dx) < SNAP_THRESHOLD && Math.abs(dy) < SNAP_THRESHOLD) {
    currentCamX = targetX;
    currentCamY = targetY;
  } else {
    currentCamX += dx * LERP_FACTOR;
    currentCamY += dy * LERP_FACTOR;
  }

  // Final position: Math.floor so every tile sits on an integer pixel.
  // Because the LERP has already smoothed out micro-irregularities,
  // floor produces a steady 2-or-3 px advance rather than the chaotic
  // alternation that happens when rounding a jerky raw position.
  worldContainer.x = Math.floor(cw / 2 - currentCamX);
  worldContainer.y = Math.floor(ch / 2 - currentCamY);
}

/** Expose internal camera for the player container positioning. */
export function getCameraPosition(): { x: number; y: number } {
  return { x: currentCamX, y: currentCamY };
}

export function computeViewBounds(
  app: Application,
  px: number,
  py: number,
  mapW: number,
  mapH: number,
): ViewBounds {
  const cw = app.screen.width;
  const ch = app.screen.height;
  const viewW = Math.ceil(cw / TILE_SIZE / 2) + 2;
  const viewH = Math.ceil(ch / TILE_SIZE / 2) + 2;

  return {
    minX: Math.max(1, px - viewW),
    maxX: Math.min(mapW, px + viewW),
    minY: Math.max(1, py - viewH),
    maxY: Math.min(mapH, py + viewH),
    viewW,
    viewH,
  };
}

export function handleCanvasClick(
  app: Application,
  playerX: number,
  playerY: number,
  e: MouseEvent,
) {
  const canvas = app.canvas as HTMLCanvasElement;
  const rect = canvas.getBoundingClientRect();
  const cw = app.screen.width;
  const ch = app.screen.height;

  const scaleX = rect.width / cw;
  const scaleY = rect.height / ch;

  const mx = (e.clientX - rect.left) / scaleX;
  const my = (e.clientY - rect.top) / scaleY;

  const tileX = playerX + Math.round((mx - cw / 2) / TILE_SIZE);
  const tileY = playerY + Math.round((my - ch / 2) / TILE_SIZE);
  if (tileX >= 1 && tileX <= 100 && tileY >= 1 && tileY <= 100) {
    if (gameState.pendingSpellSlot !== null) {
      sendAttackSpell(gameState.pendingSpellSlot);
      gameState.pendingSpellSlot = null;
    } else {
      sendClick(tileX, tileY);
    }
  }
}
