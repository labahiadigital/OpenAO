use openao_protocol::opcodes::client_packet_id;
use openao_protocol::PacketWriter;
use smallvec::SmallVec;

use crate::game_data::GameData;
use crate::persistence::InventoryRow;
use crate::world::{NpcState, PlayerState};

/// Frontend readGetMyCharacter: id=getShort, map=getShort, x=getShort, y=getShort,
/// heading=getByte, name=getString, hp=getShort, maxHp=getShort, dead=getByte, level=getShort
pub fn build_my_character_packet(player: &PlayerState) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id(client_packet_id::GET_MY_CHARACTER);
    w.write_short(player.id as u16);
    w.write_short(player.pos.map as u16);
    w.write_short(player.pos.x as u16);
    w.write_short(player.pos.y as u16);
    w.write_byte(player.heading);
    w.write_string(&player.name);
    w.write_short(player.hp as u16);
    w.write_short(player.max_hp as u16);
    w.write_byte(if player.dead { 1 } else { 0 });
    w.write_short(player.level as u16);
    w.into_bytes()
}

/// Frontend readGetCharacter: id=getShort, x=getShort, y=getShort, heading=getByte,
/// name=getString, hp=getShort, maxHp=getShort, dead=getByte, level=getShort
pub fn build_character_packet(player: &PlayerState) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id(client_packet_id::GET_CHARACTER);
    w.write_short(player.id as u16);
    w.write_short(player.pos.x as u16);
    w.write_short(player.pos.y as u16);
    w.write_byte(player.heading);
    w.write_string(&player.name);
    w.write_short(player.hp as u16);
    w.write_short(player.max_hp as u16);
    w.write_byte(if player.dead { 1 } else { 0 });
    w.write_short(player.level as u16);
    w.into_bytes()
}

/// Frontend readGetNpc: id=getShort, x=getShort, y=getShort, heading=getByte,
/// npcType=getShort, hp=getShort, maxHp=getShort
pub fn build_npc_packet(npc: &NpcState, _game_data: &GameData) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id(client_packet_id::GET_NPC);
    w.write_short(npc.id as u16);
    w.write_short(npc.pos.x as u16);
    w.write_short(npc.pos.y as u16);
    w.write_byte(npc.heading);
    w.write_short(npc.npc_type as u16);
    w.write_short(npc.hp as u16);
    w.write_short(npc.max_hp as u16);
    w.into_bytes()
}

/// Frontend readMoveEntity: id=getShort, x=getShort, y=getShort, heading=getByte, serverTick=getShort
#[allow(dead_code)]
pub fn build_move_entity_packet(entity_id: u32, x: i32, y: i32, heading: u8) -> Vec<u8> {
    build_move_entity_packet_with_tick(entity_id, x, y, heading, 0)
}

/// Build MOVE_ENTITY with an explicit server tick for interpolation.
pub fn build_move_entity_packet_with_tick(entity_id: u32, x: i32, y: i32, heading: u8, server_tick: u16) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id_and_capacity(client_packet_id::MOVE_ENTITY, 10);
    w.write_short(entity_id as u16);
    w.write_short(x as u16);
    w.write_short(y as u16);
    w.write_byte(heading);
    w.write_short(server_tick);
    w.into_bytes()
}

/// Frontend readAddInvItem: slot=getByte, idItem=getShort, name=getString, amount=getShort,
/// equipped=getByte, grhIndex=getShort, objType=getByte, maxHit=getShort, minHit=getShort,
/// maxDef=getShort, minDef=getShort, value=getInt
pub fn build_inv_item_packet(row: &InventoryRow, item_data: &ItemData) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id(client_packet_id::AGREGAR_USER_INV_ITEM);
    w.write_byte(row.slot as u8);
    w.write_short(row.item_id as u16);
    w.write_string(&item_data.name);
    w.write_short(row.amount as u16);
    w.write_byte(if row.equipped { 1 } else { 0 });
    w.write_short(item_data.grh_index);
    w.write_byte(item_data.obj_type);
    w.write_short(item_data.max_hit);
    w.write_short(item_data.min_hit);
    w.write_short(item_data.max_def);
    w.write_short(item_data.min_def);
    w.write_int(item_data.value);
    w.into_bytes()
}

pub struct ItemData {
    pub name: String,
    pub grh_index: u16,
    pub obj_type: u8,
    pub min_hit: u16,
    pub max_hit: u16,
    pub min_def: u16,
    pub max_def: u16,
    pub value: u32,
    pub newbie: bool,
    pub no_drop: bool,
}

