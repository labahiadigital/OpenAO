use super::Database;

impl Database {
    pub async fn add_ban(&self, account_id: &str, reason: &str, banned_by: &str) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT OR REPLACE INTO bans (account_id, reason, banned_by) VALUES (?1, ?2, ?3)")
            .bind(account_id).bind(reason).bind(banned_by)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn remove_ban(&self, account_id: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM bans WHERE account_id = ?1")
            .bind(account_id).execute(&self.pool).await?;
        Ok(result.rows_affected() > 0)
    }

    #[allow(dead_code)]
    pub async fn is_banned(&self, account_id: &str) -> Result<bool, sqlx::Error> {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM bans WHERE account_id = ?1")
            .bind(account_id).fetch_one(&self.pool).await?;
        Ok(count > 0)
    }

    pub async fn load_all_bans(&self) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query_scalar::<_, String>("SELECT account_id FROM bans")
            .fetch_all(&self.pool).await?;
        Ok(rows)
    }

    pub async fn add_mute(&self, account_id: &str, reason: &str, muted_by: &str) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT OR REPLACE INTO mutes (account_id, reason, muted_by) VALUES (?1, ?2, ?3)")
            .bind(account_id).bind(reason).bind(muted_by)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn remove_mute(&self, account_id: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM mutes WHERE account_id = ?1")
            .bind(account_id).execute(&self.pool).await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn is_muted(&self, account_id: &str) -> Result<bool, sqlx::Error> {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM mutes WHERE account_id = ?1")
            .bind(account_id).fetch_one(&self.pool).await?;
        Ok(count > 0)
    }

    #[allow(dead_code)]
    pub async fn load_all_mutes(&self) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query_scalar::<_, String>("SELECT account_id FROM mutes")
            .fetch_all(&self.pool).await?;
        Ok(rows)
    }

    pub async fn add_ip_ban(&self, ip: &str, reason: &str, banned_by: &str) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT OR REPLACE INTO ip_bans (ip_address, reason, banned_by) VALUES (?1, ?2, ?3)")
            .bind(ip).bind(reason).bind(banned_by)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn remove_ip_ban(&self, ip: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM ip_bans WHERE ip_address = ?1")
            .bind(ip).execute(&self.pool).await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn load_all_ip_bans(&self) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query_scalar::<_, String>("SELECT ip_address FROM ip_bans")
            .fetch_all(&self.pool).await?;
        Ok(rows)
    }
}
