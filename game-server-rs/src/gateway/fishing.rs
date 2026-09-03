use crate::error::HandlerResult;
use crate::gateway::packets::build_console_message;
use crate::replication::build_play_sound;
use crate::world::FishingState;

use super::GameSession;
use super::WsSink;

const FISHING_TICK_MS: u64 = 4000;
const FISHING_SUCCESS_CHANCE: i32 = 80;
const SND_PESCAR: u16 = 46;

struct FishingRod {
    item_id: i32,
    #[allow(dead_code)]
    power: i32,
}

const FISHING_RODS: &[FishingRod] = &[
    FishingRod { item_id: 138, power: 1 },
    FishingRod { item_id: 563, power: 1 },
];

struct FishReward {
    item_id: i32,
    weight: u32,
}

const FISH_REWARDS: &[FishReward] = &[
    FishReward { item_id: 139, weight: 24326 },
    FishReward { item_id: 544, weight: 256 },
    FishReward { item_id: 545, weight: 6 },
];

pub fn is_fishing_rod(item_id: i32) -> bool {
    FISHING_RODS.iter().any(|r| r.item_id == item_id)
}

fn get_rod_power(item_id: i32) -> i32 {
    FISHING_RODS.iter().find(|r| r.item_id == item_id).map(|r| r.power).unwrap_or(1)
}

