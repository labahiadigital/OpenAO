#[cfg(test)]
mod tests {
    use crate::persistence::Database;

    async fn setup_test_db() -> Database {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        db.run_migrations().await.unwrap();
        db
    }

    async fn create_test_account(db: &Database, acc_id: &str, username: &str) {
        sqlx::query("INSERT INTO accounts (id, username, email, password_hash) VALUES (?, ?, ?, ?)")
            .bind(acc_id)
            .bind(username)
            .bind(format!("{username}@test.com"))
            .bind("hash")
            .execute(db.pool())
            .await
            .unwrap();
    }

    // ───────────────────────────────────────────
    // Character lifecycle (create → load → save → load roundtrip)
    // ───────────────────────────────────────────

    #[tokio::test]
    async fn character_create_and_load_roundtrip() {
        let db = setup_test_db().await;
        create_test_account(&db, "acc-001", "testuser").await;

        db.create_character_with_class("char-001", "acc-001", "TestHero", 1)
            .await
            .unwrap();

        let c = db
            .load_character("char-001")
            .await
            .unwrap()
            .expect("character should exist");

        assert_eq!(c.name, "TestHero");
        assert_eq!(c.id_clase, 1);
        assert_eq!(c.map_id, 1);
        assert_eq!(c.pos_x, 50);
        assert_eq!(c.pos_y, 50);
        assert_eq!(c.level, 1);
        assert!(!c.dead);
        assert!(!c.criminal);
        assert_eq!(c.faction, "none");
        assert_eq!(c.gold, 100);
    }

    #[tokio::test]
    async fn character_save_preserves_all_mutable_fields() {
        let db = setup_test_db().await;
        create_test_account(&db, "acc-002", "savetest").await;
        db.create_character_with_class("char-002", "acc-002", "SaveTest", 3)
            .await
            .unwrap();

        db.save_character_state(
            "char-002", 5, 30, 40, 200, 250, 100, 120, 5000, 25, 10000, 15000,
            false, "armada", true, 10, 50, 20, 18, 15, 22, 3, 25, 35, 5, 10,
            3, 7, 2, 99, 88, true, 1500, 3, 2, 500, 300, 200, 15, 10,
        )
        .await
        .unwrap();

        let c = db
            .load_character("char-002")
            .await
            .unwrap()
            .expect("character should exist");

        assert_eq!(c.map_id, 5);
        assert_eq!(c.pos_x, 30);
        assert_eq!(c.pos_y, 40);
        assert_eq!(c.hp, 200);
        assert_eq!(c.max_hp, 250);
        assert_eq!(c.mana, 100);
        assert_eq!(c.max_mana, 120);
        assert_eq!(c.gold, 5000);
        assert_eq!(c.level, 25);
        assert_eq!(c.exp, 10000);
        assert_eq!(c.exp_next_level, 15000);
        assert!(!c.dead);
        assert_eq!(c.faction, "armada");
        assert!(c.criminal);
        assert_eq!(c.min_hit, 10);
        assert_eq!(c.max_hit, 50);
        assert_eq!(c.attr_fuerza, 20);
        assert_eq!(c.attr_agilidad, 18);
        assert_eq!(c.attr_inteligencia, 15);
        assert_eq!(c.attr_constitucion, 22);
        assert_eq!(c.home_map, 3);
        assert_eq!(c.home_x, 25);
        assert_eq!(c.home_y, 35);
        assert_eq!(c.id_head, 5);
        assert_eq!(c.id_body, 10);
        assert_eq!(c.id_helmet, 3);
        assert_eq!(c.id_weapon, 7);
        assert_eq!(c.id_shield, 2);
        assert_eq!(c.id_arrow_slot, 99);
        assert_eq!(c.id_ring_slot, 88);
        assert!(c.navegando);
        assert_eq!(c.id_clase, 3);
        assert_eq!(c.faction_rank, 2);
        assert_eq!(c.faction_score, 500);
        assert_eq!(c.faction_score_armada, 300);
        assert_eq!(c.faction_score_caos, 200);
        assert_eq!(c.criminales_matados, 15);
        assert_eq!(c.ciudadanos_matados, 10);
    }

    // ───────────────────────────────────────────
    // All 8 classes create with correct initial stats
    // ───────────────────────────────────────────

