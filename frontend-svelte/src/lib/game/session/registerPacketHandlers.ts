import { CLIENT_PACKET_ID, PacketReader } from "@openao/protocol";
import { gameSession } from "./gameSession.svelte";
import { gameState } from "../state/gameState.svelte";
import { toastStore } from "../state/toastStore.svelte";
import { particleEngine } from "../rendering/particleSystem";
import { GameSoundManager } from "../lib/sound";
import { preloadAdjacentMaps, resetPreloadState, evictDistantMaps } from "../utils/assetPreloader";
import { loadMapData } from "../utils/gameLoader";

let soundManager: GameSoundManager | null = null;
function getSoundManager(): GameSoundManager {
  if (!soundManager) {
    soundManager = new GameSoundManager();
  }
  return soundManager;
}

export function registerAllPacketHandlers() {
  gameSession.onPacket(CLIENT_PACKET_ID.getMyCharacter, readGetMyCharacter);
  gameSession.onPacket(CLIENT_PACKET_ID.getCharacter, readGetCharacter);
  gameSession.onPacket(CLIENT_PACKET_ID.pong, readPong);
  gameSession.onPacket(CLIENT_PACKET_ID.dialog, readDialog);
  gameSession.onPacket(CLIENT_PACKET_ID.console, readConsole);
  gameSession.onPacket(CLIENT_PACKET_ID.error, readError);
  gameSession.onPacket(CLIENT_PACKET_ID.updateHP, readUpdateHP);
  gameSession.onPacket(CLIENT_PACKET_ID.tUpdateHP, readTUpdateHP);
  gameSession.onPacket(CLIENT_PACKET_ID.updateMaxHP, readUpdateMaxHP);
  gameSession.onPacket(CLIENT_PACKET_ID.updateMana, readUpdateMana);
  gameSession.onPacket(CLIENT_PACKET_ID.tUpdateMana, readTUpdateMana);
  gameSession.onPacket(CLIENT_PACKET_ID.actExp, readActExp);
  gameSession.onPacket(CLIENT_PACKET_ID.actMyLevel, readActMyLevel);
  gameSession.onPacket(CLIENT_PACKET_ID.actGold, readActGold);
  gameSession.onPacket(CLIENT_PACKET_ID.actPosition, readActPosition);
  gameSession.onPacket(CLIENT_PACKET_ID.moveEntity, readMoveEntity);
  gameSession.onPacket(CLIENT_PACKET_ID.changeHeading, readChangeHeading);
  gameSession.onPacket(CLIENT_PACKET_ID.deleteCharacter, readDeleteCharacter);
  gameSession.onPacket(CLIENT_PACKET_ID.telepMe, readTelepMe);
  gameSession.onPacket(CLIENT_PACKET_ID.nameMap, readNameMap);
  gameSession.onPacket(CLIENT_PACKET_ID.actOnline, readActOnline);
  gameSession.onPacket(CLIENT_PACKET_ID.playSound, readPlaySound);
  gameSession.onPacket(CLIENT_PACKET_ID.actColorName, readActColorName);

  gameSession.onPacket(CLIENT_PACKET_ID.getNpc, readGetNpc);
  gameSession.onPacket(CLIENT_PACKET_ID.agregarUserInvItem, readAddInvItem);
  gameSession.onPacket(CLIENT_PACKET_ID.quitarUserInvItem, readRemoveInvItem);
  gameSession.onPacket(CLIENT_PACKET_ID.aprenderSpell, readLearnSpell);

  gameSession.onPacket(CLIENT_PACKET_ID.changeRopa, readChangeRopa);
  gameSession.onPacket(CLIENT_PACKET_ID.changeHelmet, readChangeHelmet);
  gameSession.onPacket(CLIENT_PACKET_ID.changeWeapon, readChangeWeapon);
  gameSession.onPacket(CLIENT_PACKET_ID.changeShield, readChangeShield);
  gameSession.onPacket(CLIENT_PACKET_ID.changeBody, readChangeBody);

  gameSession.onPacket(CLIENT_PACKET_ID.openTrade, readOpenTrade);
  gameSession.onPacket(CLIENT_PACKET_ID.closeTrade, readCloseTrade);
  gameSession.onPacket(CLIENT_PACKET_ID.closeForce, readCloseForce);
  gameSession.onPacket(CLIENT_PACKET_ID.openCrafting, readOpenCrafting);
  gameSession.onPacket(CLIENT_PACKET_ID.openMarket, readOpenMarket);
  gameSession.onPacket(CLIENT_PACKET_ID.openBail, readOpenBail);
  gameSession.onPacket(CLIENT_PACKET_ID.startCastBar, readStartCastBar);
  gameSession.onPacket(CLIENT_PACKET_ID.stopCastBar, readStopCastBar);

  gameSession.onPacket(CLIENT_PACKET_ID.selfVitalsDelta, readSelfVitalsDelta);
  gameSession.onPacket(CLIENT_PACKET_ID.selfFlagsDelta, readSelfFlagsDelta);
  gameSession.onPacket(CLIENT_PACKET_ID.selfMapMetaDelta, readSelfMapMetaDelta);

  gameSession.onPacket(CLIENT_PACKET_ID.characterStatsSnapshot, readCharacterStatsSnapshot);
  gameSession.onPacket(CLIENT_PACKET_ID.entityVitalsDelta, readEntityVitalsDelta);
  gameSession.onPacket(CLIENT_PACKET_ID.animFX, readAnimFx);
  gameSession.onPacket(CLIENT_PACKET_ID.updateAgilidad, readUpdateAgilidad);
  gameSession.onPacket(CLIENT_PACKET_ID.updateFuerza, readUpdateFuerza);
  gameSession.onPacket(CLIENT_PACKET_ID.createProjectile, readCreateProjectile);
  gameSession.onPacket(CLIENT_PACKET_ID.spellProjectile, readSpellProjectile);
  gameSession.onPacket(CLIENT_PACKET_ID.spellVisual, readSpellVisual);
  gameSession.onPacket(CLIENT_PACKET_ID.renderItem, readRenderItem);
  gameSession.onPacket(CLIENT_PACKET_ID.deleteItem, readDeleteItem);
  gameSession.onPacket(CLIENT_PACKET_ID.inmo, readInmo);
  gameSession.onPacket(CLIENT_PACKET_ID.putBodyAndHeadDead, readDeath);
  gameSession.onPacket(CLIENT_PACKET_ID.revivirUsuario, readRevive);
  gameSession.onPacket(CLIENT_PACKET_ID.globalNotice, readGlobalNotice);
  gameSession.onPacket(CLIENT_PACKET_ID.blockMap, readBlockMap);
  gameSession.onPacket(CLIENT_PACKET_ID.partyState, readPartyState);
  gameSession.onPacket(CLIENT_PACKET_ID.clanState, readClanState);
  gameSession.onPacket(CLIENT_PACKET_ID.closeBail, readCloseBail);
  gameSession.onPacket(CLIENT_PACKET_ID.navegando, readNavegando);
  gameSession.onPacket(CLIENT_PACKET_ID.tInmo, readTInmo);
  gameSession.onPacket(CLIENT_PACKET_ID.changeArrow, readChangeArrow);
  gameSession.onPacket(CLIENT_PACKET_ID.openRetos, readOpenRetos);
}

