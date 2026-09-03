use super::{MarketClaimRow, MarketListingRow, Database};

impl Database {
    pub async fn create_market_listing(
        &self,
        seller_char_id: &str,
        seller_name: &str,
        item_id: i32,
        quantity: i32,
        unit_price: i32,
        duration_hours: i32,
    ) -> Result<String, sqlx::Error> {
        let id = uuid::Uuid::new_v4().to_string();
        let expires_at = chrono::Utc::now()
            + chrono::Duration::hours(duration_hours as i64);
        let expires_str = expires_at.to_rfc3339();

        sqlx::query(
            "INSERT INTO market_listings (id, seller_char_id, seller_name, item_id, quantity, unit_price, status, expires_at) VALUES (?1,?2,?3,?4,?5,?6,'active',?7)"
        )
        .bind(&id)
        .bind(seller_char_id)
        .bind(seller_name)
        .bind(item_id)
        .bind(quantity)
        .bind(unit_price)
        .bind(&expires_str)
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    pub async fn get_active_listings(&self) -> Result<Vec<MarketListingRow>, sqlx::Error> {
        let rows = sqlx::query_as::<_, MarketListingRow>(
            "SELECT * FROM market_listings WHERE status = 'active' AND expires_at > datetime('now') ORDER BY created_at DESC LIMIT 200"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_my_listings(&self, char_id: &str) -> Result<Vec<MarketListingRow>, sqlx::Error> {
        let rows = sqlx::query_as::<_, MarketListingRow>(
            "SELECT * FROM market_listings WHERE seller_char_id = ?1 ORDER BY created_at DESC LIMIT 50"
        )
        .bind(char_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn buy_market_listing(
        &self,
        listing_id: &str,
        buyer_char_id: &str,
    ) -> Result<Option<MarketListingRow>, sqlx::Error> {
        let listing = sqlx::query_as::<_, MarketListingRow>(
            "SELECT * FROM market_listings WHERE id = ?1 AND status = 'active'"
        )
        .bind(listing_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(listing) = listing else { return Ok(None); };

        if listing.seller_char_id == buyer_char_id {
            return Ok(None);
        }

        sqlx::query("UPDATE market_listings SET status = 'sold' WHERE id = ?1 AND status = 'active'")
            .bind(listing_id)
            .execute(&self.pool)
            .await?;

        let gold_claim_id = uuid::Uuid::new_v4().to_string();
        let total_gold = listing.unit_price * listing.quantity;
        sqlx::query(
            "INSERT INTO market_claims (id, char_id, claim_type, gold_amount, created_at) VALUES (?1,?2,'gold',?3,datetime('now'))"
        )
        .bind(&gold_claim_id)
        .bind(&listing.seller_char_id)
        .bind(total_gold)
        .execute(&self.pool)
        .await?;

        let item_claim_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO market_claims (id, char_id, claim_type, item_id, item_quantity, created_at) VALUES (?1,?2,'item',?3,?4,datetime('now'))"
        )
        .bind(&item_claim_id)
        .bind(buyer_char_id)
        .bind(listing.item_id)
        .bind(listing.quantity)
        .execute(&self.pool)
        .await?;

        Ok(Some(listing))
    }

    pub async fn cancel_market_listing(
        &self,
        listing_id: &str,
        char_id: &str,
    ) -> Result<Option<MarketListingRow>, sqlx::Error> {
        let listing = sqlx::query_as::<_, MarketListingRow>(
            "SELECT * FROM market_listings WHERE id = ?1 AND seller_char_id = ?2 AND status = 'active'"
        )
        .bind(listing_id)
        .bind(char_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(listing) = listing else { return Ok(None); };

        sqlx::query("UPDATE market_listings SET status = 'cancelled' WHERE id = ?1")
            .bind(listing_id)
            .execute(&self.pool)
            .await?;

        let claim_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO market_claims (id, char_id, claim_type, item_id, item_quantity, created_at) VALUES (?1,?2,'item',?3,?4,datetime('now'))"
        )
        .bind(&claim_id)
        .bind(char_id)
        .bind(listing.item_id)
        .bind(listing.quantity)
        .execute(&self.pool)
        .await?;

        Ok(Some(listing))
    }

    pub async fn get_claims(&self, char_id: &str) -> Result<Vec<MarketClaimRow>, sqlx::Error> {
        let rows = sqlx::query_as::<_, MarketClaimRow>(
            "SELECT * FROM market_claims WHERE char_id = ?1 ORDER BY created_at DESC"
        )
        .bind(char_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn delete_claim(&self, claim_id: &str, char_id: &str) -> Result<Option<MarketClaimRow>, sqlx::Error> {
        let claim = sqlx::query_as::<_, MarketClaimRow>(
            "SELECT * FROM market_claims WHERE id = ?1 AND char_id = ?2"
        )
        .bind(claim_id)
        .bind(char_id)
        .fetch_optional(&self.pool)
        .await?;

        if claim.is_some() {
            sqlx::query("DELETE FROM market_claims WHERE id = ?1")
                .bind(claim_id)
                .execute(&self.pool)
                .await?;
        }

        Ok(claim)
    }

    pub async fn expire_market_listings(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE market_listings SET status = 'expired' WHERE status = 'active' AND expires_at <= datetime('now')"
        )
        .execute(&self.pool)
        .await?;

        let expired_count = result.rows_affected();
        if expired_count > 0 {
            let rows = sqlx::query_as::<_, MarketListingRow>(
                "SELECT * FROM market_listings WHERE status = 'expired' AND id NOT IN (SELECT DISTINCT substr(id,1,36) FROM market_claims WHERE claim_type='item')"
            )
            .fetch_all(&self.pool)
            .await?;

            for row in &rows {
                let claim_id = uuid::Uuid::new_v4().to_string();
                sqlx::query(
                    "INSERT INTO market_claims (id, char_id, claim_type, item_id, item_quantity, created_at) VALUES (?1,?2,'item',?3,?4,datetime('now'))"
                )
                .bind(&claim_id)
                .bind(&row.seller_char_id)
                .bind(row.item_id)
                .bind(row.quantity)
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(expired_count)
    }
}
