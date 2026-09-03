import { query } from "../db";

export type RankingEntry = {
  name: string;
  level: number;
  exp: number;
  class_name: string;
  faction: string;
  clan_name: string | null;
};

export async function getRanking(limit = 100): Promise<RankingEntry[]> {
  const result = await query<RankingEntry>(
    `SELECT c.name, c.level, c.exp,
            cls.name as class_name, c.faction,
            cl.name as clan_name
     FROM characters c
     LEFT JOIN classes cls ON cls.id = c.id_clase
     LEFT JOIN clan_members cm ON cm.character_id = c.id
     LEFT JOIN clans cl ON cl.id = cm.clan_id
     WHERE c.deleted_at IS NULL
     ORDER BY c.level DESC, c.exp DESC
     LIMIT $1`,
    [limit],
  );
  return result.rows;
}
