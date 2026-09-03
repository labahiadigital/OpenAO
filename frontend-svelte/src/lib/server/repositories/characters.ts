import { query } from "../db";

export type CharacterRow = {
  id: string;
  account_id: string;
  name: string;
  level: number;
  map_id: number;
  id_clase: number;
  criminal: boolean;
  faction: string;
  clan_name: string | null;
  id_head: number;
  id_body: number;
  id_weapon: number;
  id_shield: number;
  id_helmet: number;
};

export async function findCharactersByAccountId(
  accountId: string,
): Promise<CharacterRow[]> {
  const result = await query<CharacterRow>(
    `SELECT c.id, c.account_id, c.name, c.level, c.map_id, c.id_clase,
            c.criminal, c.faction, cl.name as clan_name,
            c.id_head, c.id_body, c.id_weapon, c.id_shield, c.id_helmet
     FROM characters c
     LEFT JOIN clan_members cm ON cm.character_id = c.id
     LEFT JOIN clans cl ON cl.id = cm.clan_id
     WHERE c.account_id = $1 AND c.deleted_at IS NULL
     ORDER BY c.created_at ASC`,
    [accountId],
  );
  return result.rows;
}

export async function findCharacterById(
  characterId: string,
): Promise<CharacterRow | null> {
  const result = await query<CharacterRow>(
    `SELECT c.id, c.account_id, c.name, c.level, c.map_id, c.id_clase,
            c.criminal, c.faction, cl.name as clan_name,
            c.id_head, c.id_body, c.id_weapon, c.id_shield, c.id_helmet
     FROM characters c
     LEFT JOIN clan_members cm ON cm.character_id = c.id
     LEFT JOIN clans cl ON cl.id = cm.clan_id
     WHERE c.id = $1 AND c.deleted_at IS NULL`,
    [characterId],
  );
  return result.rows[0] ?? null;
}