pub fn get_item_data(game_data: &GameData, item_id: i32) -> ItemData {
    if let Some(obj) = game_data.get_object(item_id) {
        ItemData {
            name: obj.name.clone(),
            grh_index: obj.grh_index,
            obj_type: obj.obj_type as u8,
            min_hit: obj.min_hit.max(0) as u16,
            max_hit: obj.max_hit.max(0) as u16,
            min_def: obj.min_def.max(0) as u16,
            max_def: obj.max_def.max(0) as u16,
            value: obj.valor.max(0) as u32,
            newbie: obj.newbie != 0,
            no_drop: obj.no_se_cae != 0,
        }
    } else {
        ItemData {
            name: format!("Item #{}", item_id),
            grh_index: 500,
            obj_type: 1,
            min_hit: 0, max_hit: 0, min_def: 0, max_def: 0,
            value: 1,
            newbie: false,
            no_drop: false,
        }
    }
}

/// Frontend readDeleteCharacter: id=getShort
pub fn build_delete_character_packet(entity_id: u32) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id_and_capacity(client_packet_id::DELETE_CHARACTER, 3);
    w.write_short(entity_id as u16);
    w.into_bytes()
}

/// Frontend readSelfVitalsDelta: hp=getShort, maxHp=getShort, mana=getShort, maxMana=getShort
pub fn build_self_vitals(hp: i32, max_hp: i32, mana: i32, max_mana: i32) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id_and_capacity(client_packet_id::SELF_VITALS_DELTA, 9);
    w.write_short(hp as u16);
    w.write_short(max_hp as u16);
    w.write_short(mana as u16);
    w.write_short(max_mana as u16);
    w.into_bytes()
}

/// Frontend readEntityVitalsDelta: id=getShort, hp=getShort, maxHp=getShort,
/// mana=getShort (optional), maxMana=getShort (optional)
pub fn build_entity_vitals_delta(entity_id: u32, hp: i32, max_hp: i32, mana: i32, max_mana: i32) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id_and_capacity(client_packet_id::ENTITY_VITALS_DELTA, 11);
    w.write_short(entity_id as u16);
    w.write_short(hp as u16);
    w.write_short(max_hp as u16);
    w.write_short(mana as u16);
    w.write_short(max_mana as u16);
    w.into_bytes()
}

/// Frontend readLearnSpell: slot=getByte, idSpell=getShort, name=getString, manaRequired=getShort
pub fn build_learn_spell(slot: u8, spell_id: u16, name: &str, mana_required: u16) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id(client_packet_id::APRENDER_SPELL);
    w.write_byte(slot);
    w.write_short(spell_id);
    w.write_string(name);
    w.write_short(mana_required);
    w.into_bytes()
}

/// Frontend readAnimFx: entityId=getShort, fxId=getShort
pub fn build_anim_fx(entity_id: u32, fx_id: u16) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id_and_capacity(client_packet_id::ANIM_FX, 5);
    w.write_short(entity_id as u16);
    w.write_short(fx_id);
    w.into_bytes()
}

/// Frontend readSpellProjectile: startX=getByte, startY=getByte, endX=getByte, endY=getByte, spellId=getShort
pub fn build_spell_projectile(start_x: i32, start_y: i32, end_x: i32, end_y: i32, spell_id: u16) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id(client_packet_id::SPELL_PROJECTILE);
    w.write_byte(start_x as u8);
    w.write_byte(start_y as u8);
    w.write_byte(end_x as u8);
    w.write_byte(end_y as u8);
    w.write_short(spell_id);
    w.into_bytes()
}

/// Composite spell visual packet (flags bitfield: 1=projectile, 2=fx, 4=sound, 8=words).
/// Mirrors original `handleProtocol.spellVisual`.
#[allow(clippy::too_many_arguments, dead_code)]
pub fn build_spell_visual(
    start_x: Option<u8>, start_y: Option<u8>,
    end_x: Option<u8>, end_y: Option<u8>,
    spell_id: Option<u16>,
    target_id: Option<u16>,
    fx_grh: Option<u16>,
    sound_id: Option<u16>,
    caster_id: Option<u16>,
    msg: Option<&str>,
) -> Option<Vec<u8>> {
    let has_projectile = start_x.is_some() && start_y.is_some()
        && end_x.is_some() && end_y.is_some()
        && spell_id.unwrap_or(0) > 0;
    let has_fx = fx_grh.unwrap_or(0) > 0;
    let has_sound = sound_id.unwrap_or(0) > 0;
    let has_words = msg.map_or(false, |m| !m.trim().is_empty()) && caster_id.is_some();

    let mut flags: u8 = 0;
    if has_projectile { flags |= 1; }
    if has_fx { flags |= 1 << 1; }
    if has_sound { flags |= 1 << 2; }
    if has_words { flags |= 1 << 3; }

    if flags == 0 {
        return None;
    }

    let mut w = PacketWriter::with_packet_id(client_packet_id::SPELL_VISUAL);
    w.write_byte(flags);

    if has_projectile {
        w.write_byte(start_x.unwrap());
        w.write_byte(start_y.unwrap());
        w.write_byte(end_x.unwrap());
        w.write_byte(end_y.unwrap());
        w.write_short(spell_id.unwrap());
    }

    if has_fx || has_sound {
        w.write_short(target_id.unwrap_or(0));
    }

    if has_fx {
        w.write_short(fx_grh.unwrap());
    }

    if has_sound {
        w.write_short(sound_id.unwrap());
    }

    if has_words {
        w.write_short(caster_id.unwrap());
        w.write_string(msg.unwrap());
    }

    Some(w.into_bytes())
}

