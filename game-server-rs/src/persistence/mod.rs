mod account;
mod bank;
mod character;
mod inventory;
mod market;
mod moderation;
mod quests;
mod pets;
mod achievements;
#[cfg(test)]
mod persistence_tests;

use sqlx::sqlite::{SqlitePoolOptions, SqliteConnectOptions};
use sqlx::SqlitePool;
use uuid::Uuid;
use tracing::info;
use std::str::FromStr;
use std::time::Duration;
use argon2::{Argon2, PasswordHasher};

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn connect(path: &str) -> Result<Self, sqlx::Error> {
        let options = SqliteConnectOptions::from_str(path)?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .busy_timeout(Duration::from_secs(5))
            .pragma("cache_size", "-8000")
            .pragma("synchronous", "NORMAL")
            .pragma("temp_store", "MEMORY")
            .pragma("mmap_size", "268435456")
            .statement_cache_capacity(256);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        info!("SQLite database connected: {path} (WAL mode, 256 stmt cache, 8MB page cache, 256MB mmap)");
        Ok(Self { pool })
    }

    #[allow(dead_code)]
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn run_migrations(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS accounts (
                id TEXT PRIMARY KEY,
                username TEXT NOT NULL UNIQUE,
                email TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                is_admin INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS characters (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL REFERENCES accounts(id),
                name TEXT NOT NULL UNIQUE,
                id_clase INTEGER NOT NULL DEFAULT 1,
                map_id INTEGER NOT NULL DEFAULT 1,
                pos_x INTEGER NOT NULL DEFAULT 50,
                pos_y INTEGER NOT NULL DEFAULT 50,
                gold INTEGER NOT NULL DEFAULT 0,
                hp INTEGER NOT NULL DEFAULT 100,
                max_hp INTEGER NOT NULL DEFAULT 100,
                mana INTEGER NOT NULL DEFAULT 100,
                max_mana INTEGER NOT NULL DEFAULT 100,
                level INTEGER NOT NULL DEFAULT 1,
                dead INTEGER NOT NULL DEFAULT 0,
                criminal INTEGER NOT NULL DEFAULT 0,
                faction TEXT NOT NULL DEFAULT '',
                min_hit INTEGER NOT NULL DEFAULT 1,
                max_hit INTEGER NOT NULL DEFAULT 5,
                attr_fuerza INTEGER NOT NULL DEFAULT 15,
                attr_agilidad INTEGER NOT NULL DEFAULT 15,
                attr_inteligencia INTEGER NOT NULL DEFAULT 15,
                attr_constitucion INTEGER NOT NULL DEFAULT 15,
                id_head INTEGER NOT NULL DEFAULT 1,
                id_body INTEGER NOT NULL DEFAULT 1,
                id_helmet INTEGER NOT NULL DEFAULT 0,
                id_weapon INTEGER NOT NULL DEFAULT 0,
                id_shield INTEGER NOT NULL DEFAULT 0,
                navegando INTEGER NOT NULL DEFAULT 0,
                home_map INTEGER NOT NULL DEFAULT 1,
                home_x INTEGER NOT NULL DEFAULT 50,
                home_y INTEGER NOT NULL DEFAULT 50,
                exp INTEGER NOT NULL DEFAULT 0,
                exp_next_level INTEGER NOT NULL DEFAULT 300,
                bank_gold INTEGER NOT NULL DEFAULT 0,
                deleted_at TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS game_tickets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ticket TEXT NOT NULL UNIQUE,
                account_id TEXT NOT NULL REFERENCES accounts(id),
                character_id TEXT NOT NULL REFERENCES characters(id),
                consumed_at TEXT,
                expires_at TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS character_inventory (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                character_id TEXT NOT NULL REFERENCES characters(id),
                slot INTEGER NOT NULL,
                item_id INTEGER NOT NULL,
                amount INTEGER NOT NULL DEFAULT 1,
                equipped INTEGER NOT NULL DEFAULT 0,
                UNIQUE(character_id, slot)
            );

            CREATE TABLE IF NOT EXISTS character_bank (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                character_id TEXT NOT NULL REFERENCES characters(id),
                slot INTEGER NOT NULL,
                item_id INTEGER NOT NULL,
                amount INTEGER NOT NULL DEFAULT 1,
                UNIQUE(character_id, slot)
            );

            CREATE TABLE IF NOT EXISTS market_listings (
                id TEXT PRIMARY KEY,
                seller_char_id TEXT NOT NULL REFERENCES characters(id),
                seller_name TEXT NOT NULL,
                item_id INTEGER NOT NULL,
                quantity INTEGER NOT NULL,
                unit_price INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'active',
                expires_at TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS bans (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id TEXT NOT NULL UNIQUE,
                reason TEXT NOT NULL DEFAULT '',
                banned_by TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS mutes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id TEXT NOT NULL UNIQUE,
                reason TEXT NOT NULL DEFAULT '',
                muted_by TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS ip_bans (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ip_address TEXT NOT NULL UNIQUE,
                reason TEXT NOT NULL DEFAULT '',
                banned_by TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS character_quests_active (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                character_id TEXT NOT NULL REFERENCES characters(id),
                quest_id INTEGER NOT NULL,
                objectives_json TEXT NOT NULL DEFAULT '[]',
                UNIQUE(character_id, quest_id)
            );

            CREATE TABLE IF NOT EXISTS character_quests_completed (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                character_id TEXT NOT NULL REFERENCES characters(id),
                quest_id INTEGER NOT NULL,
                completed_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(character_id, quest_id)
            );

            CREATE TABLE IF NOT EXISTS character_achievements (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                character_id TEXT NOT NULL REFERENCES characters(id),
                achievement_id INTEGER NOT NULL,
                unlocked_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(character_id, achievement_id)
            );

            CREATE TABLE IF NOT EXISTS character_pets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                character_id TEXT NOT NULL REFERENCES characters(id),
                pet_type INTEGER NOT NULL,
                pet_name TEXT NOT NULL DEFAULT '',
                level INTEGER NOT NULL DEFAULT 1,
                exp INTEGER NOT NULL DEFAULT 0,
                hp INTEGER NOT NULL DEFAULT 100,
                active INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS market_claims (
                id TEXT PRIMARY KEY,
                char_id TEXT NOT NULL REFERENCES characters(id),
                claim_type TEXT NOT NULL,
                gold_amount INTEGER NOT NULL DEFAULT 0,
                item_id INTEGER,
                item_quantity INTEGER,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS character_settings (
                character_id TEXT PRIMARY KEY REFERENCES characters(id),
                settings_json TEXT NOT NULL DEFAULT '{}'
            );

            CREATE TABLE IF NOT EXISTS account_vault (
                account_id TEXT NOT NULL REFERENCES accounts(id),
                slot INTEGER NOT NULL,
                item_id INTEGER NOT NULL,
                amount INTEGER NOT NULL DEFAULT 1,
                UNIQUE(account_id, slot)
            );

            CREATE TABLE IF NOT EXISTS clan_vault (
                clan_id TEXT NOT NULL,
                slot INTEGER NOT NULL,
                item_id INTEGER NOT NULL,
                amount INTEGER NOT NULL DEFAULT 1,
                UNIQUE(clan_id, slot)
            );

            CREATE TABLE IF NOT EXISTS account_vault_gold (
                account_id TEXT PRIMARY KEY REFERENCES accounts(id),
                gold INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS clan_vault_gold (
                clan_id TEXT PRIMARY KEY,
                gold INTEGER NOT NULL DEFAULT 0
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        let alter_statements = [
            "ALTER TABLE characters ADD COLUMN exp INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE characters ADD COLUMN exp_next_level INTEGER NOT NULL DEFAULT 300",
            "ALTER TABLE characters ADD COLUMN bank_gold INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE characters ADD COLUMN faction_rank INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE characters ADD COLUMN faction_score INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE characters ADD COLUMN id_raza INTEGER NOT NULL DEFAULT 1",
            "ALTER TABLE characters ADD COLUMN faction_score_armada INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE characters ADD COLUMN faction_score_caos INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE characters ADD COLUMN criminales_matados INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE characters ADD COLUMN ciudadanos_matados INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE characters ADD COLUMN id_arrow_slot INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE characters ADD COLUMN id_ring_slot INTEGER NOT NULL DEFAULT 0",
        ];
        for stmt in &alter_statements {
            if let Err(e) = sqlx::query(stmt).execute(&self.pool).await {
                let msg = e.to_string();
                if !msg.contains("duplicate column") {
                    tracing::warn!("ALTER migration skipped: {msg}");
                }
            }
        }

        info!("SQLite migrations applied");
        Ok(())
    }

    pub async fn seed_test_data(&self) -> Result<(), sqlx::Error> {
        let account_id = Uuid::new_v4().to_string();
        let char_id = Uuid::new_v4().to_string();
        let ticket = "dev-ticket-local";

        let has_data = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM accounts")
            .fetch_one(&self.pool)
            .await?;

        if has_data > 0 {
            info!("Database already has data, skipping seed");
            return Ok(());
        }

        let admin_hash = Argon2::default()
            .hash_password(b"admin")
            .map(|h| h.to_string())
            .unwrap_or_else(|_| "admin".to_string());

        sqlx::query(
            "INSERT OR IGNORE INTO accounts (id, username, email, password_hash, is_admin) VALUES (?1, ?2, ?3, ?4, 1)",
        )
        .bind(&account_id)
        .bind("admin")
        .bind("admin@openao.local")
        .bind(&admin_hash)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "INSERT OR IGNORE INTO characters (id, account_id, name, id_clase, map_id, pos_x, pos_y, hp, max_hp, mana, max_mana, level, id_head, id_body) VALUES (?1, ?2, ?3, 1, 1, 50, 50, 250, 250, 500, 500, 1, 2, 1)",
        )
        .bind(&char_id)
        .bind(&account_id)
        .bind("Tester")
        .execute(&self.pool)
        .await?;

        let expires = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::hours(24))
            .unwrap()
            .to_rfc3339();

        sqlx::query(
            "INSERT OR IGNORE INTO game_tickets (ticket, account_id, character_id, expires_at) VALUES (?1, ?2, ?3, ?4)",
        )
        .bind(ticket)
        .bind(&account_id)
        .bind(&char_id)
        .bind(&expires)
        .execute(&self.pool)
        .await?;

        sqlx::query("INSERT OR IGNORE INTO character_inventory (character_id, slot, item_id, amount, equipped) VALUES (?1, 0, 2, 1, 1)")
            .bind(&char_id).execute(&self.pool).await?;
        sqlx::query("INSERT OR IGNORE INTO character_inventory (character_id, slot, item_id, amount, equipped) VALUES (?1, 1, 30, 1, 1)")
            .bind(&char_id).execute(&self.pool).await?;
        sqlx::query("INSERT OR IGNORE INTO character_inventory (character_id, slot, item_id, amount, equipped) VALUES (?1, 2, 1, 10, 0)")
            .bind(&char_id).execute(&self.pool).await?;
        sqlx::query("INSERT OR IGNORE INTO character_inventory (character_id, slot, item_id, amount, equipped) VALUES (?1, 3, 26, 5, 0)")
            .bind(&char_id).execute(&self.pool).await?;

        info!("Seeded test account (admin/admin@openao.local), character 'Tester', ticket '{ticket}'");
        Ok(())
    }
}

// --- Data types ---

#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
pub struct AccountRow {
    pub id: String,
    pub email: String,
    pub password_hash: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct InventoryRow {
    pub slot: i32,
    pub item_id: i32,
    pub amount: i32,
    pub equipped: bool,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BankRow {
    pub slot: i32,
    pub item_id: i32,
    pub amount: i32,
}

#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
pub struct CharacterData {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub id_clase: i32,
    #[sqlx(default)]
    pub id_raza: i32,
    pub map_id: i32,
    pub pos_x: i32,
    pub pos_y: i32,
    pub gold: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub mana: i32,
    pub max_mana: i32,
    pub level: i32,
    pub dead: bool,
    pub criminal: bool,
    pub faction: String,
    pub min_hit: i32,
    pub max_hit: i32,
    pub attr_fuerza: i32,
    pub attr_agilidad: i32,
    pub attr_inteligencia: i32,
    pub attr_constitucion: i32,
    pub id_head: i32,
    pub id_body: i32,
    pub id_helmet: i32,
    pub id_weapon: i32,
    pub id_shield: i32,
    pub id_arrow_slot: i32,
    pub id_ring_slot: i32,
    pub navegando: bool,
    pub home_map: i32,
    pub home_x: i32,
    pub home_y: i32,
    pub exp: i32,
    pub exp_next_level: i32,
    pub faction_rank: i32,
    pub faction_score: i32,
    #[sqlx(default)]
    pub faction_score_armada: i32,
    #[sqlx(default)]
    pub faction_score_caos: i32,
    #[sqlx(default)]
    pub criminales_matados: i32,
    #[sqlx(default)]
    pub ciudadanos_matados: i32,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct CharacterSummary {
    pub id: String,
    pub name: String,
    pub level: i32,
    pub id_clase: i32,
    pub map_id: i32,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MarketListingRow {
    pub id: String,
    pub seller_char_id: String,
    pub seller_name: String,
    pub item_id: i32,
    pub quantity: i32,
    pub unit_price: i32,
    pub status: String,
    pub expires_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
pub struct MarketClaimRow {
    pub id: String,
    pub char_id: String,
    pub claim_type: String,
    pub gold_amount: i32,
    pub item_id: Option<i32>,
    pub item_quantity: Option<i32>,
    pub created_at: String,
}
