use rand::Rng;

pub const MAX_LEVEL: i32 = 50;
pub const MAX_EXP_LEVEL: i32 = 50;
const LAST_LEGACY_EXP_LEVEL: i32 = 46;
pub const MAX_GOLD: i64 = 2_147_483_647;

struct ClassProgress {
    vida: f64,
    mana_inicial: f64,
    mult_mana: f64,
    hit_pre_36: i32,
    hit_post_36: i32,
}

const fn cp(vida: f64, mana_inicial: f64, mult_mana: f64, hit_pre_36: i32, hit_post_36: i32) -> ClassProgress {
    ClassProgress { vida, mana_inicial, mult_mana, hit_pre_36, hit_post_36 }
}

fn get_class_progress(class_id: i32) -> &'static ClassProgress {
    static PROGRESS: std::sync::LazyLock<[ClassProgress; 11]> = std::sync::LazyLock::new(|| [
        cp(7.5,  8.33, 2.65, 1, 1),  // 1: Mago
        cp(8.5,  2.5,  2.0,  2, 2),  // 2: Clerigo
        cp(10.5, 0.0,  0.0,  3, 2),  // 3: Guerrero
        cp(9.0,  2.5,  1.0,  3, 2),  // 4: Asesino
        cp(8.5,  2.5,  2.0,  2, 2),  // 5: Bardo (original ID 6)
        cp(8.5,  2.5,  2.0,  2, 2),  // 6: Druida (original ID 7)
        cp(10.0, 2.5,  1.0,  3, 2),  // 7: Paladin (original ID 8)
        cp(10.0, 0.0,  0.0,  3, 2),  // 8: Cazador (original ID 9)
        cp(10.0, 0.0,  0.0,  3, 2),  // 9: Trabajador (original ID 10)
        cp(8.5,  0.0,  0.0,  3, 2),  // 10: Pirata (original ID 11)
        cp(10.0, 0.0,  0.0,  3, 2),  // 11: Bandido
    ]);
    let idx = ((class_id - 1) as usize).min(PROGRESS.len() - 1);
    &PROGRESS[idx]
}

fn clamp_level(level: i32) -> i32 {
    level.clamp(1, MAX_LEVEL)
}

pub fn clamp_gold(gold: i64) -> i64 {
    gold.clamp(0, MAX_GOLD)
}

pub fn get_max_hp_for_level(class_id: i32, constitucion: i32, level: i32) -> i32 {
    let safe_level = clamp_level(level);
    let cp = get_class_progress(class_id);
    let hp_avg = cp.vida - (21.0 - constitucion as f64) * 0.5;
    let total = constitucion as f64 + hp_avg * (safe_level - 1) as f64;
    total.round() as i32
}

pub fn get_max_mana_for_level(class_id: i32, inteligencia: i32, level: i32) -> i32 {
    let safe_level = clamp_level(level);
    let cp = get_class_progress(class_id);
    let total = inteligencia as f64 * cp.mana_inicial
        + cp.mult_mana * inteligencia as f64 * (safe_level - 1) as f64;
    total.round().max(0.0) as i32
}

fn get_hit_modifier_for_level(class_id: i32, level: i32) -> i32 {
    let safe_level = clamp_level(level);
    let cp = get_class_progress(class_id);
    if safe_level <= 1 {
        return 0;
    }
    if safe_level <= 36 {
        return (safe_level - 1) * cp.hit_pre_36;
    }
    35 * cp.hit_pre_36 + (safe_level - 36) * cp.hit_post_36
}

pub fn get_min_hit_for_level(class_id: i32, level: i32) -> i32 {
    1 + get_hit_modifier_for_level(class_id, level)
}

pub fn get_max_hit_for_level(class_id: i32, level: i32) -> i32 {
    2 + get_hit_modifier_for_level(class_id, level)
}

pub fn get_legacy_exp_next_level(level: i32) -> i32 {
    let safe_level = level.clamp(1, MAX_EXP_LEVEL);
    let exp_curve_level = safe_level.min(LAST_LEGACY_EXP_LEVEL);
    let mut exp_next = 300i64;
    for current_level in 2..=exp_curve_level {
        let mult = if current_level < 15 {
            1.4
        } else if current_level < 21 {
            1.35
        } else if current_level < 33 {
            1.3
        } else if current_level < 41 {
            1.225
        } else {
            1.25
        };
        exp_next = (exp_next as f64 * mult) as i64;
    }
    exp_next as i32
}

// ---------------------------------------------------------------------------
// Combat stats (used for attack/defense calculations)
// ---------------------------------------------------------------------------

pub struct CombatStats {
    pub min_hit: i32,
    pub max_hit: i32,
    pub defense: i32,
    pub evasion: i32,
    pub accuracy: i32,
    pub magic_damage_bonus: i32,
    pub magic_resistance: i32,
}

