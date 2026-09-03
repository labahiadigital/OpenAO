use crate::error::HandlerResult;
use crate::gateway::packets::build_console_message;
use crate::replication::build_play_sound;
use crate::world::{HarvestingSkill, HarvestingState};
use crate::game_data::GameData;

use super::GameSession;
use super::WsSink;

const HARVESTING_TICK_MS: u64 = 4000;
const HARVESTING_SUCCESS_CHANCE: i32 = 75;
const SND_TALAR: u16 = 13;
const SND_MINAR: u16 = 17;

const WOOD_ITEM_ID: i32 = 58;
const ELVEN_WOOD_ITEM_ID: i32 = 1006;
const IRON_ORE_ITEM_ID: i32 = 192;
const SILVER_ORE_ITEM_ID: i32 = 193;
const GOLD_ORE_ITEM_ID: i32 = 194;

pub fn is_harvesting_tool(item_id: i32, game_data: &GameData) -> bool {
    let Some(obj) = game_data.get_object(item_id) else { return false; };
    let name = obj.name.to_lowercase();
    is_woodcutting_name(&name) || is_mining_name(&name)
}

fn is_woodcutting_name(name: &str) -> bool {
    name.contains("hacha de leñador") || name.contains("hacha de leña")
}

fn is_mining_name(name: &str) -> bool {
    name.contains("pico de miner")
}

