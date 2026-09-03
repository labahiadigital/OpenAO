import type { Container as PixiContainer, Application } from "pixi.js";
import { TILE_SIZE } from "$lib/game/lib/viewport";
import { sendClick } from "$lib/game/session/outgoingRequests";

export type ViewBounds = {
  minX: number;
  maxX: number;
  minY: number;
  maxY: number;
  viewW: number;
  viewH: number;
};

export function updateCamera(
  app: Application,
  worldContainer: PixiContainer,
  px: number,
  py: number,
): void {
  const cw = app.screen.width;
  const ch = app.screen.height;
  worldContainer.x = cw / 2 - (px - 1) * TILE_SIZE - TILE_SIZE / 2;
  worldContainer.y = ch / 2 - (py - 1) * TILE_SIZE - TILE_SIZE / 2;
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
    sendClick(tileX, tileY);
  }
}
