use super::{BankRow, Database};

impl Database {
    pub async fn load_bank(&self, character_id: &str) -> Result<Vec<BankRow>, sqlx::Error> {
        let rows = sqlx::query_as::<_, BankRow>(
            "SELECT slot, item_id, amount FROM character_bank WHERE character_id = ?1 ORDER BY slot",
        )
        .bind(character_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn update_bank_slot(&self, character_id: &str, slot: i32, item_id: i32, amount: i32) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO character_bank (character_id, slot, item_id, amount) VALUES (?1, ?2, ?3, ?4)",
        )
        .bind(character_id)
        .bind(slot)
        .bind(item_id)
        .bind(amount)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_bank_slot(&self, character_id: &str, slot: i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM character_bank WHERE character_id = ?1 AND slot = ?2")
            .bind(character_id)
            .bind(slot)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_bank_gold(&self, character_id: &str) -> Result<i32, sqlx::Error> {
        let result = sqlx::query_scalar::<_, i32>(
            "SELECT COALESCE(bank_gold, 0) FROM characters WHERE id = ?1"
        )
        .bind(character_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(result.unwrap_or(0))
    }

    pub async fn set_bank_gold(&self, character_id: &str, gold: i32) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE characters SET bank_gold = ?2 WHERE id = ?1")
            .bind(character_id)
            .bind(gold)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn load_account_vault(&self, account_id: &str) -> Result<Vec<BankRow>, sqlx::Error> {
        let rows = sqlx::query_as::<_, BankRow>(
            "SELECT slot, item_id, amount FROM account_vault WHERE account_id = ?1 ORDER BY slot",
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    #[allow(dead_code)]
    pub async fn update_account_vault_slot(&self, account_id: &str, slot: i32, item_id: i32, amount: i32) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO account_vault (account_id, slot, item_id, amount) VALUES (?1, ?2, ?3, ?4)",
        )
        .bind(account_id)
        .bind(slot)
        .bind(item_id)
        .bind(amount)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn delete_account_vault_slot(&self, account_id: &str, slot: i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM account_vault WHERE account_id = ?1 AND slot = ?2")
            .bind(account_id)
            .bind(slot)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn get_account_vault_gold(&self, account_id: &str) -> Result<i32, sqlx::Error> {
        let result = sqlx::query_scalar::<_, i32>(
            "SELECT gold FROM account_vault_gold WHERE account_id = ?1"
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(result.unwrap_or(0))
    }

    #[allow(dead_code)]
    pub async fn set_account_vault_gold(&self, account_id: &str, gold: i32) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO account_vault_gold (account_id, gold) VALUES (?1, ?2)"
        )
        .bind(account_id)
        .bind(gold)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn load_clan_vault(&self, clan_id: &str) -> Result<Vec<BankRow>, sqlx::Error> {
        let rows = sqlx::query_as::<_, BankRow>(
            "SELECT slot, item_id, amount FROM clan_vault WHERE clan_id = ?1 ORDER BY slot",
        )
        .bind(clan_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    #[allow(dead_code)]
    pub async fn update_clan_vault_slot(&self, clan_id: &str, slot: i32, item_id: i32, amount: i32) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO clan_vault (clan_id, slot, item_id, amount) VALUES (?1, ?2, ?3, ?4)",
        )
        .bind(clan_id)
        .bind(slot)
        .bind(item_id)
        .bind(amount)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn delete_clan_vault_slot(&self, clan_id: &str, slot: i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM clan_vault WHERE clan_id = ?1 AND slot = ?2")
            .bind(clan_id)
            .bind(slot)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn get_clan_vault_gold(&self, clan_id: &str) -> Result<i32, sqlx::Error> {
        let result = sqlx::query_scalar::<_, i32>(
            "SELECT gold FROM clan_vault_gold WHERE clan_id = ?1"
        )
        .bind(clan_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(result.unwrap_or(0))
    }

    #[allow(dead_code)]
    pub async fn set_clan_vault_gold(&self, clan_id: &str, gold: i32) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO clan_vault_gold (clan_id, gold) VALUES (?1, ?2)"
        )
        .bind(clan_id)
        .bind(gold)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