    #[tokio::test]
    async fn all_eight_classes_create_with_correct_stats() {
        let db = setup_test_db().await;
        create_test_account(&db, "acc-classes", "classtest").await;

        let expected: &[(i32, &str, i32, i32)] = &[
            (1, "Mago", 80, 150),
            (2, "Clerigo", 100, 120),
            (3, "Guerrero", 120, 60),
            (4, "Asesino", 90, 80),
            (5, "Bardo", 90, 100),
            (6, "Druida", 95, 110),
            (7, "Paladin", 110, 80),
            (8, "Cazador", 95, 70),
        ];

        for (class_id, name, exp_hp, exp_mana) in expected {
            let char_id = format!("char-class-{class_id}");
            db.create_character_with_class(&char_id, "acc-classes", name, *class_id)
                .await
                .unwrap();
            let c = db.load_character(&char_id).await.unwrap().unwrap();

            assert_eq!(c.id_clase, *class_id, "class {name}: id_clase");
            assert_eq!(c.hp, *exp_hp, "class {name}: hp");
            assert_eq!(c.max_hp, *exp_hp, "class {name}: max_hp");
            assert_eq!(c.mana, *exp_mana, "class {name}: mana");
            assert_eq!(c.max_mana, *exp_mana, "class {name}: max_mana");
        }
    }

    // ───────────────────────────────────────────
    // Inventory persistence roundtrip
    // ───────────────────────────────────────────

    #[tokio::test]
    async fn inventory_roundtrip() {
        let db = setup_test_db().await;
        create_test_account(&db, "acc-inv", "invtest").await;
        db.create_character_with_class("char-inv", "acc-inv", "InvTest", 1)
            .await
            .unwrap();

        db.update_inventory_slot("char-inv", 0, 100, 5, true)
            .await
            .unwrap();
        db.update_inventory_slot("char-inv", 1, 200, 10, false)
            .await
            .unwrap();
        db.update_inventory_slot("char-inv", 5, 50, 1, true)
            .await
            .unwrap();

        let items = db.load_inventory("char-inv").await.unwrap();
        assert_eq!(items.len(), 3);

        let s0 = items.iter().find(|i| i.slot == 0).unwrap();
        assert_eq!(s0.item_id, 100);
        assert_eq!(s0.amount, 5);
        assert!(s0.equipped);

        let s1 = items.iter().find(|i| i.slot == 1).unwrap();
        assert_eq!(s1.item_id, 200);
        assert_eq!(s1.amount, 10);
        assert!(!s1.equipped);
    }

    // ───────────────────────────────────────────
    // Multi-character per account
    // ───────────────────────────────────────────

    #[tokio::test]
    async fn multi_character_per_account() {
        let db = setup_test_db().await;
        create_test_account(&db, "acc-multi", "multitest").await;

        db.create_character_with_class("char-m1", "acc-multi", "Hero1", 1)
            .await
            .unwrap();
        db.create_character_with_class("char-m2", "acc-multi", "Hero2", 3)
            .await
            .unwrap();
        db.create_character_with_class("char-m3", "acc-multi", "Hero3", 5)
            .await
            .unwrap();

        let chars = db.list_characters_by_account("acc-multi").await.unwrap();
        assert_eq!(chars.len(), 3);

        let deleted = db.delete_character("char-m2", "acc-multi").await.unwrap();
        assert!(deleted);

        let chars_after = db.list_characters_by_account("acc-multi").await.unwrap();
        assert_eq!(chars_after.len(), 2);
    }

    // ───────────────────────────────────────────
    // Character settings persistence
    // ───────────────────────────────────────────

    #[tokio::test]
    async fn character_settings_roundtrip() {
        let db = setup_test_db().await;
        create_test_account(&db, "acc-set", "settest").await;
        db.create_character_with_class("char-set", "acc-set", "SetTest", 1)
            .await
            .unwrap();

        assert!(db.get_character_settings("char-set").await.unwrap().is_none());

        let json = r#"{"volume":80,"music":true}"#;
        db.save_character_settings("char-set", json).await.unwrap();
        let loaded = db.get_character_settings("char-set").await.unwrap().unwrap();
        assert_eq!(loaded, json);

        let json2 = r#"{"volume":50}"#;
        db.save_character_settings("char-set", json2).await.unwrap();
        let loaded2 = db.get_character_settings("char-set").await.unwrap().unwrap();
        assert_eq!(loaded2, json2);
    }

