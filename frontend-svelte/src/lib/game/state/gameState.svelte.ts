import type {
  InventoryItem,
  SpellEntry,
  TradeState,
  CraftingState,
  MarketState,
  RetosState,
  BailOffer,
  PartyHudMember,
  ClanHudMember,
} from "$lib/game/lib/aowProtocol";
import { InterpolationBuffer } from "$lib/game/network/interpolation";
import { PredictionBuffer, type SimulateFn } from "$lib/game/network/prediction";
import { InputSender } from "$lib/game/network/inputSender";

export interface HudState {
  id: number;
  name: string;
  clan: string;
  map: number;
  pos: { x: number; y: number };
  heading: number;
  idBody: number;
  idHead: number;
  idWeapon: number;
  idHelmet: number;
  idShield: number;
  level: number;
  exp: number;
  expNextLevel: number;
  hp: number;
  maxHp: number;
  mana: number;
  maxMana: number;
  gold: number;
  dead: boolean;
  navegando: boolean;
  inmovilizado: boolean;
  paralizado: boolean;
  zonaSegura: number;
  seguroActivado: boolean;
  seguroClanActivado: boolean;
  attrFuerza: number;
  attrAgilidad: number;
  attrInteligencia: number;
  attrConstitucion: number;
  minHit: number;
  maxHit: number;
  nameColor: number;
  inventory: InventoryItem[];
  spells: SpellEntry[];
  partyMembers: PartyHudMember[];
  clanMembers: ClanHudMember[];
}

/**
 * Time (ms) a character takes to slide from one tile to the next.
 * Matches the original game's `walkStepMs` (200ms → 5 tiles/s).
 */
export const WALK_STEP_MS = 200;

/**
 * Tracks the visual interpolation of the local player between tiles.
 * Mirrors the original engine's startCharacterMovement / moveOffsetX/Y system.
 *
 * When the player begins a step the logical position (hud.pos) jumps to the
 * target tile immediately, but `moveOffsetX/Y` start at -TILE_SIZE*delta and
 * linearly interpolate to 0 over `durationMs`.  The renderer adds these offsets
 * to produce a smooth slide.
 */
export interface PlayerMoveAnim {
  /** performance.now() when the slide started */
  startedAt: number;
  /** how long the slide takes (ms) — matches walkStepMs */
  durationMs: number;
  /** tile delta (-1, 0, or 1) on each axis */
  dx: number;
  dy: number;
}

export interface ChatMessage {
  from: string;
  text: string;
  color: string;
  timestamp: number;
}

export interface ConsoleMessage {
  text: string;
  color: string;
  source: string;
  timestamp: number;
}

function createDefaultHud(): HudState {
  return {
    id: 0,
    name: "",
    clan: "",
    map: 0,
    pos: { x: 0, y: 0 },
    heading: 3,
    idBody: 0,
    idHead: 0,
    idWeapon: 0,
    idHelmet: 0,
    idShield: 0,
    level: 1,
    exp: 0,
    expNextLevel: 0,
    hp: 0,
    maxHp: 0,
    mana: 0,
    maxMana: 0,
    gold: 0,
    dead: false,
    navegando: false,
    inmovilizado: false,
    paralizado: false,
    zonaSegura: 0,
    seguroActivado: false,
    seguroClanActivado: false,
    attrFuerza: 0,
    attrAgilidad: 0,
    attrInteligencia: 0,
    attrConstitucion: 0,
    minHit: 0,
    maxHit: 0,
    nameColor: 0,
    inventory: [],
    spells: [],
    partyMembers: [],
    clanMembers: [],
  };
}

export interface RemoteEntity {
  id: number;
  name: string;
  x: number;
  y: number;
  heading: number;
  hp: number;
  maxHp: number;
  dead: boolean;
  level: number;
  nameColor?: number;
  bodyGrh?: number;
  headGrh?: number;
  weaponGrh?: number;
  shieldGrh?: number;
  helmetGrh?: number;
  chatText?: string;
  chatTextExpireAt?: number;
}

export interface GroundItem {
  x: number;
  y: number;
  itemId: number;
  amount: number;
  grhIndex: number;
}

export interface ProjectileEvent {
  startX: number;
  startY: number;
  endX: number;
  endY: number;
  type: 'arrow' | 'spell';
  spellId?: number;
}

export interface RemoteNpc {
  id: number;
  npcType: number;
  x: number;
  y: number;
  heading: number;
  hp: number;
  maxHp: number;
  dead: boolean;
}

