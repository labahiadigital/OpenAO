use crate::persistence::Database;
use crate::gameplay::quests::{PlayerQuestLog, ActiveQuest, ObjectiveProgress, QuestId};

impl Database {
    pub async fn load_quest_log(&self, character_id: &str) -> Result<PlayerQuestLog, sqlx::Error> {
        let active_rows = sqlx::query_as::<_, (i64, String)>(
            "SELECT quest_id, objectives_json FROM character_quests_active WHERE character_id = ?",
        )
        .bind(character_id)
        .fetch_all(&self.pool)
        .await?;

        let completed_rows = sqlx::query_scalar::<_, i64>(
            "SELECT quest_id FROM character_quests_completed WHERE character_id = ?",
        )
        .bind(character_id)
        .fetch_all(&self.pool)
        .await?;

        let mut log = PlayerQuestLog::new();
        for (qid, obj_json) in active_rows {
            if let Ok(objectives) = serde_json::from_str::<Vec<ObjectiveProgressRow>>(&obj_json) {
                log.active.push(ActiveQuest {
                    quest_id: qid as QuestId,
                    objectives: objectives.into_iter().map(|o| ObjectiveProgress {
                        current: o.current,
                        required: o.required,
                        completed: o.completed,
                    }).collect(),
                });
            }
        }
        log.completed = completed_rows.into_iter().map(|id| id as QuestId).collect();
        Ok(log)
    }

    pub async fn save_quest_log(&self, character_id: &str, log: &PlayerQuestLog) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM character_quests_active WHERE character_id = ?")
            .bind(character_id)
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM character_quests_completed WHERE character_id = ?")
            .bind(character_id)
            .execute(&self.pool)
            .await?;

        for aq in &log.active {
            let objectives: Vec<ObjectiveProgressRow> = aq.objectives.iter().map(|o| ObjectiveProgressRow {
                current: o.current,
                required: o.required,
                completed: o.completed,
            }).collect();
            let obj_json = serde_json::to_string(&objectives).unwrap_or_default();
            sqlx::query(
                "INSERT INTO character_quests_active (character_id, quest_id, objectives_json) VALUES (?, ?, ?)",
            )
            .bind(character_id)
            .bind(aq.quest_id as i64)
            .bind(&obj_json)
            .execute(&self.pool)
            .await?;
        }

        for qid in &log.completed {
            sqlx::query(
                "INSERT OR IGNORE INTO character_quests_completed (character_id, quest_id) VALUES (?, ?)",
            )
            .bind(character_id)
            .bind(*qid as i64)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ObjectiveProgressRow {
    current: u32,
    required: u32,
    completed: bool,
}