/// Frontend readCreateProjectile: startX=getByte, startY=getByte, endX=getByte, endY=getByte, grhIndex=getShort
pub fn build_create_projectile(start_x: i32, start_y: i32, end_x: i32, end_y: i32, grh_index: u16) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id(client_packet_id::CREATE_PROJECTILE);
    w.write_byte(start_x as u8);
    w.write_byte(start_y as u8);
    w.write_byte(end_x as u8);
    w.write_byte(end_y as u8);
    w.write_short(grh_index);
    w.into_bytes()
}

/// Frontend readPlaySound: soundId=getShort (no entity ID)
pub fn build_play_sound(sound_id: u16) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id_and_capacity(client_packet_id::PLAY_SOUND, 3);
    w.write_short(sound_id);
    w.into_bytes()
}

#[allow(dead_code)]
pub struct SpellData {
    pub id: u16,
    pub name: String,
    pub mana_cost: u16,
    pub spell_type: SpellType,
    pub min_damage: i32,
    pub max_damage: i32,
    pub fx_id: u16,
    pub wav: u16,
}

pub enum SpellType {
    Attack,
    Heal,
    Buff,
}

pub fn get_spell_data(game_data: &GameData, spell_id: u16) -> SpellData {
    if let Some(spell) = game_data.get_spell(spell_id as i32) {
        let spell_type = match spell.spell_type {
            1 => SpellType::Attack,
            2 => SpellType::Heal,
            _ => SpellType::Buff,
        };
        SpellData {
            id: spell_id,
            name: spell.name.clone(),
            mana_cost: spell.mana_required as u16,
            spell_type,
            min_damage: spell.min_hp.abs().max(spell.min_ag).max(spell.min_fz),
            max_damage: spell.max_hp.abs().max(spell.max_ag).max(spell.max_fz),
            fx_id: spell.fx_grh,
            wav: spell.wav,
        }
    } else {
        SpellData {
            id: spell_id,
            name: "Hechizo Desconocido".into(),
            mana_cost: 10,
            spell_type: SpellType::Attack,
            min_damage: 1, max_damage: 5,
            fx_id: 1, wav: 0,
        }
    }
}

/// Frontend readRenderItem: x=getShort, y=getShort, itemId=getShort, amount=getShort, grhIndex=getShort
pub fn build_render_item(x: i32, y: i32, item_id: i32, amount: i32, grh_index: u16) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id(client_packet_id::RENDER_ITEM);
    w.write_short(x as u16);
    w.write_short(y as u16);
    w.write_short(item_id as u16);
    w.write_short(amount as u16);
    w.write_short(grh_index);
    w.into_bytes()
}

/// Frontend readDeleteItem: x=getShort, y=getShort
pub fn build_delete_ground_item(x: i32, y: i32) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id_and_capacity(client_packet_id::DELETE_ITEM, 5);
    w.write_short(x as u16);
    w.write_short(y as u16);
    w.into_bytes()
}

/// Returns (item_id, amount, grh_index) for each loot drop from an NPC template.
/// Uses SmallVec to avoid heap allocation for typical 1-4 item loot tables.
pub fn get_npc_loot(game_data: &GameData, npc_type: i32) -> SmallVec<[(i32, i32, u16); 4]> {
    let Some(npc) = game_data.get_npc(npc_type) else {
        return SmallVec::new();
    };

    npc.drop.iter().map(|drop| {
        let grh = game_data.get_object(drop.item)
            .map(|obj| obj.grh_index)
            .unwrap_or(500);
        (drop.item, drop.cant, grh)
    }).collect()
}

pub fn get_class_name(id_clase: i32) -> &'static str {
    match id_clase {
        1 => "Mago",
        2 => "Clerigo",
        3 => "Guerrero",
        4 => "Asesino",
        5 => "Bardo",
        6 => "Druida",
        7 => "Paladin",
        8 => "Cazador",
        _ => "Aventurero",
    }
}

pub const DEFAULT_SPELLS: [(u8, u16); 4] = [
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 4),
];
