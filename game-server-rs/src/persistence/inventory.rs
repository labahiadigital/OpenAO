use super::{InventoryRow, Database};

impl Database {
    pub async fn load_inventory(
        &self,
        character_id: &str,
    ) -> Result<Vec<InventoryRow>, sqlx::Error> {
        let rows = sqlx::query_as::<_, InventoryRow>(
            "SELECT slot, item_id, amount, equipped FROM character_inventory WHERE character_id = ?1 ORDER BY slot",
        )
        .bind(character_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    #[allow(dead_code)]
    pub async fn update_inventory_slot(
        &self,
        character_id: &str,
        slot: i32,
        item_id: i32,
        amount: i32,
        equipped: bool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO character_inventory (character_id, slot, item_id, amount, equipped) VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .bind(character_id)
        .bind(slot)
        .bind(item_id)
        .bind(amount)
        .bind(equipped)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn delete_inventory_slot(
        &self,
        character_id: &str,
        slot: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM character_inventory WHERE character_id = ?1 AND slot = ?2")
            .bind(character_id)
            .bind(slot)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn get_inventory(&self, character_id: &str) -> Result<Vec<InventoryRow>, sqlx::Error> {
        self.load_inventory(character_id).await
    }

    pub async fn save_full_inventory(
        &self,
        character_id: &str,
        rows: &[InventoryRow],
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM character_inventory WHERE character_id = ?1")
            .bind(character_id)
            .execute(&self.pool)
            .await?;
        for row in rows {
            sqlx::query(
                "INSERT INTO character_inventory (character_id, slot, item_id, amount, equipped) VALUES (?1, ?2, ?3, ?4, ?5)"
            )
            .bind(character_id)
            .bind(row.slot)
            .bind(row.item_id)
            .bind(row.amount)
            .bind(row.equipped)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn remove_items_from_inventory(&self, character_id: &str, item_id: i32, mut amount_to_remove: i32) -> Result<(), sqlx::Error> {
        let inv = self.load_inventory(character_id).await?;
        for row in inv {
            if row.item_id == item_id && amount_to_remove > 0 {
                let remove = amount_to_remove.min(row.amount);
                let remaining = row.amount - remove;
                if remaining <= 0 {
                    self.delete_inventory_slot(character_id, row.slot).await?;
                } else {
                    self.update_inventory_slot(character_id, row.slot, row.item_id, remaining, row.equipped).await?;
                }
                amount_to_remove -= remove;
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn add_item_to_inventory(&self, character_id: &str, item_id: i32, amount: i32) -> Result<(), sqlx::Error> {
        let inv = self.load_inventory(character_id).await?;

        for row in &inv {
            if row.item_id == item_id {
                let new_amount = row.amount + amount;
                return self.update_inventory_slot(character_id, row.slot, item_id, new_amount, row.equipped).await;
            }
        }

        let used_slots: Vec<i32> = inv.iter().map(|r| r.slot).collect();
        for slot in 0..20 {
            if !used_slots.contains(&slot) {
                return self.update_inventory_slot(character_id, slot, item_id, amount, false).await;
            }
        }

        Ok(())
    }
}