function readGetMyCharacter(r: PacketReader) {
  const id = r.getShort();
  const map = r.getShort();
  const x = r.getShort();
  const y = r.getShort();
  const heading = r.getByte();
  const name = r.getString();
  const hp = r.getShort();
  const maxHp = r.getShort();
  const dead = r.getByte() === 1;
  const level = r.getShort();

  gameState.resetNetcodeBuffers();
  gameState.mergeHud({
    id,
    name,
    map,
    pos: { x, y },
    heading,
    hp,
    maxHp,
    dead,
    level,
  });

  gameState.addConsole(`Conectado como ${name} en mapa ${map}`, "#fcd34d", "system");
  gameState.sceneReady = true;

  resetPreloadState();
  loadMapData(map).then((mapData) => {
    preloadAdjacentMaps(mapData, map);
    evictDistantMaps();
  }).catch(() => {});
}

function readGetCharacter(r: PacketReader) {
  const entityId = r.getShort();
  const x = r.getShort();
  const y = r.getShort();
  const heading = r.getByte();
  const name = r.getString();
  const hp = r.getShort();
  const maxHp = r.getShort();
  const dead = r.getByte() === 1;
  const level = r.getShort();

  gameState.upsertEntity({
    id: entityId,
    name,
    x,
    y,
    heading,
    hp,
    maxHp,
    dead,
    level,
  });

  gameState.addConsole(
    `Personaje '${name}' visible (entity=${entityId}, pos=${x},${y}, lvl=${level})`,
    "#94a3b8",
    "system",
  );
}