#[allow(clippy::too_many_arguments)]
pub fn compute_player_stats(
    level: i32,
    class_id: i32,
    base_min_hit: i32,
    base_max_hit: i32,
    fuerza: i32,
    agilidad: i32,
    weapon_min: i32,
    weapon_max: i32,
    armor_def: i32,
    shield_def: i32,
    helmet_def: i32,
) -> CombatStats {
    let str_bonus = fuerza / 5;
    let agi_bonus = agilidad / 5;

    let class_attack_mult = match class_id {
        1 => 80,  // Mago
        2 => 85,  // Clerigo
        3 => 110, // Guerrero
        4 => 105, // Asesino
        5 => 90,  // Bardo
        6 => 85,  // Druida
        7 => 100, // Paladin
        8 => 100, // Cazador
        _ => 100,
    };

    let class_defense_mult = match class_id {
        1 => 70,
        2 => 90,
        3 => 110,
        4 => 85,
        5 => 80,
        6 => 80,
        7 => 105,
        8 => 90,
        _ => 100,
    };

    let level_bonus = level / 5;

    let raw_min = base_min_hit + weapon_min + str_bonus + level_bonus;
    let raw_max = base_max_hit + weapon_max + str_bonus + level_bonus;

    let min_hit = (raw_min * class_attack_mult / 100).max(1);
    let max_hit = (raw_max * class_attack_mult / 100).max(min_hit);

    let defense = ((armor_def + shield_def + helmet_def) * class_defense_mult / 100).max(0);
    let evasion = agi_bonus + level_bonus;
    let accuracy = agi_bonus + level_bonus + fuerza / 10;

    let magic_damage_bonus = match class_id {
        1 => level / 3 + fuerza / 8,
        2 => level / 4,
        6 => level / 4,
        _ => 0,
    };

    let magic_resistance = match class_id {
        7 => level / 5 + 5,
        3 => level / 8,
        _ => 0,
    };

    CombatStats {
        min_hit,
        max_hit,
        defense,
        evasion,
        accuracy,
        magic_damage_bonus,
        magic_resistance,
    }
}

pub fn roll_physical_damage(attacker: &CombatStats, defender: &CombatStats) -> i32 {
    let mut rng = rand::rng();

    let hit_chance = 75 + attacker.accuracy - defender.evasion;
    let hit_chance = hit_chance.clamp(10, 95);

    if rng.random_range(0..100) >= hit_chance {
        return 0;
    }

    let raw = rng.random_range(attacker.min_hit..=attacker.max_hit);
    (raw - defender.defense).max(1)
}

pub fn roll_spell_damage(
    base_damage: i32,
    caster_magic_bonus: i32,
    target_magic_resistance: i32,
) -> i32 {
    let mut rng = rand::rng();
    let variation = rng.random_range(90..=110);
    let raw = base_damage * variation / 100 + caster_magic_bonus;
    (raw - target_magic_resistance).max(1)
}

pub fn gold_for_level(gold: i32, level: i32) -> i32 {
    let max_gold = match level {
        1..=10 => 50_000,
        11..=25 => 200_000,
        26..=35 => 500_000,
        36..=45 => 2_000_000,
        _ => 10_000_000,
    };
    gold.min(max_gold)
}

pub fn exp_for_kill(npc_exp: i32, player_level: i32, npc_level_approx: i32) -> i32 {
    let level_diff = player_level - npc_level_approx;
    let modifier = match level_diff {
        ..=-5 => 150,
        -4..=-1 => 120,
        0..=4 => 100,
        5..=10 => 70,
        _ => 40,
    };
    (npc_exp * modifier / 100).max(1)
}

// ---------------------------------------------------------------------------
// Recalculate full stats for a player on level-up (uses exact original formulas)
// ---------------------------------------------------------------------------

