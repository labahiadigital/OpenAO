use openao_protocol::opcodes::client_packet_id;
use openao_protocol::PacketWriter;

pub fn heading_to_delta(heading: u8) -> (i32, i32) {
    match heading {
        1 => (0, -1),
        2 => (0, 1),
        3 => (-1, 0),
        4 => (1, 0),
        _ => (0, 0),
    }
}

pub fn build_name_map(name: &str) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id(client_packet_id::NAME_MAP);
    w.write_string(name);
    w.into_bytes()
}

/// Frontend readActGold: gold=getInt
pub fn build_act_gold(gold: i32) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id_and_capacity(client_packet_id::ACT_GOLD, 5);
    w.write_int(gold as u32);
    w.into_bytes()
}

pub fn build_self_vitals(hp: i32, max_hp: i32, mana: i32, max_mana: i32) -> Vec<u8> {
    crate::replication::build_self_vitals(hp, max_hp, mana, max_mana)
}

/// Frontend readActPosition: id=getShort, x=getShort, y=getShort, moveId=getShort
pub fn build_act_position(entity_id: u32, x: i32, y: i32) -> Vec<u8> {
    build_act_position_with_move_id(entity_id, x, y, 0)
}

/// Build ACT_POSITION echoing the client's moveId for reconciliation.
pub fn build_act_position_with_move_id(entity_id: u32, x: i32, y: i32, move_id: u16) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id_and_capacity(client_packet_id::ACT_POSITION, 9);
    w.write_short(entity_id as u16);
    w.write_short(x as u16);
    w.write_short(y as u16);
    w.write_short(move_id);
    w.into_bytes()
}

/// Frontend readConsole: text=getString
pub fn build_console_message(text: &str) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id(client_packet_id::CONSOLE);
    w.write_string(text);
    w.into_bytes()
}

/// Frontend readDialog: text=getString
pub fn build_dialog_message(text: &str) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id(client_packet_id::DIALOG);
    w.write_string(text);
    w.into_bytes()
}

pub fn build_self_flags(zona_segura: u8, seguro: bool, seguro_clan: bool) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id(client_packet_id::SELF_FLAGS_DELTA);
    w.write_byte(zona_segura);
    w.write_byte(if seguro { 1 } else { 0 });
    w.write_byte(if seguro_clan { 1 } else { 0 });
    w.into_bytes()
}

/// Frontend readActMyLevel: level=getShort
pub fn build_act_level(level: i32) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id(client_packet_id::ACT_MY_LEVEL);
    w.write_short(level as u16);
    w.into_bytes()
}

/// Frontend readActExp: exp=getInt, [expNextLevel=getInt if remainingBytes >= 4]
pub fn build_act_exp(exp: i32, exp_next: i32) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id(client_packet_id::ACT_EXP);
    w.write_int(exp as u32);
    w.write_int(exp_next as u32);
    w.into_bytes()
}

/// Frontend readCharacterStatsSnapshot: fuerza=getShort, agilidad=getShort,
/// inteligencia=getShort, constitucion=getShort, [minHit=getShort, maxHit=getShort if remainingBytes >= 4]
pub fn build_self_attributes(fuerza: i32, agilidad: i32, inteligencia: i32, constitucion: i32, min_hit: i32, max_hit: i32) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id(client_packet_id::CHARACTER_STATS_SNAPSHOT);
    w.write_short(fuerza as u16);
    w.write_short(agilidad as u16);
    w.write_short(inteligencia as u16);
    w.write_short(constitucion as u16);
    w.write_short(min_hit as u16);
    w.write_short(max_hit as u16);
    w.into_bytes()
}

/// Frontend readChangeRopa/Helmet/Weapon/Shield/Body: entityId=getShort, grhId=getShort
pub fn build_change_equipment(packet_id: u8, entity_id: u32, visual_id: i32) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id_and_capacity(packet_id, 5);
    w.write_short(entity_id as u16);
    w.write_short(visual_id as u16);
    w.into_bytes()
}

/// Frontend readActColorName: id=getShort, colorCode=getByte
pub fn build_act_color_name(entity_id: u32, color: u8) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id_and_capacity(client_packet_id::ACT_COLOR_NAME, 4);
    w.write_short(entity_id as u16);
    w.write_byte(color);
    w.into_bytes()
}

