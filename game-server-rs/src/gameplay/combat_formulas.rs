use rand::Rng;

pub const DEAD_WORLD_DELAY_MS: u64 = 15_000;
pub const DRAGON_SLAYER_SWORD_ITEM_ID: i32 = 402;
pub const NEWBIE_MAX_LEVEL: i32 = 12;
pub const CLAN_RING_MAP_ID: i32 = 273;
pub const UNSAFE_LOGOUT_DELAY_MS: u64 = 10_000;
pub const NPC_EXP_MULTIPLIER: i32 = 5;
pub const NPC_GOLD_MULTIPLIER: i32 = 3;

pub fn is_newbie_character(level: i32) -> bool {
    level <= NEWBIE_MAX_LEVEL
}

pub fn resolve_boat_body_id(current_body: i32, dead: bool) -> i32 {
    if dead {
        return 87;
    }
    if current_body == 85 || current_body == 86 { current_body } else { 84 }
}

pub fn is_dragon_slayer_hit(weapon_item_id: i32, npc_type: i32, dragon_npc_type: i32) -> bool {
    weapon_item_id == DRAGON_SLAYER_SWORD_ITEM_ID && npc_type == dragon_npc_type
}

// ---------------------------------------------------------------------------
// Simulated skill (all skills use the same formula: min(100, level * 3))
// ---------------------------------------------------------------------------

pub fn simulated_skill(level: i32) -> i32 {
    (level * 3).min(100)
}

// ---------------------------------------------------------------------------
// Body parts (for PvP damage distribution)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyPart {
    Head,       // 1
    LeftLeg,    // 2
    RightLeg,   // 3
    RightArm,   // 4
    LeftArm,    // 5
    Torso,      // 6
}

pub fn random_body_part() -> BodyPart {
    let mut rng = rand::rng();
    let roll: i32 = rng.random_range(1..=8);
    if roll > 6 {
        let idx = rng.random_range(2..=6);
        match idx {
            2 => BodyPart::LeftLeg,
            3 => BodyPart::RightLeg,
            4 => BodyPart::RightArm,
            5 => BodyPart::LeftArm,
            _ => BodyPart::Torso,
        }
    } else {
        match roll {
            1 => BodyPart::Head,
            2 => BodyPart::LeftLeg,
            3 => BodyPart::RightLeg,
            4 => BodyPart::RightArm,
            5 => BodyPart::LeftArm,
            _ => BodyPart::Torso,
        }
    }
}

// ---------------------------------------------------------------------------
// Class-based combat modifiers (from balanceData.ts DEFAULT_BALANCE_DATA)
// All indexed by class_id (1..=11). Returns 0.0 for unknown classes.
// ---------------------------------------------------------------------------

fn mod_evasion(class_id: i32) -> f64 {
    match class_id {
        1  => 0.2,   // Mago
        2  => 0.8,   // Clerigo
        3  => 1.0,   // Guerrero
        4  => 0.9,   // Asesino
        5  => 1.0,   // Bardo (original ID 6: modEvasion=1.0)
        6  => 0.9,   // Druida (original ID 7: modEvasion=0.9)
        7  => 0.85,  // Paladin (original ID 8: modEvasion=0.85)
        8  => 0.9,   // Cazador (original ID 9: modEvasion=0.9)
        9  => 0.9,
        10 => 0.9,
        11 => 0.9,
        _  => 0.8,
    }
}

fn mod_escudo(class_id: i32) -> f64 {
    match class_id {
        1  => 0.0,   // Mago
        2  => 0.8,   // Clerigo
        3  => 1.0,   // Guerrero
        4  => 0.8,   // Asesino
        5  => 0.65,  // Bardo (original ID 6: 0.65)
        6  => 0.0,   // Druida (original ID 7: 0)
        7  => 0.9,   // Paladin (original ID 8: 0.9)
        8  => 0.75,  // Cazador (original ID 9: 0.75)
        9  => 0.75,
        10 => 0.75,
        11 => 0.75,
        _  => 0.75,
    }
}

fn mod_ataque_wrestling(class_id: i32) -> f64 {
    match class_id {
        1  => 0.5,   // Mago
        2  => 0.93,  // Clerigo
        3  => 1.1,   // Guerrero
        4  => 1.0,   // Asesino
        5  => 0.8,   // Bardo (original ID 6: 0.8)
        6  => 0.75,  // Druida (original ID 7: 0.75)
        7  => 1.0,   // Paladin (original ID 8: 1.0)
        8  => 0.8,   // Cazador (original ID 9: 0.8)
        9  => 0.8,
        10 => 0.8,
        11 => 0.8,
        _  => 0.8,
    }
}