let pingSentAt = 0;
let pingToken = 0;

export function sendPingWithTimestamp() {
  pingToken = (pingToken + 1) & 0xffff;
  pingSentAt = performance.now();
  gameSession.sendPing(pingToken);
}

function readPong(r: PacketReader) {
  const _token = r.getInt();
  if (pingSentAt > 0) {
    const ms = Math.round(performance.now() - pingSentAt);
    gameState.pingMs = ms;
    gameState.pingText = `Ping: ${ms} ms`;
    pingSentAt = 0;
  }
}

function readDialog(r: PacketReader) {
  const text = r.getString();
  gameState.addChat("Chat", text, "#d6d3d1");
}

function readConsole(r: PacketReader) {
  const text = r.getString();
  gameState.addConsole(text, "#9ca3af", "server");
}

function readError(r: PacketReader) {
  const text = r.getString();
  gameState.addConsole(text, "#ef4444", "error");
}

function readUpdateHP(r: PacketReader) {
  const hp = r.getShort();
  gameState.mergeHud({ hp });
}

function readTUpdateHP(r: PacketReader) {
  const hp = r.getShort();
  gameState.mergeHud({ hp });
}

function readUpdateMaxHP(r: PacketReader) {
  const hp = r.getShort();
  const _tHp = r.getShort();
  const maxHp = r.getShort();
  gameState.mergeHud({ hp, maxHp });
}

function readUpdateMana(r: PacketReader) {
  const mana = r.getShort();
  gameState.mergeHud({ mana });
}

function readTUpdateMana(r: PacketReader) {
  const mana = r.getShort();
  gameState.mergeHud({ mana });
}

function readActExp(r: PacketReader) {
  const exp = r.getInt();
  const expNextLevel = r.remainingBytes >= 4 ? r.getInt() : 0;
  gameState.mergeHud({ exp, ...(expNextLevel > 0 ? { expNextLevel } : {}) });
}

function readActMyLevel(r: PacketReader) {
  const level = r.getShort();
  gameState.mergeHud({ level });
  gameState.addConsole(`Has subido al nivel ${level}!`, "#fbbf24", "system");
  toastStore.levelUp(level);
  particleEngine.emit("levelup", window.innerWidth / 2, window.innerHeight / 2);
}

function readActGold(r: PacketReader) {
  const gold = r.getInt();
  gameState.mergeHud({ gold });
}

function readActPosition(r: PacketReader) {
  const id = r.getShort();
  const x = r.getShort();
  const y = r.getShort();
  const moveId = r.remainingBytes >= 2 ? r.getShort() : 0;
  if (id === gameState.hud.id) {
    gameState.applyServerPosition(moveId, x, y, gameState.hud.heading);
  }
}

function readMoveEntity(r: PacketReader) {
  const id = r.getShort();
  const x = r.getShort();
  const y = r.getShort();
  const heading = r.getByte();
  const serverTick = r.remainingBytes >= 2 ? r.getShort() : 0;
  if (id === gameState.hud.id) {
    gameState.applyServerPosition(0, x, y, heading);
  } else {
    gameState.moveEntity(id, x, y, heading, serverTick || undefined);
  }
}