class GameState {
  hud: HudState = $state(createDefaultHud());
  chatMessages: ChatMessage[] = $state([]);
  consoleMessages: ConsoleMessage[] = $state([]);
  mapName: string = $state("");
  tradeState: TradeState | null = $state(null);
  craftingState: CraftingState | null = $state(null);
  marketState: MarketState | null = $state(null);
  retosState: RetosState | null = $state(null);
  bailOffer: BailOffer | null = $state(null);
  castBar: { entityId: number; startMs: number; durationMs: number } | null = $state(null);
  pendingSpellSlot: number | null = $state(null);
  /** Active slide animation for the local player (null when idle). */
  playerMoveAnim: PlayerMoveAnim | null = $state(null);
  showCharacterStats: boolean = $state(false);
  showNpcInspector: number | null = $state(null);
  showAdminIntervals: boolean = $state(false);
  showOverview: boolean = $state(false);
  showDebugOverlay: boolean = $state(false);
  onlineCount: number = $state(0);
  sceneReady: boolean = $state(false);
  pingMs: number | null = $state(null);
  pingText: string = $state("Ping: -- ms");
  remoteEntities: Map<number, RemoteEntity> = $state(new Map());
  remoteNpcs: Map<number, RemoteNpc> = $state(new Map());
  groundItems: Map<string, GroundItem> = $state(new Map());

  readonly interpolationBuffers = new Map<number, InterpolationBuffer<{ x: number; y: number; heading: number }>>();
  readonly predictionBuffer = new PredictionBuffer<{ heading: number }, { x: number; y: number }>({ historyCapacity: 128, maxReplaySteps: 32 });
  readonly inputSender = new InputSender<{ heading: number }>({ historyCapacity: 64, redundancy: 3 });
  private _moveTick = 0;

  mergeHud(partial: Partial<HudState>) {
    this.hud = { ...this.hud, ...partial };
  }

  nextMoveTick(): number {
    return ++this._moveTick;
  }

  getOrCreateInterpolationBuffer(id: number): InterpolationBuffer<{ x: number; y: number; heading: number }> {
    let buf = this.interpolationBuffers.get(id);
    if (!buf) {
      buf = new InterpolationBuffer({ capacity: 32, baseDelayTicks: 2, smoothing: 0.1 });
      this.interpolationBuffers.set(id, buf);
    }
    return buf;
  }

  private static readonly movementSimulate: SimulateFn<{ heading: number }, { x: number; y: number }> = (state, _tick, input) => {
    const dx = input.heading === 3 ? 1 : input.heading === 4 ? -1 : 0;
    const dy = input.heading === 2 ? 1 : input.heading === 1 ? -1 : 0;
    return { x: state.x + dx, y: state.y + dy };
  };

  applyServerPosition(moveId: number, x: number, y: number, heading: number) {
    if (moveId > 0) {
      const report = this.predictionBuffer.reconcile(moveId, { x, y }, GameState.movementSimulate);
      this.inputSender.acknowledge({ serverTick: moveId, acknowledgedSequence: moveId });
      this.mergeHud({ pos: report.correctedState, heading });
    } else {
      const frames = this.predictionBuffer.getFrames();
      if (frames.length > 0) {
        const oldestTick = frames[0]!.tick;
        const report = this.predictionBuffer.reconcile(oldestTick, { x, y }, GameState.movementSimulate);
        this.inputSender.acknowledge({ serverTick: oldestTick, acknowledgedSequence: oldestTick });
        this.mergeHud({ pos: report.correctedState, heading });
      } else {
        this.mergeHud({ pos: { x, y }, heading });
      }
    }
  }

  resetNetcodeBuffers() {
    this.interpolationBuffers.clear();
    this.predictionBuffer.reset(0);
    this.inputSender.reset();
    this._moveTick = 0;
    this.playerMoveAnim = null;
  }

  addChat(from: string, text: string, color = "#d6d3d1") {
    this.chatMessages = [
      ...this.chatMessages,
      { from, text, color, timestamp: Date.now() },
    ].slice(-200);
  }

  addConsole(text: string, color = "#d6d3d1", source = "system") {
    this.consoleMessages = [
      ...this.consoleMessages,
      { text, color, source, timestamp: Date.now() },
    ].slice(-200);
  }

  /** Show a chat bubble over an entity sprite. Expires after 5 seconds. */
  setEntityChatText(entityId: number, text: string) {
    const entity = this.remoteEntities.get(entityId);
    if (entity) {
      entity.chatText = text;
      entity.chatTextExpireAt = Date.now() + 5000;
      this.remoteEntities.set(entityId, { ...entity });
    }
  }

  setInventory(items: InventoryItem[]) {
    this.hud = { ...this.hud, inventory: items };
  }

  updateInventorySlot(slot: number, item: InventoryItem | null) {
    const inv = [...this.hud.inventory];
    const idx = inv.findIndex((i) => i.slot === slot);
    if (item) {
      if (idx >= 0) inv[idx] = item;
      else inv.push(item);
    } else if (idx >= 0) {
      inv.splice(idx, 1);
    }
    this.hud = { ...this.hud, inventory: inv };
  }

  setSpells(spells: SpellEntry[]) {
    this.hud = { ...this.hud, spells };
  }

  upsertSpell(spell: SpellEntry) {
    const spells = [...this.hud.spells];
    const idx = spells.findIndex((s) => s.slot === spell.slot);
    if (idx >= 0) spells[idx] = spell;
    else spells.push(spell);
    this.hud = { ...this.hud, spells };
  }