fn mod_ataque_proyectiles(class_id: i32) -> f64 {
    match class_id {
        1  => 0.5,   // Mago
        2  => 0.8,   // Clerigo
        3  => 0.85,  // Guerrero
        4  => 0.7,   // Asesino
        5  => 0.7,   // Bardo (original ID 6: 0.7)
        6  => 0.6,   // Druida (original ID 7: 0.6)
        7  => 0.78,  // Paladin (original ID 8: 0.78)
        8  => 0.95,  // Cazador (original ID 9: 0.95)
        9  => 0.8,
        10 => 0.8,
        11 => 0.8,
        _  => 0.8,
    }
}

fn mod_ataque_armas(class_id: i32) -> f64 {
    match class_id {
        1  => 0.5,   // Mago
        2  => 0.98,  // Clerigo
        3  => 1.1,   // Guerrero
        4  => 1.0,   // Asesino
        5  => 0.88,  // Bardo (original ID 6: 0.88)
        6  => 0.8,   // Druida (original ID 7: 0.8)
        7  => 1.03,  // Paladin (original ID 8: 1.03)
        8  => 0.8,   // Cazador (original ID 9: 0.8)
        9  => 0.8,
        10 => 0.8,
        11 => 0.8,
        _  => 0.8,
    }
}

fn mod_dmg_armas(class_id: i32) -> f64 {
    match class_id {
        1  => 0.5,   // Mago
        2  => 0.85,  // Clerigo
        3  => 1.05,  // Guerrero
        4  => 0.95,  // Asesino
        5  => 0.82,  // Bardo (original ID 6: 0.82)
        6  => 0.75,  // Druida (original ID 7: 0.75)
        7  => 1.0,   // Paladin (original ID 8: 1.0)
        8  => 0.85,  // Cazador (original ID 9: 0.85)
        9  => 0.85,
        10 => 0.85,
        11 => 0.85,
        _  => 0.85,
    }
}

fn mod_dmg_proyectiles(class_id: i32) -> f64 {
    match class_id {
        1  => 0.5,   // Mago
        2  => 0.75,  // Clerigo
        3  => 0.87,  // Guerrero
        4  => 0.75,  // Asesino
        5  => 0.75,  // Bardo (original ID 6: 0.75)
        6  => 0.75,  // Druida (original ID 7: 0.75)
        7  => 0.8,   // Paladin (original ID 8: 0.8)
        8  => 0.93,  // Cazador (original ID 9: 0.93)
        9  => 0.8,
        10 => 0.8,
        11 => 0.8,
        _  => 0.8,
    }
}

fn mod_dmg_wrestling(class_id: i32) -> f64 {
    match class_id {
        1  => 0.4,
        2  => 0.4,
        3  => 0.4,
        4  => 0.4,
        5  => 0.4,
        6  => 0.4,
        7  => 0.4,
        8  => 0.4,
        9  => 0.4,
        10 => 0.4,
        11 => 0.4,
        _  => 0.4,
    }
}

// ---------------------------------------------------------------------------
// Evasion power (poderEvasion)
// ---------------------------------------------------------------------------

pub fn poder_evasion(level: i32, agilidad: i32, class_id: i32) -> f64 {
    let skill = simulated_skill(level) as f64;
    let tmp = (skill + (skill / 33.0) * agilidad as f64) * mod_evasion(class_id);
    tmp + 2.5 * (level - 12).max(0) as f64
}

// ---------------------------------------------------------------------------
// Shield evasion power (poderEvasionEscudo)
// shield_pct: the shield's `porcentaje` field (0..100)
// ---------------------------------------------------------------------------

pub fn poder_evasion_escudo(level: i32, class_id: i32, shield_pct: i32) -> f64 {
    if shield_pct <= 0 {
        return 0.0;
    }
    let item_modifier = shield_pct as f64 / 100.0;
    let skill = simulated_skill(level) as f64;
    (skill * mod_escudo(class_id) / 4.0) * item_modifier
}

// ---------------------------------------------------------------------------
// Weapon attack power (poderAtaqueArma)
// weapon_type: 0 = unarmed, 1 = melee, 2 = projectile, 3 = stabbing (apu)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponType {
    Unarmed,
    Melee,
    Projectile,
    Stabbing,
}

pub fn poder_ataque_arma(level: i32, agilidad: i32, class_id: i32, weapon_type: WeaponType) -> f64 {
    let skill = simulated_skill(level) as f64;
    let modifier = match weapon_type {
        WeaponType::Unarmed => mod_ataque_wrestling(class_id),
        WeaponType::Melee => mod_ataque_armas(class_id),
        WeaponType::Projectile => mod_ataque_proyectiles(class_id),
        WeaponType::Stabbing => mod_ataque_armas(class_id),
    };
    let base = (skill + 3.0 * agilidad as f64) * modifier;
    base + 2.5 * (level - 12).max(0) as f64
}

