use super::{CharacterData, CharacterSummary, Database};

impl Database {
    pub async fn load_character(
        &self,
        character_id: &str,
    ) -> Result<Option<CharacterData>, sqlx::Error> {
        let row = sqlx::query_as::<_, CharacterData>(
            r#"
            SELECT id, account_id, name, id_clase,
                   COALESCE(id_raza, 1) as id_raza,
                   map_id, pos_x, pos_y,
                   gold, hp, max_hp, mana, max_mana, level, dead, criminal,
                   faction, min_hit, max_hit, attr_fuerza, attr_agilidad,
                   attr_inteligencia, attr_constitucion, id_head, id_body,
                   id_helmet, id_weapon, id_shield,
                   COALESCE(id_arrow_slot, 0) as id_arrow_slot,
                   COALESCE(id_ring_slot, 0) as id_ring_slot,
                   navegando,
                   home_map, home_x, home_y, exp, exp_next_level,
                   COALESCE(faction_rank, 0) as faction_rank,
                   COALESCE(faction_score, 0) as faction_score,
                   COALESCE(faction_score_armada, 0) as faction_score_armada,
                   COALESCE(faction_score_caos, 0) as faction_score_caos,
                   COALESCE(criminales_matados, 0) as criminales_matados,
                   COALESCE(ciudadanos_matados, 0) as ciudadanos_matados
            FROM characters
            WHERE id = ?1 AND deleted_at IS NULL
            "#,
        )
        .bind(character_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn save_character_state(
        &self,
        id: &str,
        map_id: i32,
        pos_x: i32,
        pos_y: i32,
        hp: i32,
        max_hp: i32,
        mana: i32,
        max_mana: i32,
        gold: i32,
        level: i32,
        exp: i32,
        exp_next_level: i32,
        dead: bool,
        faction: &str,
        criminal: bool,
        min_hit: i32,
        max_hit: i32,
        attr_fuerza: i32,
        attr_agilidad: i32,
        attr_inteligencia: i32,
        attr_constitucion: i32,
        home_map: i32,
        home_x: i32,
        home_y: i32,
        id_head: i32,
        id_body: i32,
        id_helmet: i32,
        id_weapon: i32,
        id_shield: i32,
        id_arrow_slot: i32,
        id_ring_slot: i32,
        navegando: bool,
        bank_gold: i32,
        id_clase: i32,
        faction_rank: i32,
        faction_score: i32,
        faction_score_armada: i32,
        faction_score_caos: i32,
        criminales_matados: i32,
        ciudadanos_matados: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE characters
            SET map_id = ?2, pos_x = ?3, pos_y = ?4,
                hp = ?5, max_hp = ?6, mana = ?7, max_mana = ?8,
                gold = ?9, level = ?10, exp = ?11, exp_next_level = ?12,
                dead = ?13, faction = ?14, criminal = ?15,
                min_hit = ?16, max_hit = ?17,
                attr_fuerza = ?18, attr_agilidad = ?19,
                attr_inteligencia = ?20, attr_constitucion = ?21,
                home_map = ?22, home_x = ?23, home_y = ?24,
                id_head = ?25, id_body = ?26, id_helmet = ?27,
                id_weapon = ?28, id_shield = ?29,
                id_arrow_slot = ?30, id_ring_slot = ?31,
                navegando = ?32, bank_gold = ?33,
                id_clase = ?34, faction_rank = ?35, faction_score = ?36,
                faction_score_armada = ?37, faction_score_caos = ?38,
                criminales_matados = ?39, ciudadanos_matados = ?40,
                updated_at = datetime('now')
            WHERE id = ?1
            "#,
        )
        .bind(id)
        .bind(map_id)
        .bind(pos_x)
        .bind(pos_y)
        .bind(hp)
        .bind(max_hp)
        .bind(mana)
        .bind(max_mana)
        .bind(gold)
        .bind(level)
        .bind(exp)
        .bind(exp_next_level)
        .bind(dead)
        .bind(faction)
        .bind(criminal)
        .bind(min_hit)
        .bind(max_hit)
        .bind(attr_fuerza)
        .bind(attr_agilidad)
        .bind(attr_inteligencia)
        .bind(attr_constitucion)
        .bind(home_map)
        .bind(home_x)
        .bind(home_y)
        .bind(id_head)
        .bind(id_body)
        .bind(id_helmet)
        .bind(id_weapon)
        .bind(id_shield)
        .bind(id_arrow_slot)
        .bind(id_ring_slot)
        .bind(navegando)
        .bind(bank_gold)
        .bind(id_clase)
        .bind(faction_rank)
        .bind(faction_score)
        .bind(faction_score_armada)
        .bind(faction_score_caos)
        .bind(criminales_matados)
        .bind(ciudadanos_matados)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn begin_transaction(&self) -> Result<sqlx::Transaction<'_, sqlx::Sqlite>, sqlx::Error> {
        self.pool.begin().await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn save_character_state_in_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        id: &str,
        map_id: i32, pos_x: i32, pos_y: i32,
        hp: i32, max_hp: i32, mana: i32, max_mana: i32,
        gold: i32, level: i32, exp: i32, exp_next_level: i32,
        dead: bool, faction: &str, criminal: bool,
        min_hit: i32, max_hit: i32,
        attr_fuerza: i32, attr_agilidad: i32,
        attr_inteligencia: i32, attr_constitucion: i32,
        home_map: i32, home_x: i32, home_y: i32,
        id_head: i32, id_body: i32, id_helmet: i32,
        id_weapon: i32, id_shield: i32,
        id_arrow_slot: i32, id_ring_slot: i32,
        navegando: bool, bank_gold: i32,
        id_clase: i32, faction_rank: i32, faction_score: i32,
        faction_score_armada: i32, faction_score_caos: i32,
        criminales_matados: i32, ciudadanos_matados: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE characters
            SET map_id = ?2, pos_x = ?3, pos_y = ?4,
                hp = ?5, max_hp = ?6, mana = ?7, max_mana = ?8,
                gold = ?9, level = ?10, exp = ?11, exp_next_level = ?12,
                dead = ?13, faction = ?14, criminal = ?15,
                min_hit = ?16, max_hit = ?17,
                attr_fuerza = ?18, attr_agilidad = ?19,
                attr_inteligencia = ?20, attr_constitucion = ?21,
                home_map = ?22, home_x = ?23, home_y = ?24,
                id_head = ?25, id_body = ?26, id_helmet = ?27,
                id_weapon = ?28, id_shield = ?29,
                id_arrow_slot = ?30, id_ring_slot = ?31,
                navegando = ?32, bank_gold = ?33,
                id_clase = ?34, faction_rank = ?35, faction_score = ?36,
                faction_score_armada = ?37, faction_score_caos = ?38,
                criminales_matados = ?39, ciudadanos_matados = ?40,
                updated_at = datetime('now')
            WHERE id = ?1
            "#,
        )
        .bind(id).bind(map_id).bind(pos_x).bind(pos_y)
        .bind(hp).bind(max_hp).bind(mana).bind(max_mana)
        .bind(gold).bind(level).bind(exp).bind(exp_next_level)
        .bind(dead).bind(faction).bind(criminal)
        .bind(min_hit).bind(max_hit)
        .bind(attr_fuerza).bind(attr_agilidad)
        .bind(attr_inteligencia).bind(attr_constitucion)
        .bind(home_map).bind(home_x).bind(home_y)
        .bind(id_head).bind(id_body).bind(id_helmet)
        .bind(id_weapon).bind(id_shield)
        .bind(id_arrow_slot).bind(id_ring_slot)
        .bind(navegando).bind(bank_gold)
        .bind(id_clase).bind(faction_rank).bind(faction_score)
        .bind(faction_score_armada).bind(faction_score_caos)
        .bind(criminales_matados).bind(ciudadanos_matados)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    pub async fn create_character(&self, id: &str, account_id: &str, name: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO characters (id, account_id, name, id_clase, map_id, pos_x, pos_y,
                gold, hp, max_hp, mana, max_mana, level, dead, criminal, faction,
                min_hit, max_hit, attr_fuerza, attr_agilidad, attr_inteligencia, attr_constitucion,
                id_head, id_body, id_helmet, id_weapon, id_shield, navegando,
                home_map, home_x, home_y, created_at, updated_at)
            VALUES (?1, ?2, ?3, 1, 1, 50, 50,
                100, 100, 100, 80, 80, 1, false, false, 'none',
                1, 5, 10, 10, 10, 10,
                1, 1, 0, 0, 0, false,
                1, 50, 50, datetime('now'), datetime('now'))"#,
        )
        .bind(id)
        .bind(account_id)
        .bind(name)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn create_character_with_class(&self, id: &str, account_id: &str, name: &str, id_clase: i32) -> Result<(), sqlx::Error> {
        let (hp, mana, fue, agi, int, con) = match id_clase {
            1 => (80, 150, 8, 10, 18, 8),
            2 => (100, 120, 10, 10, 15, 12),
            3 => (120, 60, 18, 12, 8, 15),
            4 => (90, 80, 12, 18, 10, 10),
            5 => (90, 100, 10, 14, 14, 10),
            6 => (95, 110, 10, 12, 16, 10),
            7 => (110, 80, 16, 10, 12, 14),
            8 => (95, 70, 12, 16, 10, 12),
            _ => (100, 80, 12, 12, 12, 12),
        };

        sqlx::query(
            r#"INSERT INTO characters (id, account_id, name, id_clase, map_id, pos_x, pos_y,
                gold, hp, max_hp, mana, max_mana, level, dead, criminal, faction,
                min_hit, max_hit, attr_fuerza, attr_agilidad, attr_inteligencia, attr_constitucion,
                id_head, id_body, id_helmet, id_weapon, id_shield, navegando,
                home_map, home_x, home_y, exp, exp_next_level, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, 1, 50, 50,
                100, ?5, ?5, ?6, ?6, 1, false, false, 'none',
                1, 5, ?7, ?8, ?9, ?10,
                1, 1, 0, 0, 0, false,
                1, 50, 50, 0, 300, datetime('now'), datetime('now'))"#,
        )
        .bind(id)
        .bind(account_id)
        .bind(name)
        .bind(id_clase)
        .bind(hp)
        .bind(mana)
        .bind(fue)
        .bind(agi)
        .bind(int)
        .bind(con)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn find_character_by_account(&self, account_id: &str) -> Result<Option<CharacterData>, sqlx::Error> {
        let row = sqlx::query_as::<_, CharacterData>(
            "SELECT * FROM characters WHERE account_id = ?1 LIMIT 1"
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn get_ranking(&self) -> Result<Vec<crate::api::RankEntry>, sqlx::Error> {
        let rows = sqlx::query_as::<_, crate::api::RankEntry>(
            "SELECT name, level, gold FROM characters ORDER BY level DESC, gold DESC LIMIT 50"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_character_settings(&self, character_id: &str) -> Result<Option<String>, sqlx::Error> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT settings_json FROM character_settings WHERE character_id = ?"
        )
        .bind(character_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0))
    }

    pub async fn save_character_settings(&self, character_id: &str, json: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO character_settings (character_id, settings_json) VALUES (?, ?)
             ON CONFLICT(character_id) DO UPDATE SET settings_json = excluded.settings_json"
        )
        .bind(character_id)
        .bind(json)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_characters_by_account(&self, account_id: &str) -> Result<Vec<CharacterSummary>, sqlx::Error> {
        let rows = sqlx::query_as::<_, CharacterSummary>(
            "SELECT id, name, level, id_clase, map_id FROM characters WHERE account_id = ?"
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn delete_character(&self, character_id: &str, account_id: &str) -> Result<bool, sqlx::Error> {
        let res = sqlx::query(
            "DELETE FROM characters WHERE id = ? AND account_id = ?"
        )
        .bind(character_id)
        .bind(account_id)
        .execute(&self.pool)
        .await?;
        Ok(res.rows_affected() > 0)
    }
}
