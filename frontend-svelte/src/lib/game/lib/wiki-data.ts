import type { PublicWikiResponse } from "$lib/game/lib/wiki";

export const WIKI_REVALIDATE_SECONDS = 60 * 60 * 24 * 30;

function inferEquipmentCategory(
    objType: number,
    name?: string,
): {
    category: "weapon" | "armor" | "shield" | "helmet" | "magic_weapon";
    categoryLabel: string;
} | null {
    const normalizedName = String(name ?? "")
        .normalize("NFD")
        .replace(/[\u0300-\u036f]/g, "")
        .toLowerCase();

    if (
        objType === 26 ||
        normalizedName.includes("baston") ||
        normalizedName.includes("laud") ||
        normalizedName.includes("flauta")
    ) {
        return { category: "magic_weapon", categoryLabel: "Armas Mágicas" };
    }

    switch (objType) {
        case 2:
            return { category: "weapon", categoryLabel: "Armas" };
        case 3:
            return { category: "armor", categoryLabel: "Armaduras" };
        case 16:
            return { category: "shield", categoryLabel: "Escudos" };
        case 17:
            return { category: "helmet", categoryLabel: "Cascos" };
        default:
            return null;
    }
}

function normalizeWikiResponse(payload: unknown): PublicWikiResponse {
    const raw = (payload ?? {}) as Record<string, unknown>;
    const rawEquipment = Array.isArray(raw.equipment) ? raw.equipment : [];
    const rawObjects = Array.isArray(raw.objects) ? raw.objects : [];
    const normalizedEquipment =
        rawEquipment.length > 0
            ? rawEquipment
            : rawObjects
                  .map((entry) => {
                      const item = entry as Record<string, unknown>;
                      const inferred = inferEquipmentCategory(
                          Number(item.objType ?? 0),
                          String(item.name ?? ""),
                      );

                      if (!inferred) {
                          return null;
                      }

                      return {
                          ...item,
                          grhIndex: Number(item.grhIndex ?? 0),
                          category: inferred.category,
                          categoryLabel: inferred.categoryLabel,
                      };
                  })
                  .filter(Boolean);

    const rawStats = (raw.stats ?? {}) as Record<string, unknown>;

    return {
        generatedAt: String(raw.generatedAt ?? new Date().toISOString()),
        stats: {
            npcCount: Number(rawStats.npcCount ?? 0),
            combatNpcCount: Number(rawStats.combatNpcCount ?? 0),
            equipmentCount: Number(
                rawStats.equipmentCount ??
                    rawStats.objectCount ??
                    normalizedEquipment.length,
            ),
            spellCount: Number(rawStats.spellCount ?? 0),
            trainingMapCount: Number(rawStats.trainingMapCount ?? 0),
        },
        npcs: Array.isArray(raw.npcs)
            ? (raw.npcs as PublicWikiResponse["npcs"])
            : [],
        equipment: normalizedEquipment as PublicWikiResponse["equipment"],
        spells: Array.isArray(raw.spells)
            ? (raw.spells as PublicWikiResponse["spells"])
            : [],
        trainingMaps: Array.isArray(raw.trainingMaps)
            ? (raw.trainingMaps as PublicWikiResponse["trainingMaps"])
            : [],
    };
}

function createEmptyWikiResponse(): PublicWikiResponse {
    return {
        generatedAt: new Date().toISOString(),
        stats: {
            npcCount: 0,
            combatNpcCount: 0,
            equipmentCount: 0,
            spellCount: 0,
            trainingMapCount: 0,
        },
        npcs: [],
        equipment: [],
        spells: [],
        trainingMaps: [],
    };
}

let cachedData: PublicWikiResponse | null = null;
let cacheTimestamp = 0;

export async function getWikiData(apiBaseUrl: string): Promise<PublicWikiResponse> {
    const now = Date.now();
    if (cachedData && now - cacheTimestamp < WIKI_REVALIDATE_SECONDS * 1000) {
        return cachedData;
    }

    try {
        const response = await fetch(`${apiBaseUrl}/wiki`, {
            signal: AbortSignal.timeout(5000),
        });

        if (!response.ok) {
            throw new Error(`HTTP ${response.status}`);
        }

        cachedData = normalizeWikiResponse(await response.json());
        cacheTimestamp = now;
        return cachedData;
    } catch (error) {
        console.error(
            `No se pudo cargar la wiki desde ${apiBaseUrl}:`,
            error,
        );
    }

    return createEmptyWikiResponse();
}
