use crate::persistence::Database;
use crate::gameplay::achievements::AchievementTracker;

impl Database {
    pub async fn load_achievements(&self, character_id: &str) -> Result<AchievementTracker, sqlx::Error> {
        let rows = sqlx::query_scalar::<_, i64>(
            "SELECT achievement_id FROM character_achievements WHERE character_id = ?",
        )
        .bind(character_id)
        .fetch_all(&self.pool)
        .await?;

        let mut tracker = AchievementTracker::new();
        for id in rows {
            tracker.unlocked.insert(id as u32);
        }
        Ok(tracker)
    }

    pub async fn save_achievements(&self, character_id: &str, tracker: &AchievementTracker) -> Result<(), sqlx::Error> {
        for &aid in &tracker.unlocked {
            sqlx::query(
                "INSERT OR IGNORE INTO character_achievements (character_id, achievement_id) VALUES (?, ?)",
            )
            .bind(character_id)
            .bind(aid as i64)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }
}