function readChangeHeading(r: PacketReader) {
  const id = r.getShort();
  const heading = r.getByte();
  if (id === gameState.hud.id) {
    gameState.mergeHud({ heading });
  }
}

function readGetNpc(r: PacketReader) {
  const id = r.getShort();
  const x = r.getShort();
  const y = r.getShort();
  const heading = r.getByte();
  const npcType = r.getShort();
  const hp = r.getShort();
  const maxHp = r.getShort();

  gameState.upsertNpc({ id, npcType, x, y, heading, hp, maxHp, dead: hp <= 0 });
}

function readDeleteCharacter(r: PacketReader) {
  const id = r.getShort();
  gameState.removeEntity(id);
  gameState.removeNpc(id);
}

function readTelepMe(r: PacketReader) {
  const map = r.getShort();
  const x = r.getShort();
  const y = r.getShort();
  const heading = r.getByte();
  gameState.remoteEntities = new Map();
  gameState.remoteNpcs = new Map();
  gameState.groundItems = new Map();
  gameState.resetNetcodeBuffers();
  gameState.mergeHud({ map, pos: { x, y }, heading });
  gameState.addConsole(`Teletransportado al mapa ${map}`, "#c084fc", "system");

  loadMapData(map).then((mapData) => {
    preloadAdjacentMaps(mapData, map);
    evictDistantMaps();
  }).catch(() => {});
}

function readNameMap(r: PacketReader) {
  const name = r.getString();
  gameState.mapName = name;
}

function readActOnline(r: PacketReader) {
  const count = r.getShort();
  gameState.onlineCount = count;
}

function readPlaySound(r: PacketReader) {
  const soundId = r.getShort();
  getSoundManager().play({ soundId });
}

function readActColorName(r: PacketReader) {
  const id = r.getShort();
  const colorCode = r.getByte();
  if (id === gameState.hud.id) {
    gameState.mergeHud({ nameColor: colorCode });
    return;
  }
  gameState.setEntityNameColor(id, colorCode);
}

function readAddInvItem(r: PacketReader) {
  const slot = r.getByte();
  const idItem = r.getShort();
  const name = r.getString();
  const amount = r.getShort();
  const equipped = r.getByte() === 1;
  const grhIndex = r.getShort();
  const objType = r.getByte();
  const maxHit = r.getShort();
  const minHit = r.getShort();
  const maxDef = r.getShort();
  const minDef = r.getShort();
  const value = r.getInt();

  gameState.updateInventorySlot(slot, {
    slot,
    idItem,
    name,
    amount,
    equipped,
    grhIndex,
    objType,
    value,
    validForUser: true,
    details: `ATK: ${minHit}-${maxHit} DEF: ${minDef}-${maxDef}`,
    minHit,
    maxHit,
  } as any);
}

function readRemoveInvItem(r: PacketReader) {
  const slot = r.getByte();
  gameState.updateInventorySlot(slot, null);
}

function readLearnSpell(r: PacketReader) {
  const slot = r.getByte();
  const idSpell = r.getShort();
  const name = r.getString();
  const manaRequired = r.getShort();
  gameState.upsertSpell({ slot, idSpell, name, manaRequired });
  gameState.addConsole(`Hechizo aprendido: ${name}`, "#c084fc", "system");
}

function readChangeRopa(r: PacketReader) {
  const entityId = r.getShort();
  const grhId = r.getShort();
  gameState.setEntityEquipment(entityId, 'headGrh', grhId);
}

function readChangeHelmet(r: PacketReader) {
  const entityId = r.getShort();
  const grhId = r.getShort();
  gameState.setEntityEquipment(entityId, 'helmetGrh', grhId);
}

function readChangeWeapon(r: PacketReader) {
  const entityId = r.getShort();
  const grhId = r.getShort();
  gameState.setEntityEquipment(entityId, 'weaponGrh', grhId);
}