fn get_skill_for_item(item_id: i32, game_data: &GameData) -> Option<HarvestingSkill> {
    let obj = game_data.get_object(item_id)?;
    let name = obj.name.to_lowercase();
    if is_woodcutting_name(&name) {
        Some(HarvestingSkill::Woodcutting)
    } else if is_mining_name(&name) {
        Some(HarvestingSkill::Mining)
    } else {
        None
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn pick_woodcutting_reward(is_elven_tool: bool) -> i32 {
    if is_elven_tool {
        let roll = rand::random_range(0..100);
        if roll < 15 { ELVEN_WOOD_ITEM_ID } else { WOOD_ITEM_ID }
    } else {
        WOOD_ITEM_ID
    }
}

fn pick_mining_reward() -> i32 {
    let roll = rand::random_range(0..100);
    if roll < 5 {
        GOLD_ORE_ITEM_ID
    } else if roll < 20 {
        SILVER_ORE_ITEM_ID
    } else {
        IRON_ORE_ITEM_ID
    }
}

impl GameSession {
    pub(super) async fn handle_harvesting_tool_use(
        &self,
        entity_id: u32,
        slot: u8,
        item_id: i32,
        sink: &mut WsSink,
    ) -> Result<bool, crate::error::HandlerError> {
        let skill = match get_skill_for_item(item_id, &self.world.gd()) {
            Some(s) => s,
            None => return Ok(false),
        };

        let map_id = self.map_id.unwrap_or(0);
        let scene = self.world.get_or_create_scene(map_id);
        let Some(player) = scene.players.get(&entity_id) else { return Ok(false); };

        let char_id = player.character_id.clone();
        let inv = self.world.cache_get_inventory(&char_id);
        let equipped = inv.iter().find(|i| i.item_id == item_id && i.equipped);
        if equipped.is_none() {
            let msg = match skill {
                HarvestingSkill::Woodcutting => "Debes equiparte el hacha para usarla.",
                HarvestingSkill::Mining => "Debes equiparte el pico para usarlo.",
            };
            self.send_to_client(sink, build_console_message(msg)).await?;
            return Ok(true);
        }

        if let Some(ref h) = player.harvesting
            && h.active {
                let ip = player.client_ip.clone();
                drop(player);
                if let Some(mut p) = scene.players.get_mut(&entity_id) {
                    p.harvesting = None;
                }
                self.world.release_working_lock(&ip, entity_id);
                let msg = match skill {
                    HarvestingSkill::Woodcutting => "Has dejado de talar.",
                    HarvestingSkill::Mining => "Has dejado de minar.",
                };
                self.send_to_client(sink, build_console_message(msg)).await?;
                return Ok(true);
            }

        let harvesting = HarvestingState {
            active: false,
            pending_target: true,
            skill,
            slot,
            item_id,
            target_x: 0,
            target_y: 0,
            origin_x: player.pos.x,
            origin_y: player.pos.y,
            next_tick_at_ms: 0,
        };
        drop(player);
        if let Some(mut p) = scene.players.get_mut(&entity_id) {
            p.harvesting = Some(harvesting);
        }

        let msg = match skill {
            HarvestingSkill::Woodcutting => "Selecciona un árbol cercano para talar.",
            HarvestingSkill::Mining => "Selecciona un yacimiento cercano para minar.",
        };
        self.send_to_client(sink, build_console_message(msg)).await?;
        Ok(true)
    }

    pub(super) async fn handle_harvesting_map_click(
        &self,
        entity_id: u32,
        click_x: i32,
        click_y: i32,
        sink: &mut WsSink,
    ) -> Result<bool, crate::error::HandlerError> {
        let map_id = self.map_id.unwrap_or(0);
        let scene = self.world.get_or_create_scene(map_id);
        let Some(player) = scene.players.get(&entity_id) else { return Ok(false); };

        let has_pending = player.harvesting.as_ref().map(|h| h.pending_target).unwrap_or(false);
        if !has_pending {
            return Ok(false);
        }

        let px = player.pos.x;
        let py = player.pos.y;
        let harv_ref = match player.harvesting.as_ref() {
            Some(h) => h,
            None => return Ok(false),
        };
        let h_item_id = harv_ref.item_id;
        let h_slot = harv_ref.slot;
        let h_skill = harv_ref.skill;
        drop(player);

        if (px - click_x).abs() > 2 || (py - click_y).abs() > 2 {
            let msg = match h_skill {
                HarvestingSkill::Woodcutting => "Debes hacer click sobre un árbol cercano.",
                HarvestingSkill::Mining => "Debes hacer click sobre un yacimiento cercano.",
            };
            self.send_to_client(sink, build_console_message(msg)).await?;
            return Ok(true);
        }

        let client_ip = scene.players.get(&entity_id).map(|p| p.client_ip.clone()).unwrap_or_default();
        if self.world.has_active_gathering_on_ip(&client_ip, entity_id) {
            self.send_to_client(sink, build_console_message("Ya tienes otro personaje trabajando desde esta conexión.")).await?;
            return Ok(true);
        }

        let now = now_ms();
        let harvesting = HarvestingState {
            active: true,
            pending_target: false,
            skill: h_skill,
            slot: h_slot,
            item_id: h_item_id,
            target_x: click_x,
            target_y: click_y,
            origin_x: px,
            origin_y: py,
            next_tick_at_ms: now + HARVESTING_TICK_MS,
        };
        if let Some(mut p) = scene.players.get_mut(&entity_id) {
            p.harvesting = Some(harvesting);
        }

        self.world.acquire_working_lock(&client_ip, entity_id);

        let sound = match h_skill {
            HarvestingSkill::Woodcutting => SND_TALAR,
            HarvestingSkill::Mining => SND_MINAR,
        };
        let harvest_pos = scene.players.get(&entity_id).map(|p| p.pos.clone());
        if let Some(ref pos) = harvest_pos {
            scene.broadcast_in_range(0, pos, build_play_sound(sound));
        } else {
            scene.broadcast(0, build_play_sound(sound));
        }

        let msg = match h_skill {
            HarvestingSkill::Woodcutting => "Comienzas a talar.",
            HarvestingSkill::Mining => "Comienzas a minar.",
        };
        self.send_to_client(sink, build_console_message(msg)).await?;
        Ok(true)
    }

    pub(super) async fn tick_harvesting(
        &self,
        entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let now = now_ms();
        let map_id = self.map_id.unwrap_or(0);
        let scene = self.world.get_or_create_scene(map_id);
        let Some(player) = scene.players.get(&entity_id) else { return Ok(()); };

        let should_tick = player.harvesting.as_ref()
            .map(|h| h.active && h.next_tick_at_ms <= now)
            .unwrap_or(false);

        if !should_tick {
            return Ok(());
        }

        let dead = player.dead;
        let px = player.pos.x;
        let py = player.pos.y;
        let harv_ref2 = match player.harvesting.as_ref() {
            Some(h) => h,
            None => return Ok(()),
        };
        let origin_x = harv_ref2.origin_x;
        let origin_y = harv_ref2.origin_y;
        let skill = harv_ref2.skill;
        let h_item_id = harv_ref2.item_id;
        let char_id = player.character_id.clone();
        drop(player);

        if dead || px != origin_x || py != origin_y {
            cancel_harvesting(entity_id, &scene, sink, self, Some("La recolección se canceló.")).await?;
            return Ok(());
        }

        if let Some(mut p) = scene.players.get_mut(&entity_id)
            && let Some(ref mut h) = p.harvesting {
                h.next_tick_at_ms = now + HARVESTING_TICK_MS;
            }

        let roll = rand::random_range(1..=100);
        if roll > HARVESTING_SUCCESS_CHANCE {
            return Ok(());
        }

        let is_elven_tool = self.world.gd().get_object(h_item_id)
            .map(|o| o.name.to_lowercase().contains("elfic"))
            .unwrap_or(false);

        let reward_item_id = match skill {
            HarvestingSkill::Woodcutting => pick_woodcutting_reward(is_elven_tool),
            HarvestingSkill::Mining => pick_mining_reward(),
        };
        let reward_name = self.world.gd().get_object(reward_item_id)
            .map(|o| o.name.clone())
            .unwrap_or_else(|| format!("Item {}", reward_item_id));

        let added = self.world.cache_add_item(&char_id, reward_item_id, 1);
        if !added {
            cancel_harvesting(entity_id, &scene, sink, self, Some("La recolección se detuvo porque no tienes espacio.")).await?;
            return Ok(());
        }

        self.send_full_inventory(sink).await?;

        let msg = format!("Has obtenido 1 {}.", reward_name);
        self.send_to_client(sink, build_console_message(&msg)).await?;

        let sound = match skill {
            HarvestingSkill::Woodcutting => SND_TALAR,
            HarvestingSkill::Mining => SND_MINAR,
        };
        let h_pos = scene.players.get(&entity_id).map(|p| p.pos.clone());
        if let Some(ref pos) = h_pos {
            scene.broadcast_in_range(0, pos, build_play_sound(sound));
        } else {
            scene.broadcast(0, build_play_sound(sound));
        }

        Ok(())
    }
}

async fn cancel_harvesting(
    entity_id: u32,
    scene: &crate::world::Scene,
    sink: &mut WsSink,
    session: &GameSession,
    reason: Option<&str>,
) -> HandlerResult {
    if let Some(mut p) = scene.players.get_mut(&entity_id) {
        let ip = p.client_ip.clone();
        p.harvesting = None;
        drop(p);
        session.world.release_working_lock(&ip, entity_id);
    }
    if let Some(msg) = reason {
        session.send_to_client(sink, build_console_message(msg)).await?;
    }
    Ok(())
}

pub fn cancel_harvesting_on_move(entity_id: u32, scene: &crate::world::Scene, world: &crate::world::GameWorld) {
    if let Some(mut p) = scene.players.get_mut(&entity_id) {
        let was_active = p.harvesting.as_ref().map(|h| h.active).unwrap_or(false);
        let ip = p.client_ip.clone();
        if was_active {
            p.harvesting = None;
            drop(p);
            world.release_working_lock(&ip, entity_id);
            if let Some(tx) = scene.personal_tx.get(&entity_id) {
                let _ = tx.send(build_console_message("La recolección se canceló."));
            }
        } else if p.harvesting.is_some() {
            p.harvesting = None;
        }
    }
}