pub fn get_name_color(criminal: bool, faction: &str, is_admin: bool) -> u8 {
    if is_admin {
        4
    } else if faction == "armada" {
        2
    } else if faction == "caos" {
        3
    } else if criminal {
        1
    } else {
        0
    }
}

/// Frontend readActOnline: count=getShort
pub fn build_act_online(count: u16) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id(client_packet_id::ACT_ONLINE);
    w.write_short(count);
    w.into_bytes()
}

/// Frontend readOpenBail: kills=getInt, citizensKilled=getInt, fianza=getInt,
/// goldRequired=getInt, goldAvailable=getInt, canPay=getByte
pub fn build_open_bail(kills: i32, citizens_killed: i32, fianza: i32, gold_required: i32, gold_available: i32, can_pay: bool) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id(client_packet_id::OPEN_BAIL);
    w.write_int(kills as u32);
    w.write_int(citizens_killed as u32);
    w.write_int(fianza as u32);
    w.write_int(gold_required as u32);
    w.write_int(gold_available as u32);
    w.write_byte(if can_pay { 1 } else { 0 });
    w.into_bytes()
}

/// Frontend readStartCastBar: entityId=getShort, durationMs=getInt
#[allow(dead_code)]
pub fn build_start_cast_bar(entity_id: u32, duration_ms: u32) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id(client_packet_id::START_CAST_BAR);
    w.write_short(entity_id as u16);
    w.write_int(duration_ms);
    w.into_bytes()
}

/// Frontend readStopCastBar: entityId=getShort
#[allow(dead_code)]
pub fn build_stop_cast_bar(entity_id: u32) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id(client_packet_id::STOP_CAST_BAR);
    w.write_short(entity_id as u16);
    w.into_bytes()
}

/// Frontend readOpenCrafting: json=getString
pub fn build_open_crafting(json_payload: &str) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id(client_packet_id::OPEN_CRAFTING);
    w.write_string(json_payload);
    w.into_bytes()
}

/// Deprecated: use `balance::recalc_on_level_up` for exact per-level stats.
#[allow(dead_code)]
pub fn class_level_bonus(id_clase: i32) -> (i32, i32) {
    match id_clase {
        1 => (8, 22),   // Mago
        2 => (9, 17),   // Clerigo
        3 => (11, 0),   // Guerrero
        4 => (9, 8),    // Asesino
        5 => (9, 14),   // Bardo
        6 => (9, 17),   // Druida
        7 => (10, 8),   // Paladin
        8 => (9, 6),    // Cazador
        _ => (9, 10),
    }
}

/// Frontend readNavegando: navegando=getByte
pub fn build_navegando(navegando: bool) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id(client_packet_id::NAVEGANDO);
    w.write_byte(if navegando { 1 } else { 0 });
    w.into_bytes()
}

/// Frontend readGlobalNotice: text=getString
#[allow(dead_code)]
pub fn build_global_notice(text: &str) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id(client_packet_id::GLOBAL_NOTICE);
    w.write_string(text);
    w.into_bytes()
}

/// Frontend readSelfMapMetaDelta: mapName=getString
pub fn build_self_map_meta_delta(map_name: &str) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id(client_packet_id::SELF_MAP_META_DELTA);
    w.write_string(map_name);
    w.into_bytes()
}

/// Frontend readDeath (putBodyAndHeadDead): id=getShort, head=getShort, body=getShort,
/// helmet=getShort, weapon=getShort, shield=getShort
pub fn build_put_body_and_head_dead(entity_id: u32, head: i32, body: i32, helmet: i32, weapon: i32, shield: i32) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id(client_packet_id::PUT_BODY_AND_HEAD_DEAD);
    w.write_short(entity_id as u16);
    w.write_short(head as u16);
    w.write_short(body as u16);
    w.write_short(helmet as u16);
    w.write_short(weapon as u16);
    w.write_short(shield as u16);
    w.into_bytes()
}

/// Frontend readRevive (revivirUsuario): id=getShort, head=getShort, body=getShort
pub fn build_revivir_usuario(entity_id: u32, head: i32, body: i32) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id(client_packet_id::REVIVIR_USUARIO);
    w.write_short(entity_id as u16);
    w.write_short(head as u16);
    w.write_short(body as u16);
    w.into_bytes()
}