function readChangeShield(r: PacketReader) {
  const entityId = r.getShort();
  const grhId = r.getShort();
  gameState.setEntityEquipment(entityId, 'shieldGrh', grhId);
}

function readChangeBody(r: PacketReader) {
  const entityId = r.getShort();
  const grhId = r.getShort();
  gameState.setEntityEquipment(entityId, 'bodyGrh', grhId);
}

function readOpenTrade(r: PacketReader) {
  const count = r.getByte();
  const items: import("$lib/game/lib/aowProtocol").TradeItem[] = [];
  for (let i = 0; i < count; i++) {
    const itemId = r.getShort();
    const name = r.getString();
    const value = r.getInt();
    const grhIndex = r.getShort();
    items.push({
      slot: i,
      name,
      grhIndex,
      amount: 1,
      value,
      validForUser: true,
      details: `Item #${itemId}`,
    });
  }
  gameState.tradeState = {
    mode: "merchant",
    merchantItems: items,
    playerItems: [],
  };
  gameState.addConsole("Comercio abierto", "#fbbf24", "system");
}

function readCloseTrade(_r: PacketReader) {
  gameState.tradeState = null;
  gameState.addConsole("Comercio cerrado", "#9ca3af", "system");
}

function readCloseForce(_r: PacketReader) {
  gameState.tradeState = null;
  gameState.craftingState = null;
  gameState.marketState = null;
  gameState.retosState = null;
  gameState.bailOffer = null;
}

function readOpenCrafting(r: PacketReader) {
  const json = r.getString();
  try {
    const data = JSON.parse(json);
    gameState.craftingState = {
      profession: data.profession ?? '',
      title: data.title ?? 'Crafteo',
      recipes: data.recipes ?? [],
    };
    gameState.addConsole(`Crafteo abierto: ${data.title ?? 'Crafteo'}`, "#fbbf24", "system");
  } catch {
    gameState.addConsole("Crafteo abierto", "#fbbf24", "system");
  }
}

function readOpenMarket(r: PacketReader) {
  const json = r.getString();
  try {
    gameState.marketState = JSON.parse(json);
  } catch {
    gameState.addConsole("Error al abrir el mercado.", "#ef4444", "system");
  }
}

function readStartCastBar(r: PacketReader) {
  const entityId = r.getShort();
  const durationMs = r.getInt();
  gameState.castBar = { entityId, startMs: Date.now(), durationMs };
}

function readStopCastBar(r: PacketReader) {
  const entityId = r.getShort();
  if (gameState.castBar?.entityId === entityId) {
    gameState.castBar = null;
  }
}

function readOpenBail(r: PacketReader) {
  const kills = r.getInt();
  const citizensKilled = r.getInt();
  const fianza = r.getInt();
  const goldRequired = r.getInt();
  const goldAvailable = r.getInt();
  const canPay = r.getByte() === 1;
  gameState.bailOffer = { kills, citizensKilled, fianza, goldRequired, goldAvailable, canPay };
}

function readSelfVitalsDelta(r: PacketReader) {
  const hp = r.getShort();
  const maxHp = r.getShort();
  const mana = r.getShort();
  const maxMana = r.getShort();
  gameState.mergeHud({ hp, maxHp, mana, maxMana });
}

function readSelfFlagsDelta(r: PacketReader) {
  const zonaSegura = r.getByte();
  const seguroActivado = r.getByte() === 1;
  const seguroClanActivado = r.getByte() === 1;
  gameState.mergeHud({ zonaSegura, seguroActivado, seguroClanActivado });
}

function readSelfMapMetaDelta(r: PacketReader) {
  const mapName = r.getString();
  gameState.mapName = mapName;
}

function readInmo(r: PacketReader) {
  const _inmo = r.getByte();
  const x = r.getShort();
  const y = r.getShort();
  gameState.mergeHud({ pos: { x, y }, inmovilizado: true });
}

