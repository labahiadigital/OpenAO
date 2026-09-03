use openao_protocol::opcodes::client_packet_id;
use openao_protocol::PacketWriter;

use super::packets::*;
use super::GameSession;
use crate::error::{GameError, HandlerResult};
use crate::gateway::WsSink;

impl GameSession {
    /// Dispatch a slash command to the appropriate handler via a linear scan
    /// of prefix/exact match rules. This replaces the original 170-line
    /// if-else chain with a data-driven table that is easy to extend.
    async fn dispatch_command(
        &mut self,
        cmd: &str,
        cmd_lower: &str,
        entity_id: u32,
        sink: &mut WsSink,
    ) -> Result<bool, crate::error::HandlerError> {
        // --- Prefix-matched commands (order matters: longer prefixes first) ---
        if cmd_lower.starts_with("tp ") {
            self.handle_teleport(cmd, sink).await?;
            return Ok(true);
        }
        if cmd_lower.starts_with("global ") {
            self.handle_global_chat(&cmd[7..], entity_id).await?;
            return Ok(true);
        }
        if cmd_lower.starts_with("w ") {
            self.handle_whisper(&cmd[2..], entity_id, sink).await?;
            return Ok(true);
        }
        if cmd_lower.starts_with("fundir ") {
            let recipe_id: i32 = cmd[7..].trim().parse().unwrap_or(0);
            self.handle_smelt(recipe_id, sink).await?;
            return Ok(true);
        }
        if cmd_lower.starts_with("p ") {
            self.handle_party_chat(&cmd[2..], entity_id, sink).await?;
            return Ok(true);
        }
        if cmd_lower.starts_with("c ") {
            self.handle_clan_chat(&cmd[2..], entity_id, sink).await?;
            return Ok(true);
        }
        if cmd_lower.starts_with("party ") {
            self.handle_party_invite(cmd[6..].trim(), entity_id, sink).await?;
            return Ok(true);
        }
        if cmd_lower.starts_with("expulsarparty ") {
            self.handle_party_kick(cmd[14..].trim(), entity_id, sink).await?;
            return Ok(true);
        }

        // --- Clan aliases (must come before "clan " prefix) ---
        if let Some(sub) = self.resolve_clan_alias(cmd, cmd_lower) {
            self.handle_clan_command(&sub, entity_id, sink).await?;
            return Ok(true);
        }
        if cmd_lower.starts_with("clan ") {
            self.handle_clan_command(cmd[5..].trim(), entity_id, sink).await?;
            return Ok(true);
        }

        if cmd_lower.starts_with("faccion") {
            let arg = cmd_lower.strip_prefix("faccion").unwrap_or("").trim().to_string();
            self.handle_faction_command(&arg, entity_id, sink).await?;
            return Ok(true);
        }
        if cmd_lower.starts_with("fianza") {
            let arg = cmd_lower.strip_prefix("fianza").unwrap_or("").trim().to_string();
            self.handle_fianza(&arg, entity_id, sink).await?;
            return Ok(true);
        }

        // --- Quest commands ---
        if cmd_lower.starts_with("mision aceptar ") || cmd_lower.starts_with("quest accept ") {
            let id_str = if cmd_lower.starts_with("mision aceptar ") { &cmd[15..] } else { &cmd[13..] };
            let qid: u32 = id_str.trim().parse().unwrap_or(0);
            self.handle_quest_accept(qid, entity_id, sink).await?;
            return Ok(true);
        }
        if cmd_lower.starts_with("mision abandonar ") || cmd_lower.starts_with("quest abandon ") {
            let id_str = if cmd_lower.starts_with("mision abandonar ") { &cmd[17..] } else { &cmd[14..] };
            let qid: u32 = id_str.trim().parse().unwrap_or(0);
            self.handle_quest_abandon(qid, entity_id, sink).await?;
            return Ok(true);
        }
        if cmd_lower.starts_with("mision completar ") || cmd_lower.starts_with("quest complete ") {
            let id_str = if cmd_lower.starts_with("mision completar ") { &cmd[17..] } else { &cmd[15..] };
            let qid: u32 = id_str.trim().parse().unwrap_or(0);
            self.handle_quest_complete(qid, entity_id, sink).await?;
            return Ok(true);
        }

        // --- Pet commands ---
        if cmd_lower.starts_with("invocar ") || cmd_lower.starts_with("petsummon ") {
            let idx_str = if cmd_lower.starts_with("invocar ") { &cmd[8..] } else { &cmd[10..] };
            let idx: usize = idx_str.trim().parse().unwrap_or(0);
            self.handle_pet_summon(idx, entity_id, sink).await?;
            return Ok(true);
        }
        if cmd_lower.starts_with("liberar ") || cmd_lower.starts_with("petrelease ") {
            let idx_str = if cmd_lower.starts_with("liberar ") { &cmd[8..] } else { &cmd[11..] };
            let idx: usize = idx_str.trim().parse().unwrap_or(0);
            self.handle_pet_release(idx, entity_id, sink).await?;
            return Ok(true);
        }

        // --- Admin commands with args ---
        if cmd_lower.starts_with("darexp ") { self.handle_admin_give_exp(&cmd[7..], entity_id, sink).await?; return Ok(true); }
        if cmd_lower.starts_with("daroro ") { self.handle_admin_give_gold(&cmd[7..], entity_id, sink).await?; return Ok(true); }
        if cmd_lower.starts_with("kick ") { self.handle_admin_kick(&cmd[5..], entity_id, sink).await?; return Ok(true); }
        if cmd_lower.starts_with("telepuser ") { self.handle_admin_telepuser(&cmd[10..], entity_id, sink).await?; return Ok(true); }
        if cmd_lower.starts_with("traer ") { self.handle_admin_bring(&cmd[6..], entity_id, sink).await?; return Ok(true); }
        if cmd_lower.starts_with("crearitem ") { self.handle_admin_create_item(&cmd[10..], entity_id, sink).await?; return Ok(true); }
        if cmd_lower.starts_with("ban ") { self.handle_admin_ban(&cmd[4..], sink).await?; return Ok(true); }
        if cmd_lower.starts_with("unban ") { self.handle_admin_unban(&cmd[6..], sink).await?; return Ok(true); }
        if cmd_lower.starts_with("mute ") { self.handle_admin_mute(&cmd[5..], sink).await?; return Ok(true); }
        if cmd_lower.starts_with("inspect ") { self.handle_admin_inspect(&cmd[8..], sink).await?; return Ok(true); }
        if cmd_lower.starts_with("cambiarclase ") { self.handle_admin_change_class(&cmd[13..], entity_id, sink).await?; return Ok(true); }
        if cmd_lower.starts_with("invocarnpc ") { self.handle_admin_spawn_npc(&cmd[11..], entity_id, sink).await?; return Ok(true); }
        if cmd_lower.starts_with("carcel ") || cmd_lower.starts_with("jail ") {
            let args = if cmd_lower.starts_with("carcel ") { &cmd[7..] } else { &cmd[5..] };
            self.handle_admin_jail(args.trim(), entity_id, sink).await?;
            return Ok(true);
        }
        if cmd_lower.starts_with("globalgm") { self.handle_admin_global_gm(cmd.get(9..).unwrap_or("").trim(), sink).await?; return Ok(true); }
        if cmd_lower.starts_with("banip ") { self.handle_admin_ban_ip(&cmd[6..], sink).await?; return Ok(true); }
        if cmd_lower.starts_with("unbanip ") { self.handle_admin_unban_ip(&cmd[8..], sink).await?; return Ok(true); }
        if cmd_lower.starts_with("verip ") { self.handle_admin_verip(cmd[6..].trim(), entity_id, sink).await?; return Ok(true); }
        if cmd_lower.starts_with("bot ") { self.handle_admin_bot(&cmd[4..], entity_id, sink).await?; return Ok(true); }
        if cmd_lower.starts_with("intervalo ") { self.handle_admin_intervalo(&cmd[10..], sink).await?; return Ok(true); }

        // --- Exact-match commands ---
        match cmd_lower {
            "revivir" | "respawn" => self.handle_respawn(sink).await?,
            "meditar" => self.handle_meditate(entity_id, sink).await?,
            "stats" | "estadisticas" => self.handle_stats(entity_id, sink).await?,
            "aceptar" => self.handle_party_accept(entity_id, sink).await?,
            "salirparty" => self.handle_party_leave(entity_id, sink).await?,
            "hogar" => self.handle_hogar(entity_id, sink).await?,
            "asignarhogar" => self.handle_asignar_hogar(entity_id, sink).await?,
            "enlistar" => self.handle_enlistar(entity_id, sink).await?,
            "recompensa" => self.handle_recompensa(entity_id, sink).await?,
            "telepme" => self.handle_admin_telepme(entity_id, sink).await?,
            "devrevivir" => self.handle_admin_revive(entity_id, sink).await?,
            "worldsave" => self.handle_admin_world_save(sink).await?,
            "limpiarpiso" | "cleanfloor" => self.handle_admin_clean_floor(entity_id, sink).await?,
            "dobleexp" => self.handle_admin_toggle_double_exp(sink).await?,
            "dobleoro" => self.handle_admin_toggle_double_gold(sink).await?,
            "devresetmap" => self.handle_admin_reset_map(entity_id, sink).await?,
            "quitarnpc" | "borrarnpc" => self.handle_admin_remove_npc(entity_id, sink).await?,
            "quitarnpcpermanente" | "borrarnpcpermanente" => self.handle_admin_remove_npc_permanent(entity_id, sink).await?,
            "embarcar" | "navegar" => self.handle_embarcar(entity_id, sink).await?,
            "desembarcar" => self.handle_desembarcar(entity_id, sink).await?,
            "comerciar" | "trade" => { self.send_to_client(sink, build_console_message("Haz click en un jugador para iniciar una transacción.")).await?; }
            "cancelartrade" | "cancelarcomercio" => self.handle_trade_cancel(sink).await?,
            "confirmartrade" | "confirmarcomercio" => self.handle_trade_confirm(sink).await?,
            "misiones" | "quests" => self.handle_quest_list(entity_id, sink).await?,
            "mascotas" | "pets" => self.handle_pet_list(entity_id, sink).await?,
            "despachar" | "petdismiss" => self.handle_pet_dismiss(entity_id, sink).await?,
            "territorios" | "territories" => self.handle_territory_list(entity_id, sink).await?,
            "logros" | "achievements" => self.handle_achievements_list(entity_id, sink).await?,
            "salir" => self.handle_safe_logout(entity_id, sink).await?,
            "apagar" => { self.send_to_client(sink, build_console_message("Apagado programado no soportado. Usa Ctrl+C.")).await?; }
            "invi" => self.handle_admin_invisibility(entity_id, sink).await?,
            "resetaciertos" => { self.send_to_client(sink, build_console_message("Reset de aciertos: no aplica en el motor actual.")).await?; }
            "recargarobjs" | "recargarnpcs" | "recargarbalance" | "recargarcrafting" | "recargar" => {
                match self.world.reload_game_data() {
                    Ok(()) => {
                        self.send_to_client(sink, build_console_message("Game data recargada exitosamente.")).await?;
                    }
                    Err(e) => {
                        self.send_to_client(sink, build_console_message(&e)).await?;
                    }
                }
            }
            "intervalos" | "paquetes" | "packettop" => {
                self.handle_admin_packet_stats(sink).await?;
            }
            "desinvocarbots" | "quitarbots" | "borrarbots" => {
                self.handle_admin_remove_bots(entity_id, sink).await?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    /// Resolve `/clanXXX arg` shorthand aliases into `"XXX arg"` for handle_clan_command.
    fn resolve_clan_alias(&self, cmd: &str, cmd_lower: &str) -> Option<String> {
        static CLAN_ALIASES: &[(&str, &str, usize)] = &[
            ("clancrear ", "crear ", 10),
            ("clanexpulsar ", "expulsar ", 13),
            ("clanlider ", "lider ", 10),
            ("claneliminar", "eliminar ", 13),
            ("clanpostular ", "postular ", 13),
            ("clanaceptar ", "aceptar ", 12),
            ("clanrechazar ", "rechazar ", 13),
            ("clancolider ", "colider ", 12),
        ];
        if cmd_lower == "clansalir" {
            return Some("salir".to_string());
        }
        for &(prefix, sub_cmd, skip) in CLAN_ALIASES {
            if cmd_lower.starts_with(prefix) {
                return Some(format!("{}{}", sub_cmd, &cmd[skip..]));
            }
        }
        None
    }

    /// Stub commands matched by prefix that just return a message.
    fn try_stub_prefix_command(&self, cmd_lower: &str) -> Option<&'static str> {
        if cmd_lower.starts_with("intervalo ") {
            return Some("Cambio de intervalos no implementado.");
        }
        if cmd_lower.starts_with("bot ") {
            return Some("Sistema de bots no implementado.");
        }
        
        None
    }

    pub(super) async fn handle_dialog(
        &mut self,
        message: &str,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let entity_id = match self.entity_id {
            Some(id) => id,
            None => return Ok(()),
        };

        if let Some(map_id) = self.map_id {
            let now = self.world.uptime_ms();
            let scene = self.world.get_or_create_scene(map_id);
            if let Some(p) = scene.players.get(&entity_id) {
                if !p.action_cooldowns.can_dialog(now) {
                    return Ok(());
                }
            }
            if let Some(mut p) = scene.players.get_mut(&entity_id) {
                p.action_cooldowns.trigger_dialog(now);
            }
        }

        let name = self.character_name.as_deref().unwrap_or("???");

        if let Some(cmd) = message.strip_prefix('/') {
            let cmd_lower = cmd.to_lowercase();
            tracing::info!(
                target: "chat_audit",
                player = %name, entity = entity_id,
                map = ?self.map_id, cmd = %cmd_lower,
                "CMD"
            );

            if self.dispatch_command(cmd, &cmd_lower, entity_id, sink).await? {
            } else if let Some(stub_msg) = self.try_stub_prefix_command(&cmd_lower) {
                self.send_to_client(sink, build_console_message(stub_msg)).await?;
            } else {
                let response = self.handle_command(cmd);
                self.send_to_client(sink, build_console_message(&response)).await?;
            }
        } else {
            if self.world.muted_players.contains_key(&entity_id) {
                self.send_to_client(sink, build_console_message("Estas muteado y no puedes hablar.")).await?;
                return Ok(());
            }
            tracing::info!(
                target: "chat_audit",
                player = %name, entity = entity_id,
                map = ?self.map_id,
                "CHAT: {}", message
            );
            let chat_text = format!("{} dice: {}", name, message);
            let pkt = build_dialog_message(&chat_text);

            self.send_to_client(sink, pkt.clone()).await?;

            if let Some(mid) = self.map_id {
                let scene = self.world.get_or_create_scene(mid);
                let chat_pos = scene.players.get(&entity_id).map(|p| p.pos.clone());
                if let Some(ref pos) = chat_pos {
                    scene.broadcast_in_range(entity_id, pos, pkt);
                } else {
                    scene.broadcast(entity_id, pkt);
                }
            }
        }

        Ok(())
    }

    pub(super) fn handle_command(&self, cmd: &str) -> String {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let command = parts.first().unwrap_or(&"").to_lowercase();

        match command.as_str() {
            "online" => {
                let mut total_pve = 0usize;
                let total: usize = self.world.scenes.iter().map(|s| {
                    let count = s.players.len();
                    total_pve += count;
                    count
                }).sum();
                format!("Usuarios online: {} (PvE: {})", total, total_pve)
            }
            "pos" => {
                if let (Some(eid), Some(mid)) = (self.entity_id, self.map_id) {
                    let scene = self.world.get_or_create_scene(mid);
                    if let Some(p) = scene.players.get(&eid) {
                        format!("Posicion: mapa={}, x={}, y={}", p.pos.map, p.pos.x, p.pos.y)
                    } else {
                        "No se pudo obtener posicion".to_string()
                    }
                } else {
                    "No conectado".to_string()
                }
            }
            "hp" => {
                if let (Some(eid), Some(mid)) = (self.entity_id, self.map_id) {
                    let scene = self.world.get_or_create_scene(mid);
                    if let Some(p) = scene.players.get(&eid) {
                        format!("HP: {}/{}, Mana: {}/{}", p.hp, p.max_hp, p.mana, p.max_mana)
                    } else {
                        "No encontrado".to_string()
                    }
                } else {
                    "No conectado".to_string()
                }
            }
            "help" => {
                "Comandos: /online /pos /hp /stats /meditar /global /w /p /c /tp /revivir /fundir /faccion /enlistar /recompensa /fianza /asignarhogar /party /aceptar /salirparty /expulsarparty /clan [crear|salir|info|expulsar|lider|eliminar|postular|aceptar|rechazar|colider] /hogar /salir | Admin: /darexp /daroro /kick /telepme /telepuser /traer /devrevivir /crearitem /ban /unban /mute /globalgm /worldsave /inspect /cambiarclase /limpiarpiso /dobleexp /dobleoro /invocarnpc /quitarnpc /devresetmap /apagar | /help".to_string()
            }
            _ => format!("Comando desconocido: /{}", command),
        }
    }

    async fn handle_global_chat(
        &self,
        message: &str,
        sender_eid: u32,
    ) -> HandlerResult {
        if self.world.muted_players.contains_key(&sender_eid) {
            return Ok(());
        }
        let name = self.character_name.as_deref().unwrap_or("???");
        let text = message.trim();
        if text.is_empty() {
            return Ok(());
        }

        let chat_text = format!("[Global] {} dice: {}", name, text);
        let pkt = build_console_message(&chat_text);

        for scene_ref in self.world.scenes.iter() {
            let scene = scene_ref.value();
            scene.broadcast(sender_eid, pkt.clone());
            scene.send_to_player(sender_eid, pkt.clone());
        }
        Ok(())
    }

    async fn handle_whisper(
        &self,
        args: &str,
        entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        if self.world.muted_players.contains_key(&entity_id) {
            self.send_to_client(sink, build_console_message("Estas muteado.")).await?;
            return Ok(());
        }
        let trimmed = args.trim();
        let space_pos = trimmed.find(' ');
        let (target_name, message) = match space_pos {
            Some(pos) => (&trimmed[..pos], trimmed[pos + 1..].trim()),
            None => {
                self.send_to_client(sink, build_console_message("Uso: /w nombre mensaje")).await?;
                return Ok(());
            }
        };

        if message.is_empty() {
            self.send_to_client(sink, build_console_message("Uso: /w nombre mensaje")).await?;
            return Ok(());
        }

        let sender_name = self.character_name.as_deref().unwrap_or("???");
        let target_lower = target_name.to_lowercase();

        for scene_ref in self.world.scenes.iter() {
            let scene = scene_ref.value();
            for player_ref in scene.players.iter() {
                if player_ref.name.to_lowercase() == target_lower {
                    let whisper_msg = format!("{} te susurra: {}", sender_name, message);
                    let pkt = build_console_message(&whisper_msg);
                    scene.send_to_player(*player_ref.key(), pkt);

                    let confirm = format!("Susurras a {}: {}", target_name, message);
                    self.send_to_client(sink, build_console_message(&confirm)).await?;
                    return Ok(());
                }
            }
        }

        let err = GameError::player_not_found(target_name);
        self.send_to_client(sink, err.to_console_packet()).await?;
        Ok(())
    }

    async fn handle_meditate(
        &self,
        entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let map_id = match self.map_id {
            Some(id) => id,
            None => return Ok(()),
        };

        let scene = self.world.get_or_create_scene(map_id);
        if let Some(mut player) = scene.players.get_mut(&entity_id) {
            if player.mana >= player.max_mana {
                self.send_to_client(sink, build_console_message("Tu maná ya está al máximo.")).await?;
                return Ok(());
            }

            let regen = (player.max_mana as f32 * 0.08).max(5.0) as i32;
            player.mana = (player.mana + regen).min(player.max_mana);

            let vitals = build_self_vitals(player.hp, player.max_hp, player.mana, player.max_mana);
            self.send_to_client(sink, vitals).await?;
            self.send_to_client(sink, build_console_message("Meditas y recuperas maná.")).await?;

            let fx = crate::replication::build_anim_fx(entity_id, 1);
            let med_pos = scene.players.get(&entity_id).map(|p| p.pos.clone());
            if let Some(ref pos) = med_pos {
                scene.broadcast_in_range(entity_id, pos, fx);
            } else {
                scene.broadcast(entity_id, fx);
            }
        }

        Ok(())
    }

    async fn handle_stats(
        &self,
        entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let map_id = match self.map_id {
            Some(id) => id,
            None => return Ok(()),
        };

        let scene = self.world.get_or_create_scene(map_id);
        if let Some(player) = scene.players.get(&entity_id) {
            let stats = format!(
                "--- Estadísticas ---\nNivel: {} | Exp: {}/{}\nHP: {}/{} | Mana: {}/{}\nOro: {}\nFue: {} | Agi: {} | Int: {} | Con: {}",
                player.level, player.exp, player.exp_next_level,
                player.hp, player.max_hp, player.mana, player.max_mana,
                player.gold,
                player.attr_fuerza, player.attr_agilidad, player.attr_inteligencia, player.attr_constitucion,
            );
            self.send_to_client(sink, build_console_message(&stats)).await?;
            self.send_to_client(sink, build_self_attributes(
                player.attr_fuerza, player.attr_agilidad,
                player.attr_inteligencia, player.attr_constitucion,
                player.min_hit, player.max_hit,
            )).await?;
        }

        Ok(())
    }

    const REVIVE_CAST_MS: u64 = 10_000;

    pub(super) async fn handle_respawn(
        &mut self,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let entity_id = match self.entity_id {
            Some(id) => id,
            None => return Ok(()),
        };
        let map_id = match self.map_id {
            Some(id) => id,
            None => return Ok(()),
        };

        let scene = self.world.get_or_create_scene(map_id);
        let (is_dead, already_casting) = scene.players.get(&entity_id)
            .map(|p| (p.dead, p.revive_at_ms > 0))
            .unwrap_or((false, false));

        if !is_dead {
            self.send_to_client(sink, build_console_message("No estás muerto.")).await?;
            return Ok(());
        }

        if already_casting {
            self.send_to_client(sink, build_console_message("Ya estás resucitando...")).await?;
            return Ok(());
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        if let Some(mut p) = scene.players.get_mut(&entity_id) {
            p.revive_at_ms = now + Self::REVIVE_CAST_MS;
        }

        self.send_to_client(sink, build_start_cast_bar(entity_id, Self::REVIVE_CAST_MS as u32)).await?;
        self.send_to_client(sink, build_console_message("Resucitando... espera 10 segundos.")).await?;

        Ok(())
    }

    pub(super) fn check_pending_logout(&self, entity_id: u32) -> bool {
        let map_id = match self.map_id {
            Some(m) => m,
            None => return false,
        };
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let scene = self.world.get_or_create_scene(map_id);
        let logout_at = scene.players.get(&entity_id)
            .map(|p| p.logout_expires_at_ms)
            .unwrap_or(0);

        logout_at > 0 && now_ms >= logout_at
    }

    pub(super) async fn tick_revive(
        &mut self,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let entity_id = match self.entity_id {
            Some(id) => id,
            None => return Ok(()),
        };
        let map_id = match self.map_id {
            Some(id) => id,
            None => return Ok(()),
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let scene = self.world.get_or_create_scene(map_id);
        let revive_at = scene.players.get(&entity_id)
            .map(|p| p.revive_at_ms)
            .unwrap_or(0);

        if revive_at == 0 || now < revive_at {
            return Ok(());
        }

        let (home_map, home_x, home_y) = scene.players.get(&entity_id)
            .map(|p| (p.home_map, p.home_x, p.home_y))
            .unwrap_or((1, 50, 50));

        let need_teleport = home_map != map_id;

        if let Some(mut player) = scene.players.get_mut(&entity_id) {
            player.dead = false;
            player.dead_world_active = false;
            player.dead_world_transition_at_ms = 0;
            player.hp = player.max_hp / 2;
            player.mana = player.max_mana / 2;
            player.pos.x = home_x;
            player.pos.y = home_y;
            player.pos.map = if need_teleport { home_map } else { map_id };
            player.revive_at_ms = 0;
        }

        let (hp, max_hp, mana, max_mana, id_head, id_body) = scene.players.get(&entity_id)
            .map(|p| (p.hp, p.max_hp, p.mana, p.max_mana, p.id_head, p.id_body))
            .unwrap_or((0, 0, 0, 0, 0, 0));
        let vitals = build_self_vitals(hp, max_hp, mana, max_mana);
        self.send_to_client(sink, vitals).await?;

        let revive_pkt = build_revivir_usuario(entity_id, id_head, id_body);
        let revive_pos = crate::world::Position { map: if need_teleport { home_map } else { map_id }, x: home_x, y: home_y };
        scene.broadcast_in_range(0, &revive_pos, revive_pkt.clone());
        self.send_to_client(sink, revive_pkt).await?;

        let entity_vitals = crate::replication::build_entity_vitals_delta(entity_id, hp, max_hp, mana, max_mana);
        scene.broadcast_in_range(entity_id, &revive_pos, entity_vitals);

        self.send_to_client(sink, build_stop_cast_bar(entity_id)).await?;

        let pos_pkt = build_act_position(entity_id, home_x, home_y);
        self.send_to_client(sink, pos_pkt).await?;

        if need_teleport {
            drop(scene);
            self.do_teleport(entity_id, map_id, home_map, home_x, home_y, sink).await?;
        }

        self.send_to_client(sink, build_console_message("Has resucitado.")).await?;

        Ok(())
    }

    async fn handle_faction_command(
        &self,
        arg: &str,
        entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let map_id = match self.map_id {
            Some(id) => id,
            None => return Ok(()),
        };

        let scene = self.world.get_or_create_scene(map_id);

        match arg {
            "armada" | "caos" => {
                let current = scene.players.get(&entity_id).map(|p| p.faction.clone()).unwrap_or_default();
                if current == arg {
                    self.send_to_client(sink, build_console_message("Ya perteneces a esa facción.")).await?;
                    return Ok(());
                }
                if !current.is_empty() && current != "none" {
                    self.send_to_client(sink, build_console_message("Ya perteneces a una facción. Debes dejarla primero.")).await?;
                    return Ok(());
                }
                let level = scene.players.get(&entity_id).map(|p| p.level).unwrap_or(0);
                if level < 25 {
                    self.send_to_client(sink, build_console_message("Necesitas nivel 25 para unirte a una facción.")).await?;
                    return Ok(());
                }
                let criminal = scene.players.get(&entity_id).map(|p| p.criminal).unwrap_or(false);
                if let Some(mut player) = scene.players.get_mut(&entity_id) {
                    player.faction = arg.to_string();
                    player.faction_rank = 0;
                    player.faction_score = 0;
                }
                let color = get_name_color(criminal, arg, false);
                let color_pkt = build_act_color_name(entity_id, color);
                self.send_to_client(sink, color_pkt.clone()).await?;
                let faction_pos = scene.players.get(&entity_id).map(|p| p.pos.clone());
                if let Some(ref pos) = faction_pos {
                    scene.broadcast_in_range(entity_id, pos, color_pkt);
                } else {
                    scene.broadcast(entity_id, color_pkt);
                }
                let msg = format!("Te has unido a la facción {}!", arg);
                self.send_to_client(sink, build_console_message(&msg)).await?;
            }
            "salir" => {
                let was_in_faction = scene.players.get(&entity_id)
                    .map(|p| p.faction != "none" && !p.faction.is_empty())
                    .unwrap_or(false);
                if !was_in_faction {
                    self.send_to_client(sink, build_console_message("No perteneces a ninguna facción.")).await?;
                    return Ok(());
                }
                let criminal = scene.players.get(&entity_id).map(|p| p.criminal).unwrap_or(false);
                if let Some(mut player) = scene.players.get_mut(&entity_id) {
                    player.faction = "none".to_string();
                    player.faction_rank = 0;
                    player.faction_score = 0;
                }
                let color = get_name_color(criminal, "none", false);
                let color_pkt = build_act_color_name(entity_id, color);
                self.send_to_client(sink, color_pkt.clone()).await?;
                let leave_pos = scene.players.get(&entity_id).map(|p| p.pos.clone());
                if let Some(ref pos) = leave_pos {
                    scene.broadcast_in_range(entity_id, pos, color_pkt);
                } else {
                    scene.broadcast(entity_id, color_pkt);
                }
                self.send_to_client(sink, build_console_message("Has abandonado tu facción.")).await?;
            }
            "info" | "" => {
                if let Some(player) = scene.players.get(&entity_id) {
                    let faction = &player.faction;
                    if faction == "none" || faction.is_empty() {
                        self.send_to_client(sink, build_console_message("No perteneces a ninguna facción. Usa /faccion armada o /faccion caos")).await?;
                    } else {
                        let title = crate::gameplay::factions::get_rank_title(faction, player.faction_rank)
                            .unwrap_or("Sin rango");
                        let msg = format!(
                            "Facción: {} | Rango: {} ({}) | Score: {}",
                            faction, player.faction_rank, title, player.faction_score
                        );
                        self.send_to_client(sink, build_console_message(&msg)).await?;
                    }
                }
            }
            _ => {
                self.send_to_client(sink, build_console_message("Uso: /faccion [armada|caos|salir|info]")).await?;
            }
        }

        Ok(())
    }

    pub(super) async fn handle_toggle_safe(
        &mut self,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let entity_id = match self.entity_id {
            Some(e) => e,
            None => return Ok(()),
        };
        let map_id = match self.map_id {
            Some(m) => m,
            None => return Ok(()),
        };

        let scene = self.world.get_or_create_scene(map_id);
        let (new_state, pk) = if let Some(mut p) = scene.players.get_mut(&entity_id) {
            p.seguro_activado = !p.seguro_activado;
            let zona = self.world.gd().maps_meta.get(&map_id).map(|m| m.pk).unwrap_or(0) as u8;
            (p.seguro_activado, zona)
        } else {
            return Ok(());
        };

        let clan_safe = scene.players.get(&entity_id).map(|p| p.seguro_clan_activado).unwrap_or(false);
        self.send_to_client(sink, build_self_flags(pk, new_state, clan_safe)).await?;

        let msg = if new_state { "Seguro activado" } else { "Seguro desactivado" };
        self.send_to_client(sink, build_console_message(msg)).await?;

        Ok(())
    }

    async fn handle_party_chat(
        &self,
        message: &str,
        sender_eid: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        if self.world.muted_players.contains_key(&sender_eid) {
            self.send_to_client(sink, build_console_message("Estas muteado.")).await?;
            return Ok(());
        }
        let name = self.character_name.as_deref().unwrap_or("???");
        let text = message.trim();
        if text.is_empty() {
            return Ok(());
        }

        let map_id = match self.map_id {
            Some(m) => m,
            None => return Ok(()),
        };

        let scene = self.world.get_or_create_scene(map_id);
        let party_id = match scene.players.get(&sender_eid) {
            Some(p) => p.party_id.clone(),
            None => None,
        };

        let party_id = match party_id {
            Some(pid) => pid,
            None => {
                self.send_to_client(sink, build_console_message("No perteneces a ninguna party.")).await?;
                return Ok(());
            }
        };

        let chat_msg = format!("[Party] {}: {}", name, text);
        let pkt = build_console_message(&chat_msg);

        if let Some(party) = self.world.parties.get(&party_id) {
            for &member_eid in &party.member_ids {
                for scene_entry in self.world.scenes.iter() {
                    scene_entry.send_to_player(member_eid, pkt.clone());
                }
            }
        }

        Ok(())
    }

    async fn handle_clan_chat(
        &self,
        message: &str,
        sender_eid: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        if self.world.muted_players.contains_key(&sender_eid) {
            self.send_to_client(sink, build_console_message("Estas muteado.")).await?;
            return Ok(());
        }
        let name = self.character_name.as_deref().unwrap_or("???");
        let text = message.trim();
        if text.is_empty() {
            return Ok(());
        }

        let map_id = match self.map_id {
            Some(m) => m,
            None => return Ok(()),
        };

        let scene = self.world.get_or_create_scene(map_id);
        let clan_id = match scene.players.get(&sender_eid) {
            Some(p) => p.clan_id.clone(),
            None => None,
        };

        let clan_id = match clan_id {
            Some(cid) => cid,
            None => {
                self.send_to_client(sink, build_console_message("No perteneces a ningun clan.")).await?;
                return Ok(());
            }
        };

        let chat_msg = format!("[Clan] {}: {}", name, text);
        let pkt = build_console_message(&chat_msg);

        if let Some(clan) = self.world.clans.get(&clan_id) {
            for &member_eid in &clan.member_ids {
                for scene_entry in self.world.scenes.iter() {
                    scene_entry.send_to_player(member_eid, pkt.clone());
                }
            }
        }

        Ok(())
    }

    async fn handle_hogar(
        &self,
        entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let map_id = match self.map_id {
            Some(m) => m,
            None => return Ok(()),
        };

        let scene = self.world.get_or_create_scene(map_id);
        if let Some(p) = scene.players.get(&entity_id) {
            let home_map = p.home_map;
            let home_x = p.home_x;
            let home_y = p.home_y;
            let msg = format!("Tu hogar esta en mapa {}, posicion ({}, {})", home_map, home_x, home_y);
            self.send_to_client(sink, build_console_message(&msg)).await?;
        }

        Ok(())
    }

    async fn handle_safe_logout(
        &self,
        entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let map_id = match self.map_id {
            Some(m) => m,
            None => return Ok(()),
        };

        let scene = self.world.get_or_create_scene(map_id);
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        if let Some(p) = scene.players.get(&entity_id) {
            if p.logout_expires_at_ms > now_ms {
                self.send_to_client(sink, build_console_message(
                    "[Servidor] Debes permanecer quieto durante 10 segundos para salir. Si te mueves, la salida se cancelará."
                )).await?;
                return Ok(());
            }
            if !p.dead && (p.paralizado || p.inmovilizado) {
                self.send_to_client(sink, build_console_message(
                    "[Servidor] No puedes salir mientras estás paralizado o inmovilizado."
                )).await?;
                return Ok(());
            }
        }

        let is_safe_zone = self.world.gd().get_map_meta(map_id)
            .map(|m| m.pk != 0)
            .unwrap_or(false);
        let is_dead = scene.players.get(&entity_id).map(|p| p.dead).unwrap_or(false);

        if is_safe_zone || is_dead {
            self.send_to_client(sink, build_console_message("[Servidor] Cerrando sesión...")).await?;
            if let Some(mut p) = scene.players.get_mut(&entity_id) {
                p.logout_expires_at_ms = 1;
            }
            return Ok(());
        }

        if let Some(mut p) = scene.players.get_mut(&entity_id) {
            p.logout_expires_at_ms = now_ms + crate::gameplay::combat_formulas::UNSAFE_LOGOUT_DELAY_MS;
        }
        self.send_to_client(sink, build_console_message(
            "[Servidor] Debes permanecer quieto durante 10 segundos para salir. Si te mueves, la salida se cancelará."
        )).await?;

        Ok(())
    }

    async fn handle_fianza(
        &self,
        arg: &str,
        entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let map_id = match self.map_id {
            Some(m) => m,
            None => return Ok(()),
        };

        let scene = self.world.get_or_create_scene(map_id);
        let player_pos = scene.players.get(&entity_id).map(|p| (p.pos.x, p.pos.y));
        let (px, py) = player_pos.unwrap_or((50, 50));
        if !self.world.gd().is_safe_position(map_id, px, py) {
            self.send_to_client(sink, build_console_message("Solo puedes usar /fianza en zona segura.")).await?;
            return Ok(());
        }

        let scene = self.world.get_or_create_scene(map_id);
        let is_criminal = scene.players.get(&entity_id).map(|p| p.criminal).unwrap_or(false);

        if !is_criminal {
            self.send_to_client(sink, build_console_message("Ya eres ciudadano.")).await?;
            return Ok(());
        }

        const BAIL_COST_PER_CITIZEN: i32 = 5000;
        let (gold, ciudadanos, bail_paid) = scene.players.get(&entity_id)
            .map(|p| (p.gold, p.ciudadanos_matados, 0i32))
            .unwrap_or((0, 0, 0));

        let bail_kills = if ciudadanos > bail_paid { ciudadanos } else { 0 };
        let bail_cost = if bail_kills > 0 {
            bail_kills * crate::gameplay::combat_formulas::NPC_GOLD_MULTIPLIER * BAIL_COST_PER_CITIZEN
        } else {
            BAIL_COST_PER_CITIZEN / 2
        };

        if arg != "pagar" {
            let can_pay = gold >= bail_cost;
            let pkt = crate::gateway::packets::build_open_bail(0, ciudadanos, 1, bail_cost, gold, can_pay);
            self.send_to_client(sink, pkt).await?;
            return Ok(());
        }

        if gold < bail_cost {
            let msg = format!("Necesitas {} de oro para pagar tu fianza. Tienes {}.", bail_cost, gold);
            self.send_to_client(sink, build_console_message(&msg)).await?;
            return Ok(());
        }

        let faction = scene.players.get(&entity_id).map(|p| p.faction.clone()).unwrap_or_default();
        if let Some(mut player) = scene.players.get_mut(&entity_id) {
            player.gold = crate::gameplay::balance::clamp_gold((player.gold - bail_cost) as i64) as i32;
            player.criminal = false;
            let gold_pkt = crate::gateway::packets::build_act_gold(player.gold);
            drop(player);
            self.send_to_client(sink, gold_pkt).await?;
        }
        let color = get_name_color(false, &faction, false);
        let color_pkt = build_act_color_name(entity_id, color);
        self.send_to_client(sink, color_pkt.clone()).await?;
        let bail_pos = scene.players.get(&entity_id).map(|p| p.pos.clone());
        if let Some(ref pos) = bail_pos {
            scene.broadcast_in_range(entity_id, pos, color_pkt);
        } else {
            scene.broadcast(entity_id, color_pkt);
        }

        let msg = format!("Has pagado {} de oro y vuelves a ser ciudadano.", bail_cost);
        self.send_to_client(sink, build_console_message(&msg)).await?;
        Ok(())
    }

    async fn handle_asignar_hogar(
        &self,
        entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let map_id = match self.map_id {
            Some(m) => m,
            None => return Ok(()),
        };

        let scene = self.world.get_or_create_scene(map_id);
        if let Some(mut player) = scene.players.get_mut(&entity_id) {
            player.home_map = player.pos.map;
            player.home_x = player.pos.x;
            player.home_y = player.pos.y;
            drop(player);
            self.send_to_client(sink, build_console_message("Tu hogar fue asignado a esta posicion.")).await?;
        }
        Ok(())
    }

    async fn handle_enlistar(
        &self,
        entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let map_id = match self.map_id {
            Some(m) => m,
            None => return Ok(()),
        };

        let scene = self.world.get_or_create_scene(map_id);
        let (faction, criminal) = match scene.players.get(&entity_id) {
            Some(p) => (p.faction.clone(), p.criminal),
            None => return Ok(()),
        };

        if faction != "none" && !faction.is_empty() {
            let msg = format!("Ya perteneces a la faccion {}. Usa /faccion salir primero.", faction);
            self.send_to_client(sink, build_console_message(&msg)).await?;
            return Ok(());
        }

        if criminal {
            self.send_to_client(sink, build_console_message("Debes ser ciudadano para enlistarte en la Armada. Si quieres unirte al Caos, usa /faccion caos.")).await?;
        } else {
            self.send_to_client(sink, build_console_message("Usa /faccion armada o /faccion caos para enlistarte.")).await?;
        }
        Ok(())
    }

    async fn handle_recompensa(
        &self,
        entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let map_id = match self.map_id {
            Some(m) => m,
            None => return Ok(()),
        };

        let scene = self.world.get_or_create_scene(map_id);
        let (faction, faction_rank, faction_score) = match scene.players.get(&entity_id) {
            Some(p) => (p.faction.clone(), p.faction_rank, p.faction_score),
            None => return Ok(()),
        };

        if faction == "none" || faction.is_empty() {
            self.send_to_client(sink, build_console_message("No perteneces a ninguna faccion.")).await?;
            return Ok(());
        }

        let msg = format!(
            "Faccion: {} | Rango: {} | Puntos: {}",
            faction, faction_rank, faction_score
        );
        self.send_to_client(sink, build_console_message(&msg)).await?;
        Ok(())
    }

    async fn handle_admin_give_exp(
        &mut self,
        args: &str,
        entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let amount: i32 = args.trim().parse().unwrap_or(0);
        if amount <= 0 {
            self.send_to_client(sink, build_console_message("Uso: /darexp cantidad")).await?;
            return Ok(());
        }
        let map_id = match self.map_id {
            Some(m) => m,
            None => return Ok(()),
        };
        let scene = self.world.get_or_create_scene(map_id);
        if let Some(mut player) = scene.players.get_mut(&entity_id) {
            player.exp += amount;
            while player.exp >= player.exp_next_level {
                player.exp -= player.exp_next_level;
                player.level += 1;
                player.exp_next_level = (player.exp_next_level as f32 * 1.15) as i32;
                player.max_hp += 15;
                player.hp = player.max_hp;
                player.max_mana += 10;
                player.mana = player.max_mana;
            }
            let vitals = build_self_vitals(player.hp, player.max_hp, player.mana, player.max_mana);
            drop(player);
            self.send_to_client(sink, vitals).await?;
        }
        let scene = self.world.get_or_create_scene(map_id);
        if let Some(player) = scene.players.get(&entity_id) {
            self.send_to_client(sink, build_act_exp(player.exp, player.exp_next_level)).await?;
            self.send_to_client(sink, build_act_level(player.level)).await?;
        }
        self.send_to_client(sink, build_console_message(&format!("Recibiste {} de experiencia.", amount))).await?;
        Ok(())
    }

    async fn handle_admin_give_gold(
        &mut self,
        args: &str,
        entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let amount: i32 = args.trim().parse().unwrap_or(0);
        if amount <= 0 {
            self.send_to_client(sink, build_console_message("Uso: /daroro cantidad")).await?;
            return Ok(());
        }
        let map_id = match self.map_id {
            Some(m) => m,
            None => return Ok(()),
        };
        let scene = self.world.get_or_create_scene(map_id);
        if let Some(mut player) = scene.players.get_mut(&entity_id) {
            player.gold = crate::gameplay::balance::clamp_gold((player.gold + amount) as i64) as i32;
            let gold = player.gold;
            drop(player);
            self.send_to_client(sink, build_act_gold(gold)).await?;
        }
        self.send_to_client(sink, build_console_message(&format!("Recibiste {} de oro.", amount))).await?;
        Ok(())
    }

    async fn handle_admin_kick(
        &mut self,
        args: &str,
        _entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let target_name = args.trim();
        if target_name.is_empty() {
            self.send_to_client(sink, build_console_message("Uso: /kick nombre")).await?;
            return Ok(());
        }
        let mut found = false;
        for scene_ref in self.world.scenes.iter() {
            for entry in scene_ref.players.iter() {
                if entry.value().name.eq_ignore_ascii_case(target_name) {
                    if let Some(tx) = scene_ref.personal_tx.get(&entry.value().id) {
                        let _ = tx.send(build_console_message("Has sido expulsado del servidor."));
                    }
                    found = true;
                    break;
                }
            }
            if found { break; }
        }
        if found {
            self.send_to_client(sink, build_console_message(&format!("{} ha sido expulsado.", target_name))).await?;
        } else {
            self.send_to_client(sink, GameError::player_not_found(target_name).to_console_packet()).await?;
        }
        Ok(())
    }

    async fn handle_admin_telepme(
        &mut self,
        entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let map_id = match self.map_id {
            Some(m) => m,
            None => return Ok(()),
        };
        let scene = self.world.get_or_create_scene(map_id);
        if let Some(player) = scene.players.get(&entity_id) {
            let msg = format!("Posicion: Mapa {} ({}, {})", player.pos.map, player.pos.x, player.pos.y);
            drop(player);
            self.send_to_client(sink, build_console_message(&msg)).await?;
        }
        Ok(())
    }

    async fn handle_admin_telepuser(
        &mut self,
        args: &str,
        _entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let parts: Vec<&str> = args.split_whitespace().collect();
        if parts.len() < 4 {
            self.send_to_client(sink, build_console_message("Uso: /telepuser nombre mapa x y")).await?;
            return Ok(());
        }
        let target_name = parts[0];
        let target_map: i32 = parts[1].parse().unwrap_or(0);
        let target_x: i32 = parts[2].parse().unwrap_or(0);
        let target_y: i32 = parts[3].parse().unwrap_or(0);

        let mut target_entity: Option<(u32, i32)> = None;
        for scene_ref in self.world.scenes.iter() {
            for entry in scene_ref.players.iter() {
                if entry.value().name.eq_ignore_ascii_case(target_name) {
                    target_entity = Some((entry.value().id, entry.value().pos.map));
                    break;
                }
            }
            if target_entity.is_some() { break; }
        }

        if let Some((tid, old_map)) = target_entity {
            self.do_teleport(tid, old_map, target_map, target_x, target_y, sink).await?;
            self.send_to_client(sink, build_console_message(&format!("{} teletransportado a ({}, {}, {}).", target_name, target_map, target_x, target_y))).await?;
        } else {
            self.send_to_client(sink, GameError::player_not_found(target_name).to_console_packet()).await?;
        }
        Ok(())
    }

    async fn handle_admin_bring(
        &mut self,
        args: &str,
        entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let target_name = args.trim();
        if target_name.is_empty() {
            self.send_to_client(sink, build_console_message("Uso: /traer nombre")).await?;
            return Ok(());
        }
        let my_pos = {
            let map_id = match self.map_id {
                Some(m) => m,
                None => return Ok(()),
            };
            let scene = self.world.get_or_create_scene(map_id);
            scene.players.get(&entity_id).map(|p| p.pos.clone())
        };

        let my_pos = match my_pos {
            Some(p) => p,
            None => return Ok(()),
        };

        let mut target_entity: Option<(u32, i32)> = None;
        for scene_ref in self.world.scenes.iter() {
            for entry in scene_ref.players.iter() {
                if entry.value().name.eq_ignore_ascii_case(target_name) {
                    target_entity = Some((entry.value().id, entry.value().pos.map));
                    break;
                }
            }
            if target_entity.is_some() { break; }
        }

        if let Some((tid, old_map)) = target_entity {
            self.do_teleport(tid, old_map, my_pos.map, my_pos.x, my_pos.y, sink).await?;
            self.send_to_client(sink, build_console_message(&format!("{} traido a tu posicion.", target_name))).await?;
        } else {
            self.send_to_client(sink, GameError::player_not_found(target_name).to_console_packet()).await?;
        }
        Ok(())
    }

    async fn handle_admin_revive(
        &mut self,
        entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let map_id = match self.map_id {
            Some(m) => m,
            None => return Ok(()),
        };
        let scene = self.world.get_or_create_scene(map_id);
        if let Some(mut player) = scene.players.get_mut(&entity_id) {
            player.dead = false;
            player.dead_world_active = false;
            player.dead_world_transition_at_ms = 0;
            player.hp = player.max_hp;
            player.mana = player.max_mana;
            let (hp, max_hp, mana, max_mana) = (player.hp, player.max_hp, player.mana, player.max_mana);
            let (id_head, id_body) = (player.id_head, player.id_body);
            let player_pos = player.pos.clone();
            drop(player);

            let vitals = build_self_vitals(hp, max_hp, mana, max_mana);
            self.send_to_client(sink, vitals).await?;

            let entity_vitals = crate::replication::build_entity_vitals_delta(entity_id, hp, max_hp, mana, max_mana);
            scene.broadcast_in_range(entity_id, &player_pos, entity_vitals);

            let revive_pkt = build_revivir_usuario(entity_id, id_head, id_body);
            scene.broadcast_in_range(0, &player_pos, revive_pkt.clone());
            self.send_to_client(sink, revive_pkt).await?;
        }
        self.send_to_client(sink, build_console_message("Revivido.")).await?;
        Ok(())
    }

    async fn handle_admin_create_item(
        &mut self,
        args: &str,
        _entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let parts: Vec<&str> = args.split_whitespace().collect();
        if parts.is_empty() {
            self.send_to_client(sink, build_console_message("Uso: /crearitem id [cantidad]")).await?;
            return Ok(());
        }
        let item_id: i32 = parts[0].parse().unwrap_or(0);
        let amount: i32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);

        if item_id <= 0 || amount <= 0 {
            self.send_to_client(sink, build_console_message("ID o cantidad invalida.")).await?;
            return Ok(());
        }

        let char_id = match &self.character_id {
            Some(c) => c.clone(),
            None => return Ok(()),
        };

        self.world.cache_add_item(&char_id, item_id, amount);
        self.send_full_inventory(sink).await?;

        let item_data = crate::replication::get_item_data(&self.world.gd(), item_id);
        self.send_to_client(sink, build_console_message(&format!("Creado {}x {} (ID: {}).", amount, item_data.name, item_id))).await?;
        Ok(())
    }

    async fn handle_admin_ban(
        &mut self,
        args: &str,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let target_name = args.trim();
        if target_name.is_empty() {
            self.send_to_client(sink, build_console_message("Uso: /ban nombre")).await?;
            return Ok(());
        }
        let mut target_account: Option<String> = None;
        for scene_ref in self.world.scenes.iter() {
            for entry in scene_ref.players.iter() {
                if entry.value().name.eq_ignore_ascii_case(target_name) {
                    target_account = Some(entry.value().character_id.clone());
                    if let Some(tx) = scene_ref.personal_tx.get(&entry.value().id) {
                        let _ = tx.send(build_console_message("Has sido baneado."));
                    }
                    break;
                }
            }
            if target_account.is_some() { break; }
        }
        if let Some(acc) = &target_account {
            self.world.banned_accounts.insert(acc.clone(), target_name.to_string());
            let admin_name = self.character_name.as_deref().unwrap_or("admin");
            let _ = self.world.db.add_ban(acc, "banned by admin", admin_name).await;
            self.send_to_client(sink, build_console_message(&format!("{} ha sido baneado.", target_name))).await?;
        } else {
            self.send_to_client(sink, GameError::player_not_found(target_name).to_console_packet()).await?;
        }
        Ok(())
    }

    async fn handle_admin_unban(
        &mut self,
        args: &str,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let target_name = args.trim();
        if target_name.is_empty() {
            self.send_to_client(sink, build_console_message("Uso: /unban nombre")).await?;
            return Ok(());
        }
        let mut removed = false;
        let mut removed_key = None;
        self.world.banned_accounts.retain(|k, v| {
            if v.eq_ignore_ascii_case(target_name) { removed = true; removed_key = Some(k.clone()); false } else { true }
        });
        if removed {
            if let Some(ref key) = removed_key {
                let _ = self.world.db.remove_ban(key).await;
            }
            self.send_to_client(sink, build_console_message(&format!("{} desbaneado.", target_name))).await?;
        } else {
            self.send_to_client(sink, build_console_message(&format!("{} no estaba baneado.", target_name))).await?;
        }
        Ok(())
    }

    async fn handle_admin_mute(
        &mut self,
        args: &str,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let target_name = args.trim();
        if target_name.is_empty() {
            self.send_to_client(sink, build_console_message("Uso: /mute nombre")).await?;
            return Ok(());
        }
        let mut target_info: Option<(u32, String)> = None;
        for scene_ref in self.world.scenes.iter() {
            for entry in scene_ref.players.iter() {
                if entry.value().name.eq_ignore_ascii_case(target_name) {
                    target_info = Some((entry.value().id, entry.value().character_id.clone()));
                    break;
                }
            }
            if target_info.is_some() { break; }
        }
        if let Some((tid, char_id)) = target_info {
            let was_muted = self.world.muted_players.contains_key(&tid);
            if was_muted {
                self.world.muted_players.remove(&tid);
                let _ = self.world.db.remove_mute(&char_id).await;
                self.send_to_client(sink, build_console_message(&format!("{} desmuteado.", target_name))).await?;
            } else {
                self.world.muted_players.insert(tid, true);
                let admin_name = self.character_name.as_deref().unwrap_or("admin");
                let _ = self.world.db.add_mute(&char_id, "muted by admin", admin_name).await;
                self.send_to_client(sink, build_console_message(&format!("{} muteado.", target_name))).await?;
            }
        } else {
            self.send_to_client(sink, GameError::player_not_found(target_name).to_console_packet()).await?;
        }
        Ok(())
    }

    async fn handle_admin_global_gm(
        &mut self,
        msg: &str,
        sink: &mut WsSink,
    ) -> HandlerResult {
        if msg.is_empty() {
            self.send_to_client(sink, build_console_message("Uso: /globalgm mensaje")).await?;
            return Ok(());
        }
        let mut pw = PacketWriter::with_packet_id(client_packet_id::GLOBAL_NOTICE);
        pw.write_string(msg);
        let pkt = pw.into_bytes();
        for scene_ref in self.world.scenes.iter() {
            scene_ref.broadcast(0, pkt.clone());
        }
        Ok(())
    }

    async fn handle_admin_world_save(
        &mut self,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let mut saved = 0u32;
        let mut all_players: Vec<crate::world::PlayerState> = Vec::new();
        for scene_ref in self.world.scenes.iter() {
            for entry in scene_ref.players.iter() {
                all_players.push(entry.value().clone());
            }
        }

        if let Ok(mut tx) = self.world.db.begin_transaction().await {
            for p in &all_players {
                let bank_gold = self.world.db.get_bank_gold(&p.character_id).await.unwrap_or(0);
                let _ = crate::persistence::Database::save_character_state_in_tx(
                    &mut tx,
                    &p.character_id,
                    p.pos.map, p.pos.x, p.pos.y,
                    p.hp, p.max_hp, p.mana, p.max_mana,
                    p.gold, p.level, p.exp, p.exp_next_level,
                    p.dead, &p.faction, p.criminal,
                    p.min_hit, p.max_hit,
                    p.attr_fuerza, p.attr_agilidad,
                    p.attr_inteligencia, p.attr_constitucion,
                    p.home_map, p.home_x, p.home_y,
                    p.id_head, p.id_body, p.id_helmet,
                    p.id_weapon, p.id_shield,
                    p.id_arrow_slot, p.id_ring_slot,
                    p.navegando, bank_gold,
                    p.id_clase, p.faction_rank, p.faction_score,
                    p.faction_score_armada, p.faction_score_caos,
                    p.criminales_matados, p.ciudadanos_matados,
                ).await;
                saved += 1;
            }
            let _ = tx.commit().await;
        }

        for p in &all_players {
            self.world.cache_flush_inventory(&p.character_id).await;
        }

        self.send_to_client(sink, build_console_message(&format!("Mundo guardado. {} jugadores salvados.", saved))).await?;
        Ok(())
    }

    async fn handle_admin_inspect(
        &mut self,
        args: &str,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let target_name = args.trim();
        if target_name.is_empty() {
            self.send_to_client(sink, build_console_message("Uso: /inspect nombre")).await?;
            return Ok(());
        }
        for scene_ref in self.world.scenes.iter() {
            for entry in scene_ref.players.iter() {
                if entry.value().name.eq_ignore_ascii_case(target_name) {
                    let p = entry.value();
                    let info = format!(
                        "{}: Lvl {} | HP {}/{} | Mana {}/{} | Gold {} | Map {} ({},{}) | Class {} | Criminal: {} | Faction: {}",
                        p.name, p.level, p.hp, p.max_hp, p.mana, p.max_mana, p.gold,
                        p.pos.map, p.pos.x, p.pos.y, p.id_clase, p.criminal, p.faction
                    );
                    self.send_to_client(sink, build_console_message(&info)).await?;
                    return Ok(());
                }
            }
        }
        self.send_to_client(sink, GameError::player_not_found(target_name).to_console_packet()).await?;
        Ok(())
    }

    async fn handle_admin_change_class(
        &mut self,
        args: &str,
        entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let new_class: i32 = args.trim().parse().unwrap_or(0);
        if !(1..=8).contains(&new_class) {
            self.send_to_client(sink, build_console_message("Uso: /cambiarclase [1-8]")).await?;
            return Ok(());
        }
        let map_id = match self.map_id { Some(m) => m, None => return Ok(()) };
        let scene = self.world.get_or_create_scene(map_id);
        if let Some(mut player) = scene.players.get_mut(&entity_id) {
            player.id_clase = new_class;
        }
        self.send_to_client(sink, build_console_message(&format!("Clase cambiada a {}.", new_class))).await?;
        Ok(())
    }

    async fn handle_admin_ban_ip(
        &mut self,
        args: &str,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let ip = args.trim().to_string();
        if ip.is_empty() {
            self.send_to_client(sink, build_console_message("Uso: /banip <ip>")).await?;
            return Ok(());
        }
        let admin_name = self.character_name.as_deref().unwrap_or("admin").to_string();
        self.world.banned_ips.insert(ip.clone(), admin_name.clone());
        let _ = self.world.db.add_ip_ban(&ip, "Admin ban", &admin_name).await;
        self.send_to_client(sink, build_console_message(&format!("IP {} baneada.", ip))).await?;
        Ok(())
    }

    async fn handle_admin_unban_ip(
        &mut self,
        args: &str,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let ip = args.trim().to_string();
        if ip.is_empty() {
            self.send_to_client(sink, build_console_message("Uso: /unbanip <ip>")).await?;
            return Ok(());
        }
        self.world.banned_ips.remove(&ip);
        let _ = self.world.db.remove_ip_ban(&ip).await;
        self.send_to_client(sink, build_console_message(&format!("IP {} desbaneada.", ip))).await?;
        Ok(())
    }

    async fn handle_admin_jail(
        &mut self,
        args: &str,
        _entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let parts: Vec<&str> = args.splitn(2, ' ').collect();
        if parts.is_empty() {
            self.send_to_client(sink, build_console_message("Uso: /carcel nombre minutos")).await?;
            return Ok(());
        }
        let target_name = parts[0];
        let minutes: u64 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(5);
        let jail_map = 200;
        let jail_x = 50;
        let jail_y = 50;

        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let jail_until = now_ms + minutes * 60 * 1000;

        let mut found = false;
        for scene_ref in self.world.scenes.iter() {
            let scene = scene_ref.value();
            let target_eid: Option<(u32, i32)> = scene.players.iter()
                .find(|p| p.value().name.eq_ignore_ascii_case(target_name))
                .map(|p| (p.id, p.pos.map));

            if let Some((eid, old_map)) = target_eid {
                if let Some(mut p) = scene.players.get_mut(&eid) {
                    p.jail_until_ms = jail_until;
                }
                let _ = scene;

                let jail_scene = self.world.get_or_create_scene(jail_map);
                if let Some((_, mut ps)) = self.world.get_or_create_scene(old_map).players.remove(&eid) {
                    let old_scene = self.world.get_or_create_scene(old_map);
                    old_scene.aoi_remove(eid);
                    let del = crate::replication::build_delete_character_packet(eid);
                    old_scene.broadcast(eid, del);

                    ps.pos.map = jail_map;
                    ps.pos.x = jail_x;
                    ps.pos.y = jail_y;
                    ps.jail_until_ms = jail_until;

                    jail_scene.aoi_insert(eid, &ps.pos);
                    jail_scene.players.insert(eid, ps);
                }

                let msg = format!("{} ha sido encarcelado por {} minuto(s).", target_name, minutes);
                jail_scene.send_to_player(eid, build_console_message(&format!("Has sido encarcelado por {} minuto(s).", minutes)));
                self.send_to_client(sink, build_console_message(&msg)).await?;
                found = true;
                break;
            }
        }

        if !found {
            self.send_to_client(sink, build_console_message("Jugador no encontrado.")).await?;
        }
        Ok(())
    }

    async fn handle_admin_invisibility(
        &mut self,
        entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let map_id = match self.map_id { Some(m) => m, None => return Ok(()) };
        let scene = self.world.get_or_create_scene(map_id);
        let Some(mut player) = scene.players.get_mut(&entity_id) else { return Ok(()) };

        player.invisible = !player.invisible;
        let now_invisible = player.invisible;
        let pos = player.pos.clone();
        drop(player);

        if now_invisible {
            let del_pkt = crate::replication::build_delete_character_packet(entity_id);
            scene.broadcast_in_range(entity_id, &pos, del_pkt);
            self.send_to_client(sink, build_console_message("Eres invisible. Los demás jugadores no pueden verte.")).await?;
        } else {
            if let Some(p) = scene.players.get(&entity_id) {
                let announce = crate::replication::build_character_packet(&p);
                scene.broadcast_in_range(entity_id, &pos, announce);
            }
            self.send_to_client(sink, build_console_message("Ya no eres invisible.")).await?;
        }
        Ok(())
    }

    async fn handle_admin_clean_floor(
        &mut self,
        entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let map_id = match self.map_id { Some(m) => m, None => return Ok(()) };
        let scene = self.world.get_or_create_scene(map_id);
        let positions: Vec<(i32, i32)> = scene.ground_items.iter().map(|e| *e.key()).collect();
        let clean_center = scene.players.get(&entity_id).map(|p| p.pos.clone());
        for pos in &positions {
            scene.ground_items.remove(pos);
            let pkt = crate::replication::build_delete_ground_item(pos.0, pos.1);
            if let Some(ref center) = clean_center {
                scene.broadcast_in_range(entity_id, center, pkt.clone());
            } else {
                scene.broadcast(entity_id, pkt.clone());
            }
            self.send_to_client(sink, pkt).await?;
        }
        self.send_to_client(sink, build_console_message(&format!("Piso limpiado. {} items removidos.", positions.len()))).await?;
        Ok(())
    }

    async fn handle_admin_toggle_double_exp(
        &mut self,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let prev = self.world.double_exp.load(std::sync::atomic::Ordering::Relaxed);
        self.world.double_exp.store(!prev, std::sync::atomic::Ordering::Relaxed);
        let state = if !prev { "activado" } else { "desactivado" };
        let msg = format!("Doble experiencia {}.", state);
        let mut pw = PacketWriter::with_packet_id(client_packet_id::GLOBAL_NOTICE);
        pw.write_string(&msg);
        let pkt = pw.into_bytes();
        for scene_ref in self.world.scenes.iter() {
            scene_ref.broadcast(0, pkt.clone());
        }
        self.send_to_client(sink, build_console_message(&msg)).await?;
        Ok(())
    }

    async fn handle_admin_toggle_double_gold(
        &mut self,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let prev = self.world.double_gold.load(std::sync::atomic::Ordering::Relaxed);
        self.world.double_gold.store(!prev, std::sync::atomic::Ordering::Relaxed);
        let state = if !prev { "activado" } else { "desactivado" };
        let msg = format!("Doble oro {}.", state);
        let mut pw = PacketWriter::with_packet_id(client_packet_id::GLOBAL_NOTICE);
        pw.write_string(&msg);
        let pkt = pw.into_bytes();
        for scene_ref in self.world.scenes.iter() {
            scene_ref.broadcast(0, pkt.clone());
        }
        self.send_to_client(sink, build_console_message(&msg)).await?;
        Ok(())
    }

    async fn handle_admin_spawn_npc(
        &mut self,
        args: &str,
        entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let npc_index: i32 = args.trim().parse().unwrap_or(0);
        if npc_index <= 0 {
            self.send_to_client(sink, build_console_message("Uso: /invocarnpc idNpc")).await?;
            return Ok(());
        }
        let gd = self.world.gd();
        let template = match gd.get_npc(npc_index) {
            Some(t) => t,
            None => {
                self.send_to_client(sink, build_console_message("NPC no encontrado en data.")).await?;
                return Ok(());
            }
        };
        let map_id = match self.map_id { Some(m) => m, None => return Ok(()) };
        let scene = self.world.get_or_create_scene(map_id);
        let (px, py) = scene.players.get(&entity_id).map(|p| (p.pos.x, p.pos.y)).unwrap_or((50, 50));

        let npc_id = self.world.next_id();
        let npc_spells: Vec<crate::world::NpcSpellSlot> = template.spells.iter()
            .map(|s| crate::world::NpcSpellSlot { spell_id: s.id_spell })
            .collect();
        let npc_state = crate::world::NpcState {
            id: npc_id,
            npc_type: npc_index,
            pos: crate::world::Position { map: map_id, x: px + 1, y: py },
            heading: 3,
            hp: template.hp.max(template.max_hp),
            max_hp: template.max_hp,
            min_hit: template.min_hit,
            max_hit: template.max_hit,
            defense: template.def,
            exp_reward: template.exp,
            movement: template.movement,
            dead: false,
            paralizado: false,
            inmovilizado: false,
            cc_expire_tick: 0,
            aggro_target: None,
            spells: npc_spells,
            spell_cast_interval_ms: template.spell_cast_interval_ms.unwrap_or(2000),
            last_spell_cast_at: 0,
            spell_range: template.spell_range.unwrap_or(8),
            magic_def: template.magic_def,
            magic_resistance: template.magic_resistance,
            summoned_by: None,
            summon_expires_at_ms: 0,
            admin_bot_owner: None,
        };
        scene.aoi_insert(npc_id, &npc_state.pos);
        let pkt = crate::replication::build_npc_packet(&npc_state, &self.world.gd());
        let spawn_pos = npc_state.pos.clone();
        scene.broadcast_in_range(0, &spawn_pos, pkt.clone());
        self.send_to_client(sink, pkt).await?;
        scene.npcs.insert(npc_id, npc_state);
        self.send_to_client(sink, build_console_message(&format!("NPC {} ({}) invocado.", template.name, npc_index))).await?;
        Ok(())
    }

    async fn handle_admin_reset_map(
        &mut self,
        entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let map_id = match self.map_id { Some(m) => m, None => return Ok(()) };
        let scene = self.world.get_or_create_scene(map_id);
        let reset_center = scene.players.get(&entity_id).map(|p| p.pos.clone());
        let npc_ids: Vec<u32> = scene.npcs.iter().map(|e| *e.key()).collect();
        for nid in &npc_ids {
            let npc_pos = scene.npcs.get(nid).map(|n| n.pos.clone());
            scene.npcs.remove(nid);
            scene.aoi_remove(*nid);
            let pkt = crate::replication::build_delete_character_packet(*nid);
            if let Some(ref pos) = npc_pos {
                scene.broadcast_in_range(entity_id, pos, pkt.clone());
            } else if let Some(ref pos) = reset_center {
                scene.broadcast_in_range(entity_id, pos, pkt.clone());
            } else {
                scene.broadcast(entity_id, pkt.clone());
            }
            self.send_to_client(sink, pkt).await?;
        }
        let gi_keys: Vec<(i32, i32)> = scene.ground_items.iter().map(|e| *e.key()).collect();
        for pos in &gi_keys {
            scene.ground_items.remove(pos);
            let item_pos = crate::world::Position { map: map_id, x: pos.0, y: pos.1 };
            let pkt = crate::replication::build_delete_ground_item(pos.0, pos.1);
            scene.broadcast_in_range(entity_id, &item_pos, pkt.clone());
            self.send_to_client(sink, pkt).await?;
        }
        self.send_to_client(sink, build_console_message(&format!("Mapa {} reseteado. {} NPCs y {} items removidos.", map_id, npc_ids.len(), gi_keys.len()))).await?;
        Ok(())
    }

    async fn handle_admin_remove_npc(
        &mut self,
        entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let map_id = match self.map_id { Some(m) => m, None => return Ok(()) };
        let scene = self.world.get_or_create_scene(map_id);
        let (px, py) = scene.players.get(&entity_id).map(|p| (p.pos.x, p.pos.y)).unwrap_or((50, 50));

        let mut removed_name = None;
        let mut npc_to_remove = None;
        for entry in scene.npcs.iter() {
            let npc = entry.value();
            let dist = (npc.pos.x - px).abs() + (npc.pos.y - py).abs();
            if dist <= 3 {
                npc_to_remove = Some(npc.id);
                removed_name = self.world.gd().get_npc(npc.npc_type).map(|t| t.name.clone());
                break;
            }
        }

        if let Some(npc_id) = npc_to_remove {
            let remove_pos = scene.npcs.get(&npc_id).map(|n| n.pos.clone());
            scene.npcs.remove(&npc_id);
            scene.aoi_remove(npc_id);
            let pkt = crate::replication::build_delete_character_packet(npc_id);
            if let Some(ref pos) = remove_pos {
                scene.broadcast_in_range(0, pos, pkt.clone());
            } else {
                scene.broadcast(0, pkt.clone());
            }
            self.send_to_client(sink, pkt).await?;
            self.send_to_client(sink, build_console_message(&format!("NPC {} removido.", removed_name.unwrap_or_else(|| "???".into())))).await?;
        } else {
            self.send_to_client(sink, build_console_message("No hay NPCs cercanos para remover.")).await?;
        }
        Ok(())
    }

    async fn handle_admin_remove_npc_permanent(
        &mut self,
        entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let map_id = match self.map_id { Some(m) => m, None => return Ok(()) };
        let scene = self.world.get_or_create_scene(map_id);
        let (px, py) = scene.players.get(&entity_id).map(|p| (p.pos.x, p.pos.y)).unwrap_or((50, 50));

        let mut npc_to_remove = None;
        let mut removed_name = None;
        for entry in scene.npcs.iter() {
            let npc = entry.value();
            let dist = (npc.pos.x - px).abs() + (npc.pos.y - py).abs();
            if dist <= 3 {
                npc_to_remove = Some((npc.id, npc.npc_type));
                removed_name = self.world.gd().get_npc(npc.npc_type).map(|t| t.name.clone());
                break;
            }
        }

        if let Some((npc_id, _npc_type)) = npc_to_remove {
            let remove_pos = scene.npcs.get(&npc_id).map(|n| n.pos.clone());
            scene.npcs.remove(&npc_id);
            scene.aoi_remove(npc_id);
            let pkt = crate::replication::build_delete_character_packet(npc_id);
            if let Some(ref pos) = remove_pos {
                scene.broadcast_in_range(0, pos, pkt.clone());
            } else {
                scene.broadcast(0, pkt.clone());
            }
            self.send_to_client(sink, pkt).await?;
            self.send_to_client(sink, build_console_message(&format!(
                "NPC {} removido permanentemente del mapa {}.", removed_name.unwrap_or_else(|| "???".into()), map_id
            ))).await?;
        } else {
            self.send_to_client(sink, build_console_message("No hay NPCs cercanos para remover permanentemente.")).await?;
        }
        Ok(())
    }

    async fn handle_admin_verip(
        &mut self,
        target_name: &str,
        _entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let target_name_lower = target_name.to_lowercase();
        let mut found_ip = None;
        for scene_entry in self.world.scenes.iter() {
            for player_entry in scene_entry.value().players.iter() {
                if player_entry.value().name.to_lowercase() == target_name_lower {
                    found_ip = Some(player_entry.value().client_ip.clone());
                    break;
                }
            }
            if found_ip.is_some() { break; }
        }
        match found_ip {
            Some(ip) => {
                self.send_to_client(sink, build_console_message(&format!(
                    "IP de {}: {}", target_name, ip
                ))).await?;
            }
            None => {
                self.send_to_client(sink, build_console_message(&format!(
                    "Jugador '{}' no encontrado online.", target_name
                ))).await?;
            }
        }
        Ok(())
    }

    async fn handle_admin_packet_stats(&mut self, sink: &mut WsSink) -> HandlerResult {
        let m = &self.metrics;
        let uptime = m.uptime_start.elapsed().as_secs();
        let total_in = m.total_packets_in.load(std::sync::atomic::Ordering::Relaxed);
        let total_out = m.total_packets_out.load(std::sync::atomic::Ordering::Relaxed);
        let rejected = m.packets_rejected.load(std::sync::atomic::Ordering::Relaxed);
        let conns = m.active_connections.load(std::sync::atomic::Ordering::Relaxed);
        let tick = m.current_tick.load(std::sync::atomic::Ordering::Relaxed);
        let cat = &m.packets_by_category;
        self.send_to_client(sink, build_console_message(&format!(
            "--- Estadísticas del servidor ---"
        ))).await?;
        self.send_to_client(sink, build_console_message(&format!(
            "Uptime: {}s | Conexiones: {} | Tick: {}", uptime, conns, tick
        ))).await?;
        self.send_to_client(sink, build_console_message(&format!(
            "Paquetes IN: {} | OUT: {} | Rechazados: {}", total_in, total_out, rejected
        ))).await?;
        self.send_to_client(sink, build_console_message(&format!(
            "Por categoría — Mov:{} Cmbt:{} Diag:{} Inv:{} Com:{} Soc:{} Crf:{} Gat:{} Bnk:{} Mkt:{} Chl:{} Adm:{} Sys:{}",
            cat.movement.load(std::sync::atomic::Ordering::Relaxed),
            cat.combat.load(std::sync::atomic::Ordering::Relaxed),
            cat.dialog.load(std::sync::atomic::Ordering::Relaxed),
            cat.inventory.load(std::sync::atomic::Ordering::Relaxed),
            cat.commerce.load(std::sync::atomic::Ordering::Relaxed),
            cat.social.load(std::sync::atomic::Ordering::Relaxed),
            cat.crafting.load(std::sync::atomic::Ordering::Relaxed),
            cat.gathering.load(std::sync::atomic::Ordering::Relaxed),
            cat.bank.load(std::sync::atomic::Ordering::Relaxed),
            cat.market.load(std::sync::atomic::Ordering::Relaxed),
            cat.challenge.load(std::sync::atomic::Ordering::Relaxed),
            cat.admin.load(std::sync::atomic::Ordering::Relaxed),
            cat.system.load(std::sync::atomic::Ordering::Relaxed),
        ))).await?;
        Ok(())
    }

    async fn handle_embarcar(&mut self, entity_id: u32, sink: &mut WsSink) -> HandlerResult {
        let map_id = match self.map_id {
            Some(id) => id,
            None => return Ok(()),
        };
        let scene = self.world.get_or_create_scene(map_id);
        let Some(mut player) = scene.players.get_mut(&entity_id) else { return Ok(()) };

        if player.navegando {
            drop(player);
            self.send_to_client(sink, build_console_message("Ya estás navegando.")).await?;
            return Ok(());
        }

        let adj = self.world.gd().is_adjacent_to_water(map_id, player.pos.x, player.pos.y);
        if !adj {
            drop(player);
            self.send_to_client(sink, build_console_message("Debes estar junto al agua para embarcar.")).await?;
            return Ok(());
        }

        player.navegando = true;
        let pos = player.pos.clone();
        drop(player);

        self.send_to_client(sink, build_console_message("Has embarcado. Ahora puedes navegar por el agua.")).await?;

        use openao_protocol::opcodes::client_packet_id;
        let boat_body: i32 = 87;
        scene.broadcast_in_range(0, &pos, build_change_equipment(client_packet_id::CHANGE_BODY, entity_id, boat_body));
        Ok(())
    }

    async fn handle_desembarcar(&mut self, entity_id: u32, sink: &mut WsSink) -> HandlerResult {
        let map_id = match self.map_id {
            Some(id) => id,
            None => return Ok(()),
        };
        let scene = self.world.get_or_create_scene(map_id);
        let Some(mut player) = scene.players.get_mut(&entity_id) else { return Ok(()) };

        if !player.navegando {
            drop(player);
            self.send_to_client(sink, build_console_message("No estás navegando.")).await?;
            return Ok(());
        }

        let on_water = self.world.gd().is_water_tile(map_id, player.pos.x, player.pos.y);
        let near_land = !on_water || self.world.gd().is_adjacent_to_water(map_id, player.pos.x, player.pos.y);
        if on_water && !near_land {
            drop(player);
            self.send_to_client(sink, build_console_message("Debes estar cerca de la costa para desembarcar.")).await?;
            return Ok(());
        }

        player.navegando = false;
        let original_body = player.id_body;
        let pos = player.pos.clone();
        drop(player);

        self.send_to_client(sink, build_console_message("Has desembarcado.")).await?;

        use openao_protocol::opcodes::client_packet_id;
        scene.broadcast_in_range(0, &pos, build_change_equipment(client_packet_id::CHANGE_BODY, entity_id, original_body));
        Ok(())
    }

    async fn handle_admin_bot(
        &mut self,
        args: &str,
        entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let parts: Vec<&str> = args.trim().split_whitespace().collect();
        if parts.is_empty() {
            self.send_to_client(sink, build_console_message(
                "[INFO] Uso: /bot NPC_ID NIVEL o /bot limpiar"
            )).await?;
            return Ok(());
        }

        let action = parts[0].to_lowercase();
        if ["limpiar", "clear", "off", "borrar", "desinvocar"].contains(&action.as_str()) {
            return self.handle_admin_remove_bots(entity_id, sink).await;
        }

        let npc_index: i32 = parts[0].parse().unwrap_or(0);
        if npc_index <= 0 {
            self.send_to_client(sink, build_console_message("[INFO] Uso: /bot NPC_ID [NIVEL]")).await?;
            return Ok(());
        }

        let level: i32 = if parts.len() > 1 { parts[1].parse().unwrap_or(1).max(1).min(50) } else { 30 };

        let gd = self.world.gd();
        let template = match gd.get_npc(npc_index) {
            Some(t) => t,
            None => {
                self.send_to_client(sink, build_console_message("NPC no encontrado en data.")).await?;
                return Ok(());
            }
        };

        let map_id = match self.map_id { Some(m) => m, None => return Ok(()) };
        let scene = self.world.get_or_create_scene(map_id);
        let (px, py) = scene.players.get(&entity_id).map(|p| (p.pos.x, p.pos.y)).unwrap_or((50, 50));

        let hp_scale = (level as f64 / 30.0).max(0.5);
        let scaled_hp = ((template.max_hp as f64) * hp_scale) as i32;
        let scaled_min_hit = ((template.min_hit as f64) * hp_scale) as i32;
        let scaled_max_hit = ((template.max_hit as f64) * hp_scale) as i32;

        let npc_id = self.world.next_id();
        let npc_spells: Vec<crate::world::NpcSpellSlot> = template.spells.iter()
            .map(|s| crate::world::NpcSpellSlot { spell_id: s.id_spell })
            .collect();

        let npc_state = crate::world::NpcState {
            id: npc_id,
            npc_type: npc_index,
            pos: crate::world::Position { map: map_id, x: px + 1, y: py },
            heading: 3,
            hp: scaled_hp,
            max_hp: scaled_hp,
            min_hit: scaled_min_hit,
            max_hit: scaled_max_hit,
            defense: template.def,
            exp_reward: 0,
            movement: template.movement,
            dead: false,
            paralizado: false,
            inmovilizado: false,
            cc_expire_tick: 0,
            aggro_target: None,
            spells: npc_spells,
            spell_cast_interval_ms: template.spell_cast_interval_ms.unwrap_or(2000),
            last_spell_cast_at: 0,
            spell_range: template.spell_range.unwrap_or(8),
            magic_def: template.magic_def,
            magic_resistance: template.magic_resistance,
            summoned_by: None,
            summon_expires_at_ms: 0,
            admin_bot_owner: Some(entity_id),
        };

        scene.aoi_insert(npc_id, &npc_state.pos);
        let pkt = crate::replication::build_npc_packet(&npc_state, &self.world.gd());
        let spawn_pos = npc_state.pos.clone();
        scene.broadcast_in_range(0, &spawn_pos, pkt.clone());
        self.send_to_client(sink, pkt).await?;
        scene.npcs.insert(npc_id, npc_state);

        self.send_to_client(sink, build_console_message(
            &format!("[BOT] {} (npc={}, lvl={}, hp={}) invocado.", template.name, npc_index, level, scaled_hp)
        )).await?;
        Ok(())
    }

    async fn handle_admin_remove_bots(
        &mut self,
        entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let mut removed = 0u32;
        for scene_ref in self.world.scenes.iter() {
            let scene = scene_ref.value();
            let bot_ids: Vec<u32> = scene.npcs.iter()
                .filter(|entry| entry.value().admin_bot_owner == Some(entity_id))
                .map(|entry| *entry.key())
                .collect();
            for npc_id in bot_ids {
                if let Some((_, npc)) = scene.npcs.remove(&npc_id) {
                    scene.aoi_remove(npc_id);
                    let pkt = crate::replication::build_delete_character_packet(npc_id);
                    scene.broadcast_in_range(0, &npc.pos, pkt);
                    removed += 1;
                }
            }
        }
        let msg = if removed > 0 {
            format!("[INFO] Desinvocaste {} bot{}.", removed, if removed == 1 { "" } else { "s" })
        } else {
            "[INFO] No tienes bots invocados.".to_string()
        };
        self.send_to_client(sink, build_console_message(&msg)).await?;
        Ok(())
    }

    async fn handle_admin_intervalo(
        &mut self,
        args: &str,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let parts: Vec<&str> = args.trim().split_whitespace().collect();
        if parts.is_empty() {
            self.send_to_client(sink, build_console_message(
                "[INFO] Uso: /intervalo <clave> [valor]\nClaves: melee_ms, range_ms, spell_ms, use_item_ms, dialog_ms, regen_ticks, npc_ai_ticks"
            )).await?;
            return Ok(());
        }

        let key = parts[0].to_lowercase();
        let timings = &self.world.runtime_timings;

        if parts.len() == 1 {
            let current = match key.as_str() {
                "melee_ms" => Some(timings.melee_ms.load(std::sync::atomic::Ordering::Relaxed)),
                "range_ms" => Some(timings.range_ms.load(std::sync::atomic::Ordering::Relaxed)),
                "spell_ms" => Some(timings.spell_ms.load(std::sync::atomic::Ordering::Relaxed)),
                "use_item_ms" => Some(timings.use_item_ms.load(std::sync::atomic::Ordering::Relaxed)),
                "dialog_ms" => Some(timings.dialog_ms.load(std::sync::atomic::Ordering::Relaxed)),
                "regen_ticks" => Some(timings.regen_ticks.load(std::sync::atomic::Ordering::Relaxed)),
                "npc_ai_ticks" => Some(timings.npc_ai_ticks.load(std::sync::atomic::Ordering::Relaxed)),
                _ => None,
            };
            match current {
                Some(v) => {
                    self.send_to_client(sink, build_console_message(&format!("[INFO] {}={}", key, v))).await?;
                }
                None => {
                    self.send_to_client(sink, build_console_message(
                        "[INFO] Clave no encontrada. Usa /intervalo para ver las claves."
                    )).await?;
                }
            }
            return Ok(());
        }

        let value: u64 = match parts[1].parse() {
            Ok(v) if v > 0 => v,
            _ => {
                self.send_to_client(sink, build_console_message("[INFO] El valor debe ser un entero positivo.")).await?;
                return Ok(());
            }
        };

        let updated = match key.as_str() {
            "melee_ms" => { timings.melee_ms.store(value, std::sync::atomic::Ordering::Relaxed); true }
            "range_ms" => { timings.range_ms.store(value, std::sync::atomic::Ordering::Relaxed); true }
            "spell_ms" => { timings.spell_ms.store(value, std::sync::atomic::Ordering::Relaxed); true }
            "use_item_ms" => { timings.use_item_ms.store(value, std::sync::atomic::Ordering::Relaxed); true }
            "dialog_ms" => { timings.dialog_ms.store(value, std::sync::atomic::Ordering::Relaxed); true }
            "regen_ticks" => { timings.regen_ticks.store(value, std::sync::atomic::Ordering::Relaxed); true }
            "npc_ai_ticks" => { timings.npc_ai_ticks.store(value, std::sync::atomic::Ordering::Relaxed); true }
            _ => false,
        };

        if updated {
            self.send_to_client(sink, build_console_message(&format!("[INFO] Intervalo actualizado: {}={}", key, value))).await?;
        } else {
            self.send_to_client(sink, build_console_message("[INFO] Clave no encontrada.")).await?;
        }
        Ok(())
    }
}