/// Frontend readPartyState: json=getString (delta with upsert/remove arrays)
pub fn build_party_state(json: &str) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id(client_packet_id::PARTY_STATE);
    w.write_string(json);
    w.into_bytes()
}

/// Frontend readClanState: json=getString (delta with upsert/remove arrays)
#[allow(dead_code)]
pub fn build_clan_state(json: &str) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id(client_packet_id::CLAN_STATE);
    w.write_string(json);
    w.into_bytes()
}

/// Frontend readCloseBail
#[allow(dead_code)]
pub fn build_close_bail() -> Vec<u8> {
    let w = PacketWriter::with_packet_id(client_packet_id::CLOSE_BAIL);
    w.into_bytes()
}

/// Frontend readCloseForce — closes all open modals on the client
#[allow(dead_code)]
pub fn build_close_force() -> Vec<u8> {
    let w = PacketWriter::with_packet_id(client_packet_id::CLOSE_FORCE);
    w.into_bytes()
}

/// Frontend readBlockMap: x=getShort, y=getShort, blocked=getByte
#[allow(dead_code)]
pub fn build_block_map(x: i32, y: i32, blocked: bool) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id(client_packet_id::BLOCK_MAP);
    w.write_short(x as u16);
    w.write_short(y as u16);
    w.write_byte(if blocked { 1 } else { 0 });
    w.into_bytes()
}

/// Frontend readInmo: inmo=getByte, x=getShort, y=getShort
#[allow(dead_code)]
pub fn build_inmo(x: i32, y: i32, state: u8) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id(client_packet_id::INMO);
    w.write_byte(state);
    w.write_short(x as u16);
    w.write_short(y as u16);
    w.into_bytes()
}

/// Frontend readSpellVisual: flags=getByte + conditional fields
/// Encodes spell visual as a bitmap with optional projectile, FX, sound, and cast words.
#[allow(dead_code)]
pub fn build_spell_visual(
    projectile: Option<(i32, i32, i32, i32, u16)>,
    target_id: Option<u32>,
    fx_grh: Option<u16>,
    sound_id: Option<u16>,
    cast_words: Option<(u32, &str)>,
) -> Vec<u8> {
    let mut flags: u8 = 0;
    if projectile.is_some() { flags |= 1; }
    if fx_grh.is_some() { flags |= 1 << 1; }
    if sound_id.is_some() { flags |= 1 << 2; }
    if cast_words.is_some() { flags |= 1 << 3; }

    let mut w = PacketWriter::with_packet_id(client_packet_id::SPELL_VISUAL);
    w.write_byte(flags);

    if let Some((sx, sy, ex, ey, spell_id)) = projectile {
        w.write_byte(sx as u8);
        w.write_byte(sy as u8);
        w.write_byte(ex as u8);
        w.write_byte(ey as u8);
        w.write_short(spell_id);
    }

    let has_target_id = fx_grh.is_some() || sound_id.is_some();
    if has_target_id {
        w.write_short(target_id.unwrap_or(0) as u16);
    }

    if let Some(grh) = fx_grh {
        w.write_short(grh);
    }

    if let Some(sid) = sound_id {
        w.write_short(sid);
    }

    if let Some((caster_id, msg)) = cast_words {
        w.write_short(caster_id as u16);
        w.write_string(msg);
    }

    w.into_bytes()
}

/// Frontend readOpenRetos: json=getString
#[allow(dead_code)]
pub fn build_open_retos(json: &str) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id(client_packet_id::OPEN_RETOS);
    w.write_string(json);
    w.into_bytes()
}

/// Frontend readOpenMarket: json=getString
#[allow(dead_code)]
pub fn build_open_market(json: &str) -> Vec<u8> {
    let mut w = PacketWriter::with_packet_id(client_packet_id::OPEN_MARKET);
    w.write_string(json);
    w.into_bytes()
}

/// Deprecated: use `balance::get_legacy_exp_next_level` (identical logic, proper clamping).
#[allow(dead_code)]
pub fn calc_exp_next_level(level: i32) -> i32 {
    crate::gameplay::balance::get_legacy_exp_next_level(level)
}