    // ───────────────────────────────────────────
    // Ban/mute persistence
    // ───────────────────────────────────────────

    #[tokio::test]
    async fn ban_mute_persistence() {
        let db = setup_test_db().await;

        db.add_ban("acc-ban", "cheating", "admin1").await.unwrap();
        assert!(db.load_all_bans().await.unwrap().contains(&"acc-ban".to_string()));

        db.add_mute("char-mute", "spam", "admin1").await.unwrap();
        assert!(db.is_muted("char-mute").await.unwrap());

        db.remove_ban("acc-ban").await.unwrap();
        assert!(!db.load_all_bans().await.unwrap().contains(&"acc-ban".to_string()));

        db.remove_mute("char-mute").await.unwrap();
        assert!(!db.is_muted("char-mute").await.unwrap());
    }

    // ───────────────────────────────────────────
    // IP ban persistence
    // ───────────────────────────────────────────

    #[tokio::test]
    async fn ip_ban_persistence() {
        let db = setup_test_db().await;

        db.add_ip_ban("192.168.1.100", "botting", "admin1")
            .await
            .unwrap();
        assert!(db
            .load_all_ip_bans()
            .await
            .unwrap()
            .contains(&"192.168.1.100".to_string()));

        db.remove_ip_ban("192.168.1.100").await.unwrap();
        assert!(!db
            .load_all_ip_bans()
            .await
            .unwrap()
            .contains(&"192.168.1.100".to_string()));
    }

    // ───────────────────────────────────────────
    // Quest persistence roundtrip
    // ───────────────────────────────────────────

    #[tokio::test]
    async fn quest_save_load_roundtrip() {
        use crate::gameplay::quests::{ActiveQuest, ObjectiveProgress, PlayerQuestLog};

        let db = setup_test_db().await;
        create_test_account(&db, "acc-q", "questtest").await;
        db.create_character_with_class("char-q", "acc-q", "QTest", 1)
            .await
            .unwrap();

        let mut log = PlayerQuestLog::new();
        log.active.push(ActiveQuest {
            quest_id: 1,
            objectives: vec![ObjectiveProgress {
                current: 3,
                required: 5,
                completed: false,
            }],
        });
        log.active.push(ActiveQuest {
            quest_id: 2,
            objectives: vec![ObjectiveProgress {
                current: 0,
                required: 10,
                completed: false,
            }],
        });
        log.completed.push(99);

        db.save_quest_log("char-q", &log).await.unwrap();

        let loaded = db.load_quest_log("char-q").await.unwrap();
        assert_eq!(loaded.active.len(), 2);
        assert_eq!(loaded.active[0].quest_id, 1);
        assert_eq!(loaded.active[0].objectives[0].current, 3);
        assert_eq!(loaded.active[0].objectives[0].required, 5);
        assert_eq!(loaded.active[1].quest_id, 2);
        assert!(loaded.completed.contains(&99));
    }

    // ───────────────────────────────────────────
    // Batch transaction save (world save)
    // ───────────────────────────────────────────

    #[tokio::test]
    async fn batch_transaction_save() {
        let db = setup_test_db().await;
        create_test_account(&db, "acc-batch", "batchtest").await;
        db.create_character_with_class("char-b1", "acc-batch", "Batch1", 1)
            .await
            .unwrap();
        db.create_character_with_class("char-b2", "acc-batch", "Batch2", 3)
            .await
            .unwrap();

        let mut tx = db.begin_transaction().await.unwrap();

        Database::save_character_state_in_tx(
            &mut tx, "char-b1", 10, 20, 30, 100, 100, 50, 50, 999, 5, 1000, 2000,
            false, "none", false, 2, 10, 15, 15, 15, 15, 1, 50, 50, 1, 1, 0, 0,
            0, 0, 0, false, 0, 1, 0, 0, 0, 0, 0, 0,
        )
        .await
        .unwrap();

        Database::save_character_state_in_tx(
            &mut tx, "char-b2", 20, 30, 40, 200, 200, 100, 100, 5000, 10, 5000, 8000,
            false, "armada", false, 5, 20, 20, 20, 20, 20, 1, 50, 50, 2, 2, 0, 0,
            0, 0, 0, false, 0, 3, 0, 0, 0, 0, 0, 0,
        )
        .await
        .unwrap();

        tx.commit().await.unwrap();

        let c1 = db.load_character("char-b1").await.unwrap().unwrap();
        assert_eq!(c1.map_id, 10);
        assert_eq!(c1.gold, 999);

        let c2 = db.load_character("char-b2").await.unwrap().unwrap();
        assert_eq!(c2.map_id, 20);
        assert_eq!(c2.gold, 5000);
        assert_eq!(c2.faction, "armada");
    }