  upsertNpc(npc: RemoteNpc) {
    const next = new Map(this.remoteNpcs);
    next.set(npc.id, npc);
    this.remoteNpcs = next;
  }

  removeNpc(id: number) {
    const next = new Map(this.remoteNpcs);
    next.delete(id);
    this.remoteNpcs = next;
    this.interpolationBuffers.delete(id);
  }

  upsertEntity(entity: RemoteEntity) {
    const next = new Map(this.remoteEntities);
    next.set(entity.id, entity);
    this.remoteEntities = next;
  }

  moveEntity(id: number, x: number, y: number, heading: number, serverTick?: number) {
    const e = this.remoteEntities.get(id);
    if (e) {
      const next = new Map(this.remoteEntities);
      next.set(id, { ...e, x, y, heading });
      this.remoteEntities = next;
      const buf = this.getOrCreateInterpolationBuffer(id);
      const tick = serverTick ?? Math.round(performance.now() / 16.67);
      buf.insert(tick, { x, y, heading }, performance.now());
      return;
    }
    const n = this.remoteNpcs.get(id);
    if (n) {
      const next = new Map(this.remoteNpcs);
      next.set(id, { ...n, x, y, heading });
      this.remoteNpcs = next;
      const buf = this.getOrCreateInterpolationBuffer(id);
      const tick = serverTick ?? Math.round(performance.now() / 16.67);
      buf.insert(tick, { x, y, heading }, performance.now());
    }
  }

  addGroundItem(item: GroundItem) {
    const next = new Map(this.groundItems);
    next.set(`${item.x},${item.y}`, item);
    this.groundItems = next;
  }

  removeGroundItem(x: number, y: number) {
    const next = new Map(this.groundItems);
    next.delete(`${x},${y}`);
    this.groundItems = next;
  }

  setEntityNameColor(id: number, colorCode: number) {
    const e = this.remoteEntities.get(id);
    if (e) {
      const next = new Map(this.remoteEntities);
      next.set(id, { ...e, nameColor: colorCode });
      this.remoteEntities = next;
    }
  }

  setEntityEquipment(id: number, slot: 'bodyGrh' | 'headGrh' | 'weaponGrh' | 'shieldGrh' | 'helmetGrh', grhId: number) {
    if (id === this.hud.id) {
      const hudSlotMap: Record<string, keyof HudState> = {
        bodyGrh: 'idBody',
        headGrh: 'idHead',
        weaponGrh: 'idWeapon',
        helmetGrh: 'idHelmet',
        shieldGrh: 'idShield',
      };
      const hudKey = hudSlotMap[slot];
      if (hudKey) {
        this.mergeHud({ [hudKey]: grhId } as Partial<HudState>);
      }
      return;
    }
    const e = this.remoteEntities.get(id);
    if (e) {
      const next = new Map(this.remoteEntities);
      next.set(id, { ...e, [slot]: grhId });
      this.remoteEntities = next;
    }
  }

  removeEntity(id: number) {
    const next = new Map(this.remoteEntities);
    next.delete(id);
    this.remoteEntities = next;
    this.interpolationBuffers.delete(id);
  }

  projectileCallbacks: Array<(p: ProjectileEvent) => void> = [];
  fxCallbacks: Array<(entityId: number, fxGrh: number) => void> = [];
  soundCallbacks: Array<(soundId: number) => void> = [];

  onProjectile(cb: (p: ProjectileEvent) => void) { this.projectileCallbacks.push(cb); }
  onFx(cb: (entityId: number, fxGrh: number) => void) { this.fxCallbacks.push(cb); }
  onSound(cb: (soundId: number) => void) { this.soundCallbacks.push(cb); }

  addProjectile(p: ProjectileEvent) {
    for (const cb of this.projectileCallbacks) cb(p);
  }

  addFx(entityId: number, fxGrh: number) {
    for (const cb of this.fxCallbacks) cb(entityId, fxGrh);
  }

  addSound(soundId: number) {
    for (const cb of this.soundCallbacks) cb(soundId);
  }

  reset() {
    this.hud = createDefaultHud();
    this.chatMessages = [];
    this.consoleMessages = [];
    this.mapName = "";
    this.tradeState = null;
    this.craftingState = null;
    this.marketState = null;
    this.retosState = null;
    this.bailOffer = null;
    this.castBar = null;
    this.showCharacterStats = false;
    this.showNpcInspector = null;
    this.showAdminIntervals = false;
    this.showOverview = false;
    this.showDebugOverlay = false;
    this.onlineCount = 0;
    this.sceneReady = false;
    this.pingMs = null;
    this.pingText = "Ping: -- ms";
    this.remoteEntities = new Map();
    this.remoteNpcs = new Map();
    this.groundItems = new Map();
    this.resetNetcodeBuffers();
  }
}

export const gameState = new GameState();
