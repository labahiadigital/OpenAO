import type { GraphicsDB, ObjectsDB, SpellsDB, NPCsDB, BodiesDB, HeadsDB, BodyData, HeadData } from "$lib/game/types/game";
import { loadGraphicsDB, loadObjectsDB, loadSpellsDB, loadNPCsDB, loadBodiesDB, loadHeadsDB } from "$lib/game/utils/gameLoader";

class AssetStore {
  graphicsDB: GraphicsDB | null = $state(null);
  objectsDB: ObjectsDB | null = $state(null);
  spellsDB: SpellsDB | null = $state(null);
  npcsDB: NPCsDB | null = $state(null);
  bodiesDB: BodiesDB | null = $state(null);
  headsDB: HeadsDB | null = $state(null);
  loaded: boolean = $state(false);
  loading: boolean = $state(false);
  error: string | null = $state(null);

  async load() {
    if (this.loaded || this.loading) return;
    this.loading = true;
    this.error = null;

    try {
      const [graphicsDB, objectsDB, spellsDB, npcsDB, bodiesDB, headsDB] = await Promise.all([
        loadGraphicsDB(),
        loadObjectsDB(),
        loadSpellsDB(),
        loadNPCsDB(),
        loadBodiesDB(),
        loadHeadsDB(),
      ]);

      this.graphicsDB = graphicsDB;
      this.objectsDB = objectsDB;
      this.spellsDB = spellsDB;
      this.npcsDB = npcsDB;
      this.bodiesDB = bodiesDB;
      this.headsDB = headsDB;
      this.loaded = true;
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to load assets";
      console.error("[AssetStore] Failed to load:", e);
    } finally {
      this.loading = false;
    }
  }

  getItemGraphic(grhIndex: number) {
    if (!this.graphicsDB) return null;
    return this.graphicsDB[grhIndex.toString()] ?? null;
  }

  getNpcName(npcType: number): string {
    if (!this.npcsDB) return `NPC #${npcType}`;
    const npc = this.npcsDB[npcType.toString()];
    return npc?.name ?? `NPC #${npcType}`;
  }

  getNpcBodyHead(npcType: number): { idBody: number; idHead: number } | null {
    if (!this.npcsDB) return null;
    const npc = this.npcsDB[npcType.toString()];
    if (!npc) return null;
    return { idBody: npc.idBody ?? 0, idHead: npc.idHead ?? 0 };
  }

  getBodyGrhId(bodyId: number, heading: number): number {
    if (!this.bodiesDB) return 0;
    const body = this.bodiesDB[bodyId.toString()];
    if (!body) return 0;
    const key = heading.toString() as keyof BodyData;
    return (body[key] as number) ?? 0;
  }

  getHeadGrhId(headId: number, heading: number): number {
    if (!this.headsDB) return 0;
    const head = this.headsDB[headId.toString()];
    if (!head) return 0;
    const key = heading.toString() as keyof HeadData;
    return (head[key] as number) ?? 0;
  }

  getBodyData(bodyId: number): { headOffsetX: number; headOffsetY: number } | null {
    if (!this.bodiesDB) return null;
    const body = this.bodiesDB[bodyId.toString()];
    if (!body) return null;
    return {
      headOffsetX: body.headOffsetX ?? 0,
      headOffsetY: body.headOffsetY ?? 0,
    };
  }
}

export const assetStore = new AssetStore();