function readDeath(r: PacketReader) {
  const id = r.getShort();
  const _head = r.getShort();
  const _body = r.getShort();
  const _helmet = r.getShort();
  const _weapon = r.getShort();
  const _shield = r.getShort();
  if (id === gameState.hud.id) {
    gameState.mergeHud({ dead: true });
    gameState.addConsole("Has muerto", "#ef4444", "system");
    toastStore.death();
    particleEngine.emit("death", window.innerWidth / 2, window.innerHeight / 2);
  } else {
    const e = gameState.remoteEntities.get(id);
    if (e) gameState.upsertEntity({ ...e, dead: true });
  }
}

function readRevive(r: PacketReader) {
  const id = r.getShort();
  const _head = r.getShort();
  const _body = r.getShort();
  if (id === gameState.hud.id) {
    gameState.mergeHud({ dead: false });
    gameState.addConsole("Has resucitado", "#4ade80", "system");
  } else {
    const e = gameState.remoteEntities.get(id);
    if (e) gameState.upsertEntity({ ...e, dead: false });
  }
}

function readGlobalNotice(r: PacketReader) {
  const text = r.getString();
  gameState.addConsole(text, "#fbbf24", "global");
  gameState.addChat("Aviso Global", text, "#fbbf24");
}

function readBlockMap(r: PacketReader) {
  const _x = r.getShort();
  const _y = r.getShort();
  const _blocked = r.getByte();
}

function readCharacterStatsSnapshot(r: PacketReader) {
  const attrFuerza = r.getShort();
  const attrAgilidad = r.getShort();
  const attrInteligencia = r.getShort();
  const attrConstitucion = r.getShort();
  if (r.remainingBytes >= 4) {
    const minHit = r.getShort();
    const maxHit = r.getShort();
    gameState.mergeHud({ attrFuerza, attrAgilidad, attrInteligencia, attrConstitucion, minHit, maxHit });
  } else {
    gameState.mergeHud({ attrFuerza, attrAgilidad, attrInteligencia, attrConstitucion });
  }
}

function readEntityVitalsDelta(r: PacketReader) {
  const entityId = r.getShort();
  const hp = r.getShort();
  const maxHp = r.getShort();
  const _mana = r.remainingBytes >= 2 ? r.getShort() : 0;
  const _maxMana = r.remainingBytes >= 2 ? r.getShort() : 0;
  const entity = gameState.remoteEntities.get(entityId);
  if (entity) {
    gameState.upsertEntity({ ...entity, hp, maxHp });
  }
  const npc = gameState.remoteNpcs.get(entityId);
  if (npc) {
    gameState.upsertNpc({ ...npc, hp, maxHp });
  }
}

function readAnimFx(r: PacketReader) {
  const entityId = r.getShort();
  const fxId = r.getShort();
  gameState.addFx(entityId, fxId);
  if (entityId === gameState.hud.id) {
    const isHeal = fxId >= 10 && fxId <= 20;
    particleEngine.emit(
      isHeal ? "heal" : "spell_hit",
      window.innerWidth / 2,
      window.innerHeight / 2,
    );
  }
}

function readRenderItem(r: PacketReader) {
  const x = r.getShort();
  const y = r.getShort();
  const itemId = r.getShort();
  const amount = r.getShort();
  const grhIndex = r.getShort();
  gameState.addGroundItem({ x, y, itemId, amount, grhIndex });
}

function readDeleteItem(r: PacketReader) {
  const x = r.getShort();
  const y = r.getShort();
  gameState.removeGroundItem(x, y);
}

function readPartyState(r: PacketReader) {
  const rawJson = r.getString();
  try {
    const delta = JSON.parse(rawJson) as {
      upsert: Array<{
        id: number;
        nameCharacter: string;
        map: number;
        pos: { x: number; y: number };
        online: boolean;
        isLeader: boolean;
      }>;
      remove: number[];
    };

    const current = gameState.hud.partyMembers ?? [];
    const next = new Map(current.map((m: any) => [String(m.id), m]));

    for (const member of delta.upsert) {
      next.set(String(member.id), member);
    }
    for (const id of delta.remove) {
      next.delete(String(id));
    }

    gameState.mergeHud({ partyMembers: [...next.values()] });
  } catch {
    // Ignore malformed party state
  }
}