    // ───────────────────────────────────────────
    // Balance integration: all 8 classes HP/mana at multiple levels
    // ───────────────────────────────────────────

    #[tokio::test]
    async fn balance_all_classes_hp_mana_at_multiple_levels() {
        use crate::gameplay::balance::*;

        for class_id in 1..=8 {
            for level in [1, 10, 25, 36, 50] {
                let hp = get_max_hp_for_level(class_id, 15, level);
                assert!(hp > 0, "class {class_id} level {level}: HP must be positive, got {hp}");

                let mana = get_max_mana_for_level(class_id, 15, level);
                assert!(mana >= 0, "class {class_id} level {level}: mana must be non-negative, got {mana}");

                // HP should grow with level
                if level > 1 {
                    let hp_prev = get_max_hp_for_level(class_id, 15, level - 1);
                    assert!(hp >= hp_prev, "class {class_id}: HP should not decrease from level {} to {level}", level - 1);
                }
            }
        }
    }

    // ───────────────────────────────────────────
    // Gold clamping edge cases
    // ───────────────────────────────────────────

    #[tokio::test]
    async fn gold_clamp_edge_cases() {
        use crate::gameplay::balance::clamp_gold;

        assert_eq!(clamp_gold(0), 0);
        assert_eq!(clamp_gold(100), 100);
        assert_eq!(clamp_gold(2_147_483_647), 2_147_483_647);
        assert_eq!(clamp_gold(2_147_483_648), 2_147_483_647);
        assert_eq!(clamp_gold(i64::MAX), 2_147_483_647);
        assert_eq!(clamp_gold(-100), 0);
    }

    // ───────────────────────────────────────────
    // EXP curve monotonically increasing levels 1-50
    // ───────────────────────────────────────────

    #[tokio::test]
    async fn exp_curve_monotonic_1_to_46_then_flat() {
        use crate::gameplay::balance::get_legacy_exp_next_level;

        // Strictly increasing from 1..=46 (LAST_LEGACY_EXP_LEVEL)
        let mut prev = 0i32;
        for level in 1..=46 {
            let exp = get_legacy_exp_next_level(level);
            assert!(exp > prev, "exp at level {level} ({exp}) <= level {} ({prev})", level - 1);
            prev = exp;
        }

        // Flat from 47..=50 (capped at level 46 curve value)
        let cap = get_legacy_exp_next_level(46);
        for level in 47..=50 {
            assert_eq!(get_legacy_exp_next_level(level), cap,
                "exp at level {level} should equal cap at 46");
        }
    }

    // ───────────────────────────────────────────
    // Combat formula sanity: all classes produce positive values
    // ───────────────────────────────────────────

    #[tokio::test]
    async fn combat_formulas_all_classes_positive_values() {
        use crate::gameplay::combat_formulas::*;

        for class_id in 1..=8 {
            let evasion = poder_evasion(25, 15, class_id);
            assert!(evasion > 0.0, "class {class_id}: evasion should be positive, got {evasion}");

            let atk = poder_ataque_arma(25, 15, class_id, WeaponType::Melee);
            assert!(atk > 0.0, "class {class_id}: melee attack should be positive, got {atk}");

            let atk_proj = poder_ataque_arma(25, 15, class_id, WeaponType::Projectile);
            assert!(atk_proj > 0.0, "class {class_id}: projectile attack should be positive, got {atk_proj}");

            let dmg = calcular_dmg(10, 30, 15, class_id, 5, 15, 0, 0, false);
            assert!(dmg > 0, "class {class_id}: melee damage should be positive, got {dmg}");
        }
    }
}