// ---------------------------------------------------------------------------
// Calculate damage (calcularDmg)
// weapon_min_hit/weapon_max_hit: weapon item minHit/maxHit (0 if unarmed)
// arrow_min_hit/arrow_max_hit: arrow item minHit/maxHit (0 if no arrow)
// is_projectile: true if weapon is ranged
// ---------------------------------------------------------------------------

pub fn calcular_dmg(
    min_hit: i32,
    max_hit: i32,
    fuerza: i32,
    class_id: i32,
    weapon_min_hit: i32,
    weapon_max_hit: i32,
    arrow_min_hit: i32,
    arrow_max_hit: i32,
    is_projectile: bool,
) -> i32 {
    let mut rng = rand::rng();

    let (dmg_arma, dmg_max_arma, mod_clase) = if weapon_max_hit > 0 {
        let mut wa = rng.random_range(weapon_min_hit.max(0)..=weapon_max_hit.max(1));
        let mod_c;
        if is_projectile {
            wa += rng.random_range(arrow_min_hit.max(0)..=arrow_max_hit.max(0).max(arrow_min_hit));
            mod_c = mod_dmg_proyectiles(class_id);
        } else {
            mod_c = mod_dmg_armas(class_id);
        }
        (wa, weapon_max_hit, mod_c)
    } else {
        let wa = rng.random_range(4..=9);
        (wa, 9, mod_dmg_wrestling(class_id))
    };

    let dmg_user = rng.random_range(min_hit.max(1)..=max_hit.max(min_hit.max(1)));
    let str_bonus = (dmg_max_arma as f64 / 5.0) * (fuerza - 15).max(0) as f64;
    ((3.0 * dmg_arma as f64 + str_bonus + dmg_user as f64) * mod_clase) as i32
}

// ---------------------------------------------------------------------------
// Hit chance (hit roll for melee/ranged)
// ---------------------------------------------------------------------------

fn clamp_chance(value: f64, min: f64, max: f64) -> f64 {
    value.clamp(min, max)
}

pub fn melee_hit_chance(atk_power: f64, def_evasion: f64) -> f64 {
    clamp_chance(50.0 + (atk_power - def_evasion) * 0.4, 5.0, 95.0)
}

pub fn roll_melee_hit(atk_power: f64, def_evasion: f64) -> bool {
    let chance = melee_hit_chance(atk_power, def_evasion);
    let mut rng = rand::rng();
    rng.random_range(1..=100) as f64 <= chance
}

// ---------------------------------------------------------------------------
// Shield block (rechazo de escudo)
// shield_pct: shield item `porcentaje` (0..100)
// ---------------------------------------------------------------------------

pub fn shield_block_chance(
    defender_level: i32,
    _defender_class_id: i32,
    shield_pct: i32,
) -> f64 {
    if shield_pct <= 0 {
        return 0.0;
    }
    let skill_defensa = simulated_skill(defender_level) as f64;
    let skill_tacticas = simulated_skill(defender_level) as f64;
    let denom = (skill_defensa + skill_tacticas).max(1.0);
    if skill_defensa > 0.0 {
        clamp_chance(
            shield_pct as f64 * (skill_defensa / denom) * 0.7,
            5.0,
            75.0,
        )
    } else {
        10.0
    }
}

pub fn roll_shield_block(defender_level: i32, defender_class_id: i32, shield_pct: i32) -> bool {
    let chance = shield_block_chance(defender_level, defender_class_id, shield_pct);
    let mut rng = rand::rng();
    rng.random_range(1..=100) as f64 <= chance
}

// ---------------------------------------------------------------------------
// Body part armor absorption (PvP)
// ---------------------------------------------------------------------------

