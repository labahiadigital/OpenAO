/**
 * Movement Input Polling System
 *
 * keydown/keyup ONLY set boolean flags.  pollMovement() is called from the
 * Pixi ticker every frame.  This completely decouples movement from the OS
 * key-repeat delay and from setTimeout jitter.
 */

import { gameSession } from "$lib/game/session/gameSession.svelte";
import { gameState, WALK_STEP_MS } from "$lib/game/state/gameState.svelte";
import { mapState } from "$lib/game/state/mapState.svelte";
import {
  sendPosition,
  sendChangeHeading,
} from "$lib/game/session/outgoingRequests";

// Priority: last-pressed direction wins (like the original).
const heldDirections = new Set<number>(); // heading values: 1=up, 2=down, 3=right, 4=left
let directionPriority: number[] = []; // most-recently pressed first
let lastStepTime = 0;

/**
 * Called every frame from the Pixi ticker.
 * If a direction key is held and enough time has passed since the
 * last step, executes the next tile-step immediately.
 */
export function pollMovement() {
  if (heldDirections.size === 0) return;
  if (
    gameSession.connectionState !== "connected" &&
    gameSession.connectionState !== "authenticated"
  )
    return;

  let heading: number | undefined;
  for (const h of directionPriority) {
    if (heldDirections.has(h)) {
      heading = h;
      break;
    }
  }
  if (heading === undefined) return;

  const now = performance.now();
  if (now - lastStepTime >= WALK_STEP_MS) {
    doMove(heading, now);
    lastStepTime = now;
  }
}

function doMove(heading: number, now: number) {
  const { x, y } = gameState.hud.pos;
  const dx = heading === 3 ? 1 : heading === 4 ? -1 : 0;
  const dy = heading === 2 ? 1 : heading === 1 ? -1 : 0;
  const nx = x + dx;
  const ny = y + dy;

  if (mapState.isTileBlocked(nx, ny)) {
    sendChangeHeading(heading);
    return;
  }

  for (const [, npc] of gameState.remoteNpcs) {
    if (npc.x === nx && npc.y === ny) {
      sendChangeHeading(heading);
      return;
    }
  }

  for (const [, e] of gameState.remoteEntities) {
    if (e.x === nx && e.y === ny && !e.dead) {
      sendChangeHeading(heading);
      return;
    }
  }

  const tick = gameState.nextMoveTick();
  gameState.predictionBuffer.record(tick, { heading }, { x: nx, y: ny });
  const moveId = gameState.inputSender.record(tick, { heading });
  gameState.mergeHud({ pos: { x: nx, y: ny }, heading });

  gameState.playerMoveAnim = {
    startedAt: now,
    durationMs: WALK_STEP_MS,
    dx,
    dy,
  };

  sendPosition(heading, moveId);
}

/** Call from keydown handler in GameView. */
export function pressDirection(heading: number) {
  if (!heldDirections.has(heading)) {
    heldDirections.add(heading);
    directionPriority = [heading, ...directionPriority.filter((h) => h !== heading)];
  }
  // If this is the first key pressed, allow immediate movement.
  if (heldDirections.size === 1) {
    lastStepTime = 0;
  }
}

/** Call from keyup handler in GameView. */
export function releaseDirection(heading: number) {
  heldDirections.delete(heading);
  directionPriority = directionPriority.filter((h) => h !== heading);
}

/** Call on blur / disconnect to clear all held keys. */
export function releaseAllDirections() {
  heldDirections.clear();
  directionPriority = [];
}
