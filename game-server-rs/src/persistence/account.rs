use sqlx::Row;

use super::{AccountRow, Database};

impl Database {
    pub async fn find_account_by_email(&self, email: &str) -> Result<Option<AccountRow>, sqlx::Error> {
        let row = sqlx::query_as::<_, AccountRow>(
            "SELECT id, email, password_hash FROM accounts WHERE email = ?1"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn create_account(&self, id: &str, email: &str, password: &str) -> Result<(), sqlx::Error> {
        let username = email.split('@').next().unwrap_or(id);
        sqlx::query(
            "INSERT INTO accounts (id, username, email, password_hash, created_at) VALUES (?1, ?2, ?3, ?4, datetime('now'))"
        )
        .bind(id)
        .bind(username)
        .bind(email)
        .bind(password)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_password_hash(&self, account_id: &str, new_hash: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE accounts SET password_hash = ?2 WHERE id = ?1")
            .bind(account_id)
            .bind(new_hash)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn consume_game_ticket(
        &self,
        ticket: &str,
    ) -> Result<Option<(String, String)>, sqlx::Error> {
        let dev_mode = std::env::var("OPENAO_DEV_TICKETS")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        if dev_mode {
            sqlx::query("UPDATE game_tickets SET consumed_at = NULL WHERE ticket = ?1")
                .bind(ticket)
                .execute(&self.pool)
                .await?;
        }

        let row = sqlx::query(
            r#"
            UPDATE game_tickets
            SET consumed_at = datetime('now')
            WHERE ticket = ?1
              AND consumed_at IS NULL
              AND expires_at > datetime('now')
            RETURNING account_id, character_id
            "#,
        )
        .bind(ticket)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| {
            let account_id: String = r.get("account_id");
            let character_id: String = r.get("character_id");
            (account_id, character_id)
        }))
    }

    pub async fn create_ticket_for_login(&self, account_id: &str, character_id: &str, ticket: &str) -> Result<(), sqlx::Error> {
        let expires = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::hours(24))
            .unwrap()
            .to_rfc3339();

        sqlx::query(
            "INSERT OR REPLACE INTO game_tickets (ticket, account_id, character_id, expires_at) VALUES (?1, ?2, ?3, ?4)"
        )
        .bind(ticket)
        .bind(account_id)
        .bind(character_id)
        .bind(&expires)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