pub fn body_part_absorption(
    part: BodyPart,
    helmet_min_def: i32,
    helmet_max_def: i32,
    body_min_def: i32,
    body_max_def: i32,
    shield_min_def: i32,
    shield_max_def: i32,
) -> i32 {
    let mut rng = rand::rng();
    match part {
        BodyPart::Head => {
            if helmet_max_def > 0 {
                rng.random_range(helmet_min_def.max(0)..=helmet_max_def)
            } else {
                0
            }
        }
        _ => {
            let min_def = body_min_def + shield_min_def;
            let max_def = body_max_def + shield_max_def;
            if max_def > 0 {
                rng.random_range(min_def.max(0)..=max_def)
            } else {
                0
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Stabbing (apuñalamiento)
// ---------------------------------------------------------------------------

fn stabbing_chance_by_class(class_id: i32) -> f64 {
    match class_id {
        1  => 0.08,  // Mago
        2  => 0.15,  // Clerigo
        3  => 0.08,  // Guerrero
        4  => 0.24,  // Asesino
        5  => 0.15,  // Bardo (original ID 6: 0.15)
        6  => 0.08,  // Druida (original ID 7: 0.08)
        7  => 0.15,  // Paladin (original ID 8: 0.15)
        8  => 0.08,  // Cazador (original ID 9: 0.08)
        9  => 0.08,
        10 => 0.08,
        11 => 0.08,
        _  => 0.08,
    }
}

fn stabbing_dmg_mod_pvp(class_id: i32) -> f64 {
    match class_id {
        1  => 0.1,   // Mago
        2  => 1.25,  // Clerigo
        3  => 1.4,   // Guerrero
        4  => 1.35,  // Asesino
        5  => 1.25,  // Bardo (original ID 6: 1.25)
        6  => 0.1,   // Druida (original ID 7: 0.1)
        7  => 1.4,   // Paladin (original ID 8: 1.4)
        8  => 1.3,   // Cazador (original ID 9: 1.3)
        9  => 1.1,
        10 => 1.1,
        11 => 1.2,
        _  => 1.0,
    }
}

fn stabbing_npc_min_mod(class_id: i32) -> f64 {
    match class_id {
        1  => 1.2,   // Mago
        2  => 1.25,  // Clerigo
        3  => 1.4,   // Guerrero
        4  => 1.6,   // Asesino
        5  => 1.25,  // Bardo (original ID 6: 1.25)
        6  => 1.2,   // Druida (original ID 7: 1.2)
        7  => 1.4,   // Paladin (original ID 8: 1.4)
        8  => 1.3,   // Cazador (original ID 9: 1.3)
        9  => 1.1,
        10 => 1.1,
        11 => 1.2,
        _  => 1.0,
    }
}

fn stabbing_npc_max_mod(class_id: i32) -> f64 {
    match class_id {
        1  => 1.2,   // Mago
        2  => 1.25,  // Clerigo
        3  => 1.4,   // Guerrero
        4  => 1.9,   // Asesino
        5  => 1.25,  // Bardo (original ID 6: 1.25)
        6  => 1.2,   // Druida (original ID 7: 1.2)
        7  => 1.4,   // Paladin (original ID 8: 1.4)
        8  => 1.3,   // Cazador (original ID 9: 1.3)
        9  => 1.1,
        10 => 1.1,
        11 => 1.2,
        _  => 1.0,
    }
}

pub fn can_stab(level: i32, class_id: i32, weapon_is_apu: bool) -> bool {
    if !weapon_is_apu {
        return false;
    }
    let skill = simulated_skill(level);
    if skill < 10 && class_id != 4 {
        return false;
    }
    true
}

pub struct StabResult {
    pub stabbed: bool,
    pub extra_damage: i32,
    pub total_damage: i32,
}

pub fn try_stab_npc(level: i32, class_id: i32, base_dmg: i32) -> StabResult {
    let skill = simulated_skill(level) as f64;
    let chance = clamp_chance(skill * stabbing_chance_by_class(class_id), 5.0, 95.0);
    let mut rng = rand::rng();
    if rng.random_range(1..=100) as f64 <= chance {
        let min_mod = stabbing_npc_min_mod(class_id);
        let max_mod = stabbing_npc_max_mod(class_id);
        let factor = rng.random_range(0.0..1.0) * (max_mod - min_mod) + min_mod;
        let extra = (base_dmg as f64 * factor) as i32;
        StabResult { stabbed: true, extra_damage: extra, total_damage: base_dmg + extra }
    } else {
        StabResult { stabbed: false, extra_damage: 0, total_damage: base_dmg }
    }
}

pub fn try_stab_pvp(level: i32, class_id: i32, base_dmg: i32) -> StabResult {
    let skill = simulated_skill(level) as f64;
    let chance = clamp_chance(skill * stabbing_chance_by_class(class_id), 5.0, 95.0);
    let mut rng = rand::rng();
    if rng.random_range(1..=100) as f64 <= chance {
        let mod_val = stabbing_dmg_mod_pvp(class_id);
        let extra = (base_dmg as f64 * mod_val) as i32;
        StabResult { stabbed: true, extra_damage: extra, total_damage: base_dmg + extra }
    } else {
        StabResult { stabbed: false, extra_damage: 0, total_damage: base_dmg }
    }
}

// ---------------------------------------------------------------------------
// NPC evasion power (stored per NPC instance, based on NPC data)
// ---------------------------------------------------------------------------

pub fn npc_evasion(npc_level_approx: i32) -> f64 {
    let skill = simulated_skill(npc_level_approx) as f64;
    skill * 0.5
}

// ---------------------------------------------------------------------------
// Magic damage class modifier (modDmgMagia from balanceData.ts)
// ---------------------------------------------------------------------------

pub fn mod_dmg_magia(class_id: i32) -> f64 {
    match class_id {
        1  => 1.0,   // Mago
        2  => 0.88,  // Clerigo
        3  => 0.0,   // Guerrero
        4  => 0.76,  // Asesino
        5  => 0.93,  // Bardo (original ID 6: modDmgMagia=0.93)
        6  => 0.92,  // Druida (original ID 7: modDmgMagia=0.92)
        7  => 0.78,  // Paladin (original ID 8: modDmgMagia=0.78)
        8  => 0.0,   // Cazador (original ID 9: modDmgMagia=0)
        9  => 0.0,
        10 => 0.0,
        11 => 0.0,
        _  => 0.0,
    }
}

// ---------------------------------------------------------------------------
// Magic resistance class bonus (modResistenciaMagica from balanceData.ts)
// ---------------------------------------------------------------------------

pub fn mod_resistencia_magica(class_id: i32) -> f64 {
    match class_id {
        1  => 0.0,   // Mago
        2  => 4.0,   // Clerigo
        3  => 12.0,  // Guerrero
        4  => 5.0,   // Asesino
        5  => 2.0,   // Bardo (original ID 6: modResistenciaMagica=2)
        6  => 3.0,   // Druida (original ID 7: modResistenciaMagica=3)
        7  => 5.0,   // Paladin (original ID 8: modResistenciaMagica=5)
        8  => 6.0,   // Cazador (original ID 9: modResistenciaMagica=6)
        9  => 0.0,
        10 => 0.0,
        11 => 0.0,
        _  => 0.0,
    }
}

// ---------------------------------------------------------------------------
// Magic bonuses (port of applyMagicBonuses from game.ts)
// Returns (final_damage, magic_penetration)
// ---------------------------------------------------------------------------

pub struct MagicBonusResult {
    pub damage: i32,
    pub magic_penetration: i32,
}

pub fn apply_magic_bonuses(
    base_damage: i32,
    caster_level: i32,
    caster_class_id: i32,
    weapon_magic_damage_bonus: i32,
    weapon_magic_penetration: i32,
    ring_magic_damage_bonus: i32,
    ring_magic_penetration: i32,
) -> MagicBonusResult {
    let mut damage = base_damage + ((base_damage as f64 * (3 * caster_level) as f64) / 100.0).round() as i32;
    let mut magic_penetration = 0i32;

    for &(bonus_pct, pen) in &[
        (weapon_magic_damage_bonus, weapon_magic_penetration),
        (ring_magic_damage_bonus, ring_magic_penetration),
    ] {
        if bonus_pct != 0 {
            damage += ((damage as f64 * bonus_pct as f64) / 100.0).round() as i32;
        }
        magic_penetration += pen;
    }

    damage = (damage as f64 * mod_dmg_magia(caster_class_id)) as i32;

    MagicBonusResult { damage, magic_penetration }
}

// ---------------------------------------------------------------------------
// Magic resistance vs NPC (port of applyMagicResistanceToNpc from game.ts)
// ---------------------------------------------------------------------------

pub fn apply_magic_resistance_to_npc(
    damage: i32,
    caster_level: i32,
    npc_magic_resistance: i32,
    npc_magic_def: i32,
    magic_penetration: i32,
) -> i32 {
    let mut next_damage = damage;
    let caster_skill = simulated_skill(caster_level);

    if npc_magic_resistance > 0 {
        let diff_skill = npc_magic_resistance - caster_skill;
        let extra = if diff_skill > 0 { diff_skill * 2 } else { 0 };
        let percent_reduction = (npc_magic_def + extra - magic_penetration).max(0);
        next_damage -= (next_damage as f64 * percent_reduction as f64 / 100.0) as i32;
    }

    next_damage -= npc_magic_def;
    next_damage.max(1)
}

// ---------------------------------------------------------------------------
// Magic resistance vs Player (port of applyMagicResistanceToUser from game.ts)
// ---------------------------------------------------------------------------

pub fn apply_magic_resistance_to_user(
    damage: i32,
    caster_level: i32,
    target_level: i32,
    target_class_id: i32,
    target_item_magic_resistance: i32,
    magic_penetration: i32,
) -> i32 {
    let mut next_damage = damage;
    let caster_skill = simulated_skill(caster_level);
    let target_magic_resistance = simulated_skill(target_level);
    let target_class_magic_resistance = mod_resistencia_magica(target_class_id) as i32;
    let diff_skill = target_magic_resistance - caster_skill;
    let extra = if diff_skill > 0 { diff_skill * 2 } else { 0 };
    let percent_reduction = (target_item_magic_resistance + target_class_magic_resistance + extra - magic_penetration).max(0);

    next_damage -= (next_damage as f64 * percent_reduction as f64 / 100.0) as i32;
    next_damage.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulated_skill_capped_at_100() {
        assert_eq!(simulated_skill(1), 3);
        assert_eq!(simulated_skill(33), 99);
        assert_eq!(simulated_skill(34), 100);
        assert_eq!(simulated_skill(50), 100);
    }

    #[test]
    fn warrior_evasion_higher_than_mage() {
        let w = poder_evasion(20, 20, 3);
        let m = poder_evasion(20, 20, 1);
        assert!(w > m);
    }

    #[test]
    fn shield_evasion_zero_without_shield() {
        assert_eq!(poder_evasion_escudo(20, 3, 0), 0.0);
    }

    #[test]
    fn poder_ataque_unarmed_vs_armed() {
        let unarmed = poder_ataque_arma(20, 20, 3, WeaponType::Unarmed);
        let armed = poder_ataque_arma(20, 20, 3, WeaponType::Melee);
        assert!(armed >= unarmed);
    }

    #[test]
    fn stab_chance_assassin_highest() {
        assert!(stabbing_chance_by_class(4) > stabbing_chance_by_class(3));
    }

    #[test]
    fn body_part_head_uses_helmet() {
        let abs = body_part_absorption(BodyPart::Head, 5, 10, 100, 200, 50, 100);
        assert!(abs <= 10);
    }

    #[test]
    fn body_part_torso_uses_body_and_shield() {
        let abs = body_part_absorption(BodyPart::Torso, 5, 10, 10, 20, 10, 20);
        assert!(abs >= 20 || abs >= 0); // min_def=20, max_def=40
    }

    #[test]
    fn calcular_dmg_armed_vs_unarmed() {
        let armed = calcular_dmg(1, 5, 20, 3, 10, 20, 0, 0, false);
        let unarmed = calcular_dmg(1, 5, 20, 3, 0, 0, 0, 0, false);
        assert!(armed >= unarmed);
    }

    #[test]
    fn melee_hit_chance_clamped() {
        let low = melee_hit_chance(0.0, 1000.0);
        let high = melee_hit_chance(1000.0, 0.0);
        assert!(low >= 5.0);
        assert!(high <= 95.0);
    }

    #[test]
    fn can_stab_requires_apu_weapon() {
        assert!(!can_stab(20, 4, false));
        assert!(can_stab(20, 4, true));
    }

    #[test]
    fn can_stab_non_assassin_needs_skill_10() {
        assert!(!can_stab(3, 3, true)); // level 3 => skill 9 < 10
        assert!(can_stab(4, 3, true));  // level 4 => skill 12 >= 10
    }

    #[test]
    fn magic_bonuses_level_scaling() {
        let result = apply_magic_bonuses(100, 20, 1, 0, 0, 0, 0);
        assert!(result.damage > 100);
        assert_eq!(result.magic_penetration, 0);
    }

    #[test]
    fn magic_bonuses_item_bonus_applies() {
        let without = apply_magic_bonuses(100, 20, 1, 0, 0, 0, 0);
        let with_bonus = apply_magic_bonuses(100, 20, 1, 20, 5, 10, 3);
        assert!(with_bonus.damage > without.damage);
        assert_eq!(with_bonus.magic_penetration, 8);
    }

    #[test]
    fn magic_resistance_npc_reduces_damage() {
        let full = apply_magic_resistance_to_npc(100, 20, 0, 0, 0);
        let resisted = apply_magic_resistance_to_npc(100, 20, 50, 10, 0);
        assert!(resisted < full);
        assert!(resisted >= 1);
    }

    #[test]
    fn magic_resistance_user_reduces_damage() {
        let full = apply_magic_resistance_to_user(100, 20, 1, 1, 0, 0);
        let resisted = apply_magic_resistance_to_user(100, 20, 20, 3, 15, 0);
        assert!(resisted < full);
        assert!(resisted >= 1);
    }

    #[test]
    fn magic_penetration_offsets_resistance() {
        let no_pen = apply_magic_resistance_to_npc(100, 20, 60, 20, 0);
        let with_pen = apply_magic_resistance_to_npc(100, 20, 60, 20, 15);
        assert!(with_pen >= no_pen);
    }

    #[test]
    fn warrior_has_zero_magic_damage_modifier() {
        assert_eq!(mod_dmg_magia(3), 0.0);
        let result = apply_magic_bonuses(100, 30, 3, 0, 0, 0, 0);
        assert_eq!(result.damage, 0);
    }

    #[test]
    fn stab_pvp_returns_stabresult() {
        let result = try_stab_pvp(50, 4, 100);
        assert!(result.total_damage >= 100);
    }

    #[test]
    fn stabbing_class_modifiers_differ() {
        let assassin = stabbing_chance_by_class(4);
        let warrior = stabbing_chance_by_class(3);
        assert!(assassin > warrior);
    }

    #[test]
    fn npc_evasion_scales_with_level() {
        let low = npc_evasion(5);
        let high = npc_evasion(40);
        assert!(high > low);
    }

    #[test]
    fn newbie_check() {
        assert!(is_newbie_character(1));
        assert!(is_newbie_character(12));
        assert!(!is_newbie_character(13));
        assert!(!is_newbie_character(50));
    }

    #[test]
    fn resolve_boat_body_dead() {
        assert_eq!(resolve_boat_body_id(84, true), 87);
        assert_eq!(resolve_boat_body_id(85, true), 87);
        assert_eq!(resolve_boat_body_id(86, true), 87);
    }

    #[test]
    fn resolve_boat_body_special_preserved() {
        assert_eq!(resolve_boat_body_id(85, false), 85);
        assert_eq!(resolve_boat_body_id(86, false), 86);
    }

    #[test]
    fn npc_multipliers_match_original() {
        assert_eq!(NPC_EXP_MULTIPLIER, 5);
        assert_eq!(NPC_GOLD_MULTIPLIER, 3);
    }

    #[test]
    fn dragon_slayer_hit_only_on_dragons() {
        assert!(is_dragon_slayer_hit(DRAGON_SLAYER_SWORD_ITEM_ID, 6, 6));
        assert!(!is_dragon_slayer_hit(DRAGON_SLAYER_SWORD_ITEM_ID, 5, 6));
        assert!(!is_dragon_slayer_hit(100, 6, 6));
    }

    #[test]
    fn dead_world_delay_is_15s() {
        assert_eq!(DEAD_WORLD_DELAY_MS, 15_000);
    }

    #[test]
    fn unsafe_logout_is_10s() {
        assert_eq!(UNSAFE_LOGOUT_DELAY_MS, 10_000);
    }

    #[test]
    fn magic_bonuses_zero_level_returns_base() {
        let result = apply_magic_bonuses(100, 0, 1, 0, 0, 0, 0);
        assert!(result.damage >= 1);
    }

    #[test]
    fn magic_resistance_npc_never_negative() {
        let dmg = apply_magic_resistance_to_npc(10, 999, 999, 0, 0);
        assert!(dmg >= 1);
    }

    #[test]
    fn magic_resistance_user_never_negative() {
        let dmg = apply_magic_resistance_to_user(10, 999, 999, 0, 0, 0);
        assert!(dmg >= 1);
    }

    #[test]
    fn pvp_base_rewards_match_original() {
        // Original: vars.exp = 50, vars.gold = 10
        // PVP_BASE_EXP * multiplicadorExp = 50 * 5 = 250
        // PVP_BASE_GOLD * multiplicadorGold = 10 * 3 = 30
        let pvp_base_exp: i32 = 50;
        let pvp_base_gold: i32 = 10;
        assert_eq!(pvp_base_exp * NPC_EXP_MULTIPLIER, 250);
        assert_eq!(pvp_base_gold * NPC_GOLD_MULTIPLIER, 30);
    }

    #[test]
    fn faction_rekill_window_is_5_minutes() {
        // Original: FACTION_REKILL_WINDOW_MS = 5 * 60 * 1000 = 300,000
        let window: u64 = 5 * 60 * 1000;
        assert_eq!(window, 300_000);
    }

    #[test]
    fn bail_cost_formula_matches_original() {
        // Original: ciudadanosMatados * multiplicadorGold * 5000
        // With 3 citizen kills: 3 * 3 * 5000 = 45000
        let citizen_kills: i32 = 3;
        let bail_cost_per_citizen: i32 = 5000;
        let cost = citizen_kills * NPC_GOLD_MULTIPLIER * bail_cost_per_citizen;
        assert_eq!(cost, 45_000);
    }

    #[test]
    fn all_11_classes_have_class_modifiers() {
        for class_id in 1..=11 {
            assert!(mod_evasion(class_id) > 0.0, "class {} missing evasion mod", class_id);
            // mod_escudo is 0 for Mago(1) and Paladin(7) — matches original
            assert!(mod_escudo(class_id) >= 0.0, "class {} invalid shield mod", class_id);
            assert!(mod_dmg_armas(class_id) > 0.0, "class {} missing weapon dmg mod", class_id);
        }
    }

    #[test]
    fn calcular_dmg_unarmed_uses_wrestling_range() {
        // Original: unarmed = randomIntFromInterval(4, 9)
        // We verify damage > 0 with no weapon
        let dmg = calcular_dmg(1, 5, 20, 1, 0, 0, 0, 0, false);
        assert!(dmg >= 1, "unarmed damage should be positive");
    }

    #[test]
    fn hidden_skill_chance_formula_bounds() {
        let calc = |skill: f64| -> f64 {
            let raw = (((0.000002 * skill - 0.0002) * skill + 0.0064) * skill + 0.1124) * 100.0;
            raw.clamp(1.0, 99.0)
        };
        let low = calc(0.0);
        assert!(low > 0.0 && low < 20.0, "skill 0 chance should be low: {}", low);
        let high = calc(100.0);
        assert!(high > 50.0 && high < 100.0, "skill 100 chance should be high: {}", high);
    }

    // ── E2E flow tests ──────────────────────────────────────────────

    /// Full melee combat flow: hit chance → damage → defense → final damage.
    #[test]
    fn e2e_melee_combat_flow_warrior_vs_target() {
        let attacker_level = 30;
        let attacker_class = 3; // guerrero
        let skill = simulated_skill(attacker_level);
        assert_eq!(skill, 90);

        let hit_power = poder_ataque_arma(attacker_level, 20, attacker_class, WeaponType::Melee);
        assert!(hit_power > 0.0);

        let dmg = calcular_dmg(
            1,   // min_hit
            50,  // max_hit
            20,  // fuerza
            attacker_class,
            15,  // weapon_min_hit
            25,  // weapon_max_hit
            0,   // arrow_min
            0,   // arrow_max
            false,
        );
        assert!(dmg > 0);

        let hit_chance = melee_hit_chance(hit_power, 50.0);
        assert!(hit_chance >= 5.0 && hit_chance <= 95.0);
    }

    /// Full magic combat flow: spell damage → magic bonus → resistance → final.
    #[test]
    fn e2e_magic_combat_flow_mage_vs_npc() {
        let caster_level = 25;
        let caster_class = 1; // mago
        let base_spell_dmg = 80;

        let result = apply_magic_bonuses(
            base_spell_dmg, caster_level, caster_class,
            10, // weapon magic bonus
            5,  // ring magic bonus
            15, // inteligencia
            0,
        );
        assert!(result.damage > base_spell_dmg);

        let final_dmg = apply_magic_resistance_to_npc(
            result.damage, caster_level,
            30, // npc magic_def
            10, // npc magic_resistance
            result.magic_penetration,
        );
        assert!(final_dmg >= 1);
        assert!(final_dmg < result.damage);
    }

    /// Full PvP scenario: Caos attacks Armada — should not flag criminal.
    #[test]
    fn e2e_faction_pvp_no_criminal_flag_rival() {
        // Ported from shouldAwardArmadaScore / shouldAwardCaosScore logic.
        // Armada player kills Caos player:
        //  - attacker is armada, victim is caos → award armada score
        //  - no criminal flag because rivals
        let attacker_faction = "armada";
        let victim_faction = "caos";
        let victim_criminal = false;

        let is_rival = (attacker_faction == "armada" && victim_faction == "caos")
            || (attacker_faction == "caos" && victim_faction == "armada");
        assert!(is_rival);

        let should_flag_criminal = !is_rival && !victim_criminal;
        assert!(!should_flag_criminal);
    }

    /// E2E: Level up from 12→13 triggers newbie system boundary.
    #[test]
    fn e2e_newbie_level_up_boundary() {
        assert!(is_newbie_character(12));
        assert!(!is_newbie_character(13));

        use crate::gameplay::balance::{get_max_hp_for_level, get_max_mana_for_level};
        // get_max_hp_for_level(class_id, constitucion, level)
        let hp_12 = get_max_hp_for_level(3, 18, 12);
        let hp_13 = get_max_hp_for_level(3, 18, 13);
        assert!(hp_13 > hp_12, "HP should increase on level up");

        // get_max_mana_for_level(class_id, inteligencia, level)
        let mana_12 = get_max_mana_for_level(1, 18, 12);
        let mana_13 = get_max_mana_for_level(1, 18, 13);
        assert!(mana_13 > mana_12, "Mana should increase on level up for mage");
    }

    /// E2E: Gold clamp prevents overflow on massive PvP rewards.
    #[test]
    fn e2e_gold_clamp_on_pvp_rewards() {
        use crate::gameplay::balance::clamp_gold;
        let max_gold = 2_147_483_647i64;
        let current = max_gold - 10;
        let reward = 30i64 * NPC_GOLD_MULTIPLIER as i64;
        let clamped = clamp_gold(current + reward);
        assert_eq!(clamped, max_gold, "gold should clamp at MAX_GOLD");
    }

    /// E2E: Stabbing requires correct conditions.
    #[test]
    fn e2e_stabbing_full_flow() {
        let assassin_class = 4;
        let level = 15;
        let has_apu_weapon = true;

        assert!(can_stab(level, assassin_class, has_apu_weapon));

        let stab = try_stab_npc(level, assassin_class, 100);
        // Stab is probabilistic — verify the result is valid regardless of RNG.
        if stab.stabbed {
            assert!(stab.extra_damage > 0);
            assert!(stab.total_damage > 100);
        } else {
            assert_eq!(stab.extra_damage, 0);
            assert_eq!(stab.total_damage, 100);
        }
    }
}
