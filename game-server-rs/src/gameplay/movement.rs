use openao_protocol::constants::{MAP_MAX_COORDINATE, MAP_MIN_COORDINATE};

use crate::world::{GameWorld, Position};

/// Direction offsets for headings 1-4 (up, down, left, right).
const HEADING_DX: [i32; 5] = [0, 0, 0, -1, 1];
const HEADING_DY: [i32; 5] = [0, -1, 1, 0, 0];

/// Validates that coordinates are within map bounds.
pub fn is_valid_position(x: i32, y: i32) -> bool {
    (MAP_MIN_COORDINATE..=MAP_MAX_COORDINATE).contains(&x)
        && (MAP_MIN_COORDINATE..=MAP_MAX_COORDINATE).contains(&y)
}

/// Calculates the next position given a heading (1=up, 2=down, 3=left, 4=right).
pub fn next_position(pos: &Position, heading: u8) -> Option<Position> {
    let h = heading as usize;
    if h == 0 || h > 4 {
        return None;
    }

    let new_x = pos.x + HEADING_DX[h];
    let new_y = pos.y + HEADING_DY[h];

    if is_valid_position(new_x, new_y) {
        Some(Position {
            map: pos.map,
            x: new_x,
            y: new_y,
        })
    } else {
        None
    }
}

/// Processes a movement request for a player.
/// Returns true if the move was successful.
pub fn process_move(
    world: &GameWorld,
    player_id: u32,
    heading: u8,
) -> bool {
    let scene_ref = world.scenes.iter().find(|entry| {
        entry.value().players.contains_key(&player_id)
    });

    let Some(scene_entry) = scene_ref else {
        return false;
    };

    let scene = scene_entry.value();

    let current_pos = {
        let player = scene.players.get(&player_id);
        match player {
            Some(p) => p.pos.clone(),
            None => return false,
        }
    };

    let Some(new_pos) = next_position(&current_pos, heading) else {
        return false;
    };

    {
        let blocked = scene.blocked.read().unwrap();
        if blocked[new_pos.x as usize][new_pos.y as usize] {
            return false;
        }
    }

    if let Some(mut player) = scene.players.get_mut(&player_id) {
        player.pos = new_pos;
        player.heading = heading;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_positions() {
        assert!(is_valid_position(1, 1));
        assert!(is_valid_position(50, 50));
        assert!(is_valid_position(100, 100));
        assert!(!is_valid_position(0, 0));
        assert!(!is_valid_position(101, 50));
    }

    #[test]
    fn next_position_headings() {
        let pos = Position { map: 1, x: 50, y: 50 };

        let up = next_position(&pos, 1).unwrap();
        assert_eq!((up.x, up.y), (50, 49));

        let down = next_position(&pos, 2).unwrap();
        assert_eq!((down.x, down.y), (50, 51));

        let left = next_position(&pos, 3).unwrap();
        assert_eq!((left.x, left.y), (49, 50));

        let right = next_position(&pos, 4).unwrap();
        assert_eq!((right.x, right.y), (51, 50));
    }

    #[test]
    fn next_position_invalid_heading() {
        let pos = Position { map: 1, x: 50, y: 50 };
        assert!(next_position(&pos, 0).is_none());
        assert!(next_position(&pos, 5).is_none());
    }

    #[test]
    fn next_position_boundary() {
        let corner = Position { map: 1, x: 1, y: 1 };
        assert!(next_position(&corner, 1).is_none()); // y=0 invalid
        assert!(next_position(&corner, 3).is_none()); // x=0 invalid
        assert!(next_position(&corner, 2).is_some());
        assert!(next_position(&corner, 4).is_some());
    }
}
