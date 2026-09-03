use crate::persistence::Database;
use crate::gameplay::pets::{PetManager, Pet};

impl Database {
    pub async fn load_pets(&self, character_id: &str) -> Result<PetManager, sqlx::Error> {
        let rows = sqlx::query_as::<_, (i32, String, i32, i32, i32, i32)>(
            "SELECT pet_type, pet_name, level, exp, hp, active FROM character_pets WHERE character_id = ? ORDER BY id",
        )
        .bind(character_id)
        .fetch_all(&self.pool)
        .await?;

        let mut mgr = PetManager::new();
        for (pet_type, name, level, exp, hp, active) in rows {
            let max_hp = 50 + (level - 1) * 10;
            mgr.pets.push(Pet {
                pet_type,
                name,
                level,
                exp,
                hp,
                max_hp,
                active: active != 0,
            });
        }
        Ok(mgr)
    }

    pub async fn save_pets(&self, character_id: &str, pets: &PetManager) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM character_pets WHERE character_id = ?")
            .bind(character_id)
            .execute(&self.pool)
            .await?;

        for pet in &pets.pets {
            sqlx::query(
                "INSERT INTO character_pets (character_id, pet_type, pet_name, level, exp, hp, active) VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(character_id)
            .bind(pet.pet_type)
            .bind(&pet.name)
            .bind(pet.level)
            .bind(pet.exp)
            .bind(pet.hp)
            .bind(if pet.active { 1 } else { 0 })
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }
}