pub fn recalc_on_level_up(
    class_id: i32,
    level: i32,
    constitucion: i32,
    inteligencia: i32,
) -> (i32, i32, i32, i32) {
    let max_hp = get_max_hp_for_level(class_id, constitucion, level);
    let max_mana = get_max_mana_for_level(class_id, inteligencia, level);
    let min_hit = get_min_hit_for_level(class_id, level);
    let max_hit = get_max_hit_for_level(class_id, level);
    (max_hp, max_mana, min_hit, max_hit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warrior_has_higher_attack_than_mage() {
        let warrior = compute_player_stats(10, 3, 1, 5, 20, 15, 5, 10, 10, 5, 3);
        let mage = compute_player_stats(10, 1, 1, 5, 15, 15, 2, 4, 5, 0, 0);
        assert!(warrior.max_hit > mage.max_hit);
    }

    #[test]
    fn damage_is_at_least_one() {
        let stats = CombatStats {
            min_hit: 1, max_hit: 1, defense: 0, evasion: 0, accuracy: 100,
            magic_damage_bonus: 0, magic_resistance: 0,
        };
        let defender = CombatStats {
            min_hit: 0, max_hit: 0, defense: 999, evasion: 0, accuracy: 0,
            magic_damage_bonus: 0, magic_resistance: 0,
        };
        let mut hits = 0;
        for _ in 0..100 {
            let dmg = roll_physical_damage(&stats, &defender);
            if dmg > 0 { hits += 1; }
        }
        assert!(hits > 0);
    }

    #[test]
    fn gold_clamp_respects_level() {
        assert_eq!(gold_for_level(999_999, 5), 50_000);
        assert_eq!(gold_for_level(100_000, 20), 100_000);
        assert_eq!(gold_for_level(999_999_999, 50), 10_000_000);
    }

    #[test]
    fn exp_scaling_by_level_diff() {
        let base = 100;
        assert!(exp_for_kill(base, 5, 15) > exp_for_kill(base, 5, 5));
        assert!(exp_for_kill(base, 50, 5) < exp_for_kill(base, 5, 5));
    }

    #[test]
    fn spell_damage_at_least_one() {
        let dmg = roll_spell_damage(10, 0, 999);
        assert!(dmg >= 1);
    }

    #[test]
    fn max_hp_matches_original_formula() {
        // Mage (class 1) with CON=18, level 10:
        // vida=7.5, hp_avg = 7.5 - (21-18)*0.5 = 6.0
        // total = 18 + 6.0 * 9 = 72
        assert_eq!(get_max_hp_for_level(1, 18, 10), 72);

        // Warrior (class 3) with CON=21, level 1:
        // total = 21 (only constitution at level 1)
        assert_eq!(get_max_hp_for_level(3, 21, 1), 21);

        // Warrior (class 3) with CON=20, level 50:
        // vida=10.5, hp_avg = 10.5 - (21-20)*0.5 = 10.0
        // total = 20 + 10.0 * 49 = 510
        assert_eq!(get_max_hp_for_level(3, 20, 50), 510);
    }

    #[test]
    fn max_mana_matches_original_formula() {
        // Mage (class 1) with INT=21, level 10:
        // manaInicial=8.33, multMana=2.65
        // total = 21*8.33 + 2.65*21*9 = 174.93 + 500.85 = 675.78 -> 676
        assert_eq!(get_max_mana_for_level(1, 21, 10), 676);

        // Warrior (class 3) with INT=15, level 10:
        // manaInicial=0, multMana=0 => 0
        assert_eq!(get_max_mana_for_level(3, 15, 10), 0);
    }

    #[test]
    fn hit_modifier_matches_original() {
        // Warrior (class 3): hitPre36=3, hitPost36=2
        // level 1 -> modifier = 0, min=1, max=2
        assert_eq!(get_min_hit_for_level(3, 1), 1);
        assert_eq!(get_max_hit_for_level(3, 1), 2);

        // level 36: (36-1)*3 = 105, min=106, max=107
        assert_eq!(get_min_hit_for_level(3, 36), 106);
        assert_eq!(get_max_hit_for_level(3, 36), 107);

        // level 50: 35*3 + (50-36)*2 = 105 + 28 = 133, min=134, max=135
        assert_eq!(get_min_hit_for_level(3, 50), 134);
        assert_eq!(get_max_hit_for_level(3, 50), 135);

        // Mage (class 1): hitPre36=1, hitPost36=1
        // level 20: (20-1)*1 = 19, min=20, max=21
        assert_eq!(get_min_hit_for_level(1, 20), 20);
        assert_eq!(get_max_hit_for_level(1, 20), 21);
    }

    #[test]
    fn exp_curve_matches_original() {
        assert_eq!(get_legacy_exp_next_level(1), 300);
        assert_eq!(get_legacy_exp_next_level(2), 420); // 300 * 1.4
        assert_eq!(get_legacy_exp_next_level(3), 588); // 420 * 1.4
    }

    #[test]
    fn gold_clamp_respects_max() {
        assert_eq!(clamp_gold(3_000_000_000), MAX_GOLD);
        assert_eq!(clamp_gold(-100), 0);
        assert_eq!(clamp_gold(1000), 1000);
    }

    #[test]
    fn recalc_on_level_up_is_consistent() {
        let (hp, mana, min_h, max_h) = recalc_on_level_up(3, 10, 20, 15);
        assert_eq!(hp, get_max_hp_for_level(3, 20, 10));
        assert_eq!(mana, get_max_mana_for_level(3, 15, 10));
        assert_eq!(min_h, get_min_hit_for_level(3, 10));
        assert_eq!(max_h, get_max_hit_for_level(3, 10));
    }

    #[test]
    fn all_classes_have_positive_hp_at_level_50() {
        for class_id in 1..=11 {
            let hp = get_max_hp_for_level(class_id, 20, 50);
            assert!(hp > 0, "class {} should have positive HP at level 50", class_id);
        }
    }

    #[test]
    fn exp_curve_breakpoints_monotonically_increasing() {
        let mut prev = get_legacy_exp_next_level(1);
        for lvl in 2..=50 {
            let current = get_legacy_exp_next_level(lvl);
            assert!(current >= prev, "level {} exp ({}) should be >= level {} exp ({})", lvl, current, lvl-1, prev);
            prev = current;
        }
    }

    #[test]
    fn exp_curve_level_1_is_300() {
        assert_eq!(get_legacy_exp_next_level(1), 300);
    }

    #[test]
    fn clamp_level_bounds() {
        assert_eq!(clamp_level(0), 1);
        assert_eq!(clamp_level(1), 1);
        assert_eq!(clamp_level(50), 50);
        assert_eq!(clamp_level(100), 50);
    }
}