fn pick_weighted_reward() -> Option<i32> {
    let total: u32 = FISH_REWARDS.iter().map(|r| r.weight).sum();
    if total == 0 {
        return None;
    }
    let roll = rand::random_range(0..total);
    let mut cumulative = 0u32;
    for reward in FISH_REWARDS {
        cumulative += reward.weight;
        if roll < cumulative {
            return Some(reward.item_id);
        }
    }
    None
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

impl GameSession {
    pub(super) async fn handle_fishing_rod_use(
        &self,
        entity_id: u32,
        slot: u8,
        item_id: i32,
        sink: &mut WsSink,
    ) -> Result<bool, crate::error::HandlerError> {
        if !is_fishing_rod(item_id) {
            return Ok(false);
        }

        let map_id = self.map_id.unwrap_or(0);
        let scene = self.world.get_or_create_scene(map_id);
        let Some(player) = scene.players.get(&entity_id) else { return Ok(false); };

        let char_id = player.character_id.clone();
        let inv = self.world.cache_get_inventory(&char_id);
        let equipped = inv.iter().find(|i| i.item_id == item_id && i.equipped);
        if equipped.is_none() {
            self.send_to_client(sink, build_console_message("Debes equiparte la caña de pescar para usarla.")).await?;
            return Ok(true);
        }

        if let Some(ref fishing) = player.fishing
            && fishing.active {
                let ip = player.client_ip.clone();
                drop(player);
                if let Some(mut p) = scene.players.get_mut(&entity_id) {
                    p.fishing = None;
                }
                self.world.release_working_lock(&ip, entity_id);
                self.send_to_client(sink, build_console_message("Has dejado de pescar.")).await?;
                return Ok(true);
            }

        let fishing = FishingState {
            active: false,
            pending_target: true,
            slot,
            item_id,
            power: get_rod_power(item_id),
            target_x: 0,
            target_y: 0,
            origin_x: player.pos.x,
            origin_y: player.pos.y,
            next_tick_at_ms: 0,
        };
        drop(player);
        if let Some(mut p) = scene.players.get_mut(&entity_id) {
            p.fishing = Some(fishing);
        }

        self.send_to_client(sink, build_console_message("Selecciona una casilla de agua cercana para pescar.")).await?;
        Ok(true)
    }

    pub(super) async fn handle_fishing_map_click(
        &self,
        entity_id: u32,
        click_x: i32,
        click_y: i32,
        sink: &mut WsSink,
    ) -> Result<bool, crate::error::HandlerError> {
        let map_id = self.map_id.unwrap_or(0);
        let scene = self.world.get_or_create_scene(map_id);
        let Some(player) = scene.players.get(&entity_id) else { return Ok(false); };

        let has_pending = player.fishing.as_ref().map(|f| f.pending_target).unwrap_or(false);
        if !has_pending {
            return Ok(false);
        }

        let map_id = player.pos.map;
        let px = player.pos.x;
        let py = player.pos.y;
        let fishing_ref = match player.fishing.as_ref() {
            Some(f) => f,
            None => return Ok(false),
        };
        let fish_item_id = fishing_ref.item_id;
        let fish_slot = fishing_ref.slot;
        let fish_power = fishing_ref.power;
        drop(player);

        if (px - click_x).abs() > 1 || (py - click_y).abs() > 1 {
            self.send_to_client(sink, build_console_message("Debes seleccionar una casilla de agua cercana para pescar.")).await?;
            return Ok(true);
        }

        if !self.world.gd().is_water_tile(map_id, click_x, click_y) {
            self.send_to_client(sink, build_console_message("Zona de pesca no autorizada. Busca otro lugar para hacerlo.")).await?;
            return Ok(true);
        }

        let player_on_water = self.world.gd().is_water_tile(map_id, px, py);
        if player_on_water || !self.world.gd().is_adjacent_to_water(map_id, px, py) {
            self.send_to_client(sink, build_console_message("Acércate a la costa para pescar.")).await?;
            return Ok(true);
        }

        let client_ip = scene.players.get(&entity_id).map(|p| p.client_ip.clone()).unwrap_or_default();
        if self.world.has_active_gathering_on_ip(&client_ip, entity_id) {
            self.send_to_client(sink, build_console_message("Ya tienes otro personaje trabajando desde esta conexión.")).await?;
            return Ok(true);
        }

        let now = now_ms();
        let fishing = FishingState {
            active: true,
            pending_target: false,
            slot: fish_slot,
            item_id: fish_item_id,
            power: fish_power,
            target_x: click_x,
            target_y: click_y,
            origin_x: px,
            origin_y: py,
            next_tick_at_ms: now + FISHING_TICK_MS,
        };
        if let Some(mut p) = scene.players.get_mut(&entity_id) {
            p.fishing = Some(fishing);
        }

        self.world.acquire_working_lock(&client_ip, entity_id);

        let sound_pkt = build_play_sound(SND_PESCAR);
        let fish_pos = scene.players.get(&entity_id).map(|p| p.pos.clone());
        if let Some(ref pos) = fish_pos {
            scene.broadcast_in_range(0, pos, sound_pkt);
        } else {
            scene.broadcast(0, sound_pkt);
        }

        self.send_to_client(sink, build_console_message("Comienzas a pescar.")).await?;
        Ok(true)
    }

    /// Called from the main packet loop to advance fishing if active.
    pub(super) async fn tick_fishing(
        &self,
        entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let now = now_ms();
        let map_id = self.map_id.unwrap_or(0);
        let scene = self.world.get_or_create_scene(map_id);
        let Some(player) = scene.players.get(&entity_id) else { return Ok(()); };

        let should_tick = player.fishing.as_ref()
            .map(|f| f.active && f.next_tick_at_ms <= now)
            .unwrap_or(false);

        if !should_tick {
            return Ok(());
        }

        let dead = player.dead;
        let px = player.pos.x;
        let py = player.pos.y;
        let fishing_ref2 = match player.fishing.as_ref() {
            Some(f) => f,
            None => return Ok(()),
        };
        let origin_x = fishing_ref2.origin_x;
        let origin_y = fishing_ref2.origin_y;
        let char_id = player.character_id.clone();
        drop(player);

        if dead || px != origin_x || py != origin_y {
            cancel_fishing(entity_id, &scene, sink, self, Some("La pesca se canceló.")).await?;
            return Ok(());
        }

        if let Some(mut p) = scene.players.get_mut(&entity_id)
            && let Some(ref mut f) = p.fishing {
                f.next_tick_at_ms = now + FISHING_TICK_MS;
            }

        let roll = rand::random_range(1..=100);
        if roll > FISHING_SUCCESS_CHANCE {
            return Ok(());
        }

        let Some(reward_item_id) = pick_weighted_reward() else { return Ok(()); };
        let reward_name = self.world.gd().get_object(reward_item_id)
            .map(|o| o.name.clone())
            .unwrap_or_else(|| format!("Item {}", reward_item_id));

        let added = self.world.cache_add_item(&char_id, reward_item_id, 1);
        if !added {
            cancel_fishing(entity_id, &scene, sink, self, Some("La pesca se detuvo porque no tienes espacio en el inventario.")).await?;
            return Ok(());
        }

        self.send_full_inventory(sink).await?;

        let msg = format!("Has pescado 1 {}.", reward_name);
        self.send_to_client(sink, build_console_message(&msg)).await?;

        let sound_pkt = build_play_sound(SND_PESCAR);
        let fish_pos2 = scene.players.get(&entity_id).map(|p| p.pos.clone());
        if let Some(ref pos) = fish_pos2 {
            scene.broadcast_in_range(0, pos, sound_pkt);
        } else {
            scene.broadcast(0, sound_pkt);
        }

        Ok(())
    }
}

async fn cancel_fishing(
    entity_id: u32,
    scene: &crate::world::Scene,
    sink: &mut WsSink,
    session: &GameSession,
    reason: Option<&str>,
) -> HandlerResult {
    if let Some(mut p) = scene.players.get_mut(&entity_id) {
        let ip = p.client_ip.clone();
        p.fishing = None;
        drop(p);
        session.world.release_working_lock(&ip, entity_id);
    }
    if let Some(msg) = reason {
        session.send_to_client(sink, build_console_message(msg)).await?;
    }
    Ok(())
}

/// Cancel fishing when player moves.
pub fn cancel_fishing_on_move(entity_id: u32, scene: &crate::world::Scene, world: &crate::world::GameWorld) {
    if let Some(mut p) = scene.players.get_mut(&entity_id) {
        let was_active = p.fishing.as_ref().map(|f| f.active).unwrap_or(false);
        let ip = p.client_ip.clone();
        if was_active {
            p.fishing = None;
            drop(p);
            world.release_working_lock(&ip, entity_id);
            if let Some(tx) = scene.personal_tx.get(&entity_id) {
                let _ = tx.send(build_console_message("La pesca se canceló."));
            }
        } else if p.fishing.is_some() {
            p.fishing = None;
        }
    }
}