function readClanState(r: PacketReader) {
  const rawJson = r.getString();
  try {
    const delta = JSON.parse(rawJson) as {
      upsert: Array<{
        id: number;
        nameCharacter: string;
        map: number;
        pos: { x: number; y: number };
        online: boolean;
      }>;
      remove: number[];
    };

    const current = gameState.hud.clanMembers ?? [];
    const next = new Map(current.map((m: any) => [String(m.id), m]));

    for (const member of delta.upsert) {
      next.set(String(member.id), member);
    }
    for (const id of delta.remove) {
      next.delete(String(id));
    }

    gameState.mergeHud({ clanMembers: [...next.values()] });
  } catch {
    // Ignore malformed clan state
  }
}

function readCreateProjectile(r: PacketReader) {
  const startX = r.getByte();
  const startY = r.getByte();
  const endX = r.getByte();
  const endY = r.getByte();
  const _grhIndex = r.getShort();
  gameState.addProjectile({ startX, startY, endX, endY, type: 'arrow' });
}

function readSpellProjectile(r: PacketReader) {
  const startX = r.getByte();
  const startY = r.getByte();
  const endX = r.getByte();
  const endY = r.getByte();
  const spellId = r.getShort();
  gameState.addProjectile({ startX, startY, endX, endY, type: 'spell', spellId });
}

function readSpellVisual(r: PacketReader) {
  const flags = r.getByte();
  const hasProjectile = (flags & 1) !== 0;
  const hasFx = (flags & (1 << 1)) !== 0;
  const hasSound = (flags & (1 << 2)) !== 0;
  const hasWords = (flags & (1 << 3)) !== 0;
  const hasTargetId = hasFx || hasSound;

  let startX = 0, startY = 0, endX = 0, endY = 0, spellId = 0;
  if (hasProjectile) {
    startX = r.getByte();
    startY = r.getByte();
    endX = r.getByte();
    endY = r.getByte();
    spellId = r.getShort();
  }

  let targetId = 0;
  if (hasTargetId) {
    targetId = r.getShort();
  }

  let fxGrh = 0;
  if (hasFx) {
    fxGrh = r.getShort();
  }

  let soundId = 0;
  if (hasSound) {
    soundId = r.getShort();
  }

  if (hasWords) {
    const _casterId = r.getShort();
    const _msg = r.getString();
  }

  if (hasProjectile) {
    gameState.addProjectile({ startX, startY, endX, endY, type: 'spell', spellId });
  }

  if (hasFx && targetId > 0) {
    gameState.addFx(targetId, fxGrh);
  }

  if (hasSound && soundId > 0) {
    gameState.addSound(soundId);
  }
}

function readUpdateAgilidad(r: PacketReader) {
  const boost = r.getShort();
  gameState.addConsole(`+${boost} agilidad`, '#4ade80');
}

function readUpdateFuerza(r: PacketReader) {
  const boost = r.getShort();
  gameState.addConsole(`+${boost} fuerza`, '#f59e0b');
}

function readCloseBail(_r: PacketReader) {
  gameState.bailOffer = null;
}

function readNavegando(r: PacketReader) {
  const navegando = r.getByte() === 1;
  gameState.mergeHud({ navegando });
}

function readTInmo(r: PacketReader) {
  const _restricted = r.getByte();
  const _x = r.getByte();
  const _y = r.getByte();
  if (r.remainingBytes >= 4) {
    const _durationMs = r.getInt();
  }
}

function readChangeArrow(r: PacketReader) {
  const _entityId = r.getShort();
  const _slot = r.getByte();
}

function readOpenRetos(r: PacketReader) {
  const json = r.getString();
  try {
    gameState.retosState = JSON.parse(json);
  } catch {
    gameState.addConsole("Error al abrir los retos.", "#ef4444", "system");
  }
}
