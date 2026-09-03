use crate::world::{GameWorld, PlayerState, NpcState, Position};

/// Calculates melee damage from attacker to target.
/// Ported from game.ts `atacar` / melee combat logic.
pub fn calculate_melee_damage(
    attacker_min_hit: i32,
    attacker_max_hit: i32,
    target_defense: i32,
) -> i32 {
    if attacker_max_hit <= attacker_min_hit {
        return 0;
    }
    let raw = rand::random_range(attacker_min_hit..=attacker_max_hit);
    (raw - target_defense).max(0)
}

/// Checks if two positions are adjacent (within 1 tile).
pub fn is_melee_range(a: &Position, b: &Position) -> bool {
    a.map == b.map && (a.x - b.x).abs() <= 1 && (a.y - b.y).abs() <= 1
}

/// Processes a melee attack from one player to another.
/// Returns the damage dealt (0 if missed or out of range).
pub fn process_player_melee(
    _world: &GameWorld,
    _attacker: &PlayerState,
    _target_id: u32,
) -> i32 {
    // TODO: Full implementation from game.ts
    // - Check safe zone
    // - Check attacker is alive and not paralyzed
    // - Check target exists and is in range
    // - Calculate hit chance
    // - Calculate damage
    // - Apply damage
    // - Send packets to both players and observers
    0
}

/// Processes a melee attack from a player to an NPC.
pub fn process_npc_melee(
    _world: &GameWorld,
    _attacker: &PlayerState,
    _target_npc: &NpcState,
) -> i32 {
    // TODO: Full implementation from game.ts
    0
}
