use openao_protocol::PacketWriter;
use openao_protocol::opcodes::client_packet_id;
use std::collections::HashMap;

use super::packets::build_console_message;
use super::GameSession;
use super::WsSink;
use crate::error::{GameError, GameErrorCode, HandlerResult};

const PUBLICATION_FEE_BPS: i32 = 500;
const DEFAULT_DURATION_HOURS: i32 = 48;
const MAX_DURATION_HOURS: i32 = 168;

impl GameSession {
    pub(super) async fn open_market(
        &self,
        npc_name: &str,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let char_id = match &self.character_id {
            Some(c) => c.clone(),
            None => return Ok(()),
        };

        let _ = self.world.db.expire_market_listings().await;

        let all_listings = self.world.db.get_active_listings().await.unwrap_or_default();
        let my_listings = self.world.db.get_my_listings(&char_id).await.unwrap_or_default();
        let claims = self.world.db.get_claims(&char_id).await.unwrap_or_default();

        let mut groups: HashMap<i32, Vec<&crate::persistence::MarketListingRow>> = HashMap::new();
        for listing in &all_listings {
            groups.entry(listing.item_id).or_default().push(listing);
        }

        let mut listing_groups = Vec::new();
        for (item_id, listings) in &groups {
            let idata = crate::replication::get_item_data(&self.world.gd(), *item_id);
            let total_qty: i32 = listings.iter().map(|l| l.quantity).sum();
            let min_price = listings.iter().map(|l| l.unit_price).min().unwrap_or(0);

            let mut entries = Vec::new();
            for l in listings {
                entries.push(serde_json::json!({
                    "id": l.id,
                    "itemId": l.item_id,
                    "sellerName": l.seller_name,
                    "itemName": idata.name,
                    "itemGrhIndex": idata.grh_index,
                    "quantity": l.quantity,
                    "price": l.unit_price * l.quantity,
                    "status": l.status,
                    "expiresAt": l.expires_at,
                    "createdAt": l.created_at,
                }));
            }

            listing_groups.push(serde_json::json!({
                "itemId": item_id,
                "itemName": idata.name,
                "itemGrhIndex": idata.grh_index,
                "totalListings": listings.len(),
                "totalQuantity": total_qty,
                "minUnitPrice": min_price,
                "listings": entries,
            }));
        }

        let mut my_entries = Vec::new();
        for l in &my_listings {
            let idata = crate::replication::get_item_data(&self.world.gd(), l.item_id);
            my_entries.push(serde_json::json!({
                "id": l.id,
                "itemId": l.item_id,
                "sellerName": l.seller_name,
                "itemName": idata.name,
                "itemGrhIndex": idata.grh_index,
                "quantity": l.quantity,
                "price": l.unit_price * l.quantity,
                "status": l.status,
                "expiresAt": l.expires_at,
                "createdAt": l.created_at,
            }));
        }

        let mut claim_entries = Vec::new();
        for c in &claims {
            let item_name = c.item_id
                .map(|id| crate::replication::get_item_data(&self.world.gd(), id).name.clone());
            let item_grh = c.item_id
                .map(|id| crate::replication::get_item_data(&self.world.gd(), id).grh_index);
            claim_entries.push(serde_json::json!({
                "id": c.id,
                "claimType": c.claim_type,
                "goldAmount": c.gold_amount,
                "itemName": item_name,
                "itemGrhIndex": item_grh,
                "itemQuantity": c.item_quantity,
                "createdAt": c.created_at,
            }));
        }

        let state = serde_json::json!({
            "npcName": npc_name,
            "publicationFeeBps": PUBLICATION_FEE_BPS,
            "defaultDurationHours": DEFAULT_DURATION_HOURS,
            "maxDurationHours": MAX_DURATION_HOURS,
            "hasMoreListings": false,
            "listingGroups": listing_groups,
            "myListings": my_entries,
            "claims": claim_entries,
        });

        let json = serde_json::to_string(&state).unwrap_or_default();
        let mut w = PacketWriter::with_packet_id(client_packet_id::OPEN_MARKET);
        w.write_string(&json);
        self.send_to_client(sink, w.into_bytes()).await?;
        Ok(())
    }

    pub(super) async fn handle_market_action(
        &mut self,
        payload_str: &str,
        sink: &mut WsSink,
    ) -> HandlerResult {
        if !self.command_limiter.check("market") {
            self.send_to_client(sink, GameError::new(GameErrorCode::RateLimited, "Mercado: espera un momento antes de otra acción.").to_console_packet()).await?;
            return Ok(());
        }
        let char_id = match &self.character_id {
            Some(c) => c.clone(),
            None => return Ok(()),
        };
        let entity_id = match self.entity_id {
            Some(e) => e,
            None => return Ok(()),
        };

        let payload: serde_json::Value = serde_json::from_str(payload_str).unwrap_or_default();
        let action = payload.get("action").and_then(|v| v.as_str()).unwrap_or("");

        match action {
            "refresh" => {
                let npc_name = self.market_npc_name.clone().unwrap_or_else(|| "Mercado".to_string());
                self.open_market(&npc_name, sink).await?;
            }
            "create" => {
                let slot = payload.get("slot").and_then(|v| v.as_i64()).unwrap_or(-1) as i32;
                let quantity = payload.get("quantity").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let unit_price = payload.get("unitPrice").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

                if !(0..20).contains(&slot) || quantity <= 0 || unit_price <= 0 {
                    self.send_to_client(sink, GameError::invalid_slot().to_console_packet()).await?;
                    return Ok(());
                }

                let inv = self.world.cache_get_inventory(&char_id);
                let inv_item = inv.iter().find(|i| i.slot == slot);
                let Some(inv_item) = inv_item else {
                    self.send_to_client(sink, GameError::new(GameErrorCode::InvalidSlot, "No tienes un item en ese slot.").to_console_packet()).await?;
                    return Ok(());
                };

                if inv_item.amount < quantity {
                    self.send_to_client(sink, GameError::new(GameErrorCode::InsufficientItems, "No tienes suficiente cantidad.").to_console_packet()).await?;
                    return Ok(());
                }

                let fee = (unit_price as i64 * quantity as i64 * PUBLICATION_FEE_BPS as i64 / 10000) as i32;
                let fee = fee.max(1);

                let map_id = self.map_id.unwrap_or(0);
                let scene = self.world.get_or_create_scene(map_id);
                let player_gold = scene.players.get(&entity_id).map(|p| p.gold).unwrap_or(0);

                if player_gold < fee {
                    self.send_to_client(sink, GameError::insufficient_gold(fee, player_gold).to_console_packet()).await?;
                    return Ok(());
                }

                if let Some(mut p) = scene.players.get_mut(&entity_id) {
                    p.gold = crate::gameplay::balance::clamp_gold((p.gold - fee) as i64) as i32;
                }

                let player_name = scene.players.get(&entity_id).map(|p| p.name.clone()).unwrap_or_default();
                let item_id = inv_item.item_id;

                if inv_item.amount == quantity {
                    self.world.cache_delete_slot(&char_id, slot);
                } else {
                    self.world.cache_update_slot(&char_id, slot, inv_item.item_id, inv_item.amount - quantity, inv_item.equipped);
                }

                self.world.db.create_market_listing(&char_id, &player_name, item_id, quantity, unit_price, DEFAULT_DURATION_HOURS).await?;

                self.send_full_inventory(sink).await?;
                self.send_to_client(sink, build_console_message("Publicación creada exitosamente.")).await?;

                let npc_name = self.market_npc_name.clone().unwrap_or_else(|| "Mercado".to_string());
                self.open_market(&npc_name, sink).await?;
            }
            "buy" => {
                let listing_id = payload.get("listingId").and_then(|v| v.as_str()).unwrap_or("");
                if listing_id.is_empty() {
                    return Ok(());
                }

                let all = self.world.db.get_active_listings().await?;
                let listing = all.iter().find(|l| l.id == listing_id);
                let Some(listing) = listing else {
                    self.send_to_client(sink, GameError::item_not_found("Publicación").to_console_packet()).await?;
                    return Ok(());
                };

                let total_cost = listing.unit_price * listing.quantity;
                let map_id = self.map_id.unwrap_or(0);
                let scene = self.world.get_or_create_scene(map_id);
                let player_gold = scene.players.get(&entity_id).map(|p| p.gold).unwrap_or(0);

                if player_gold < total_cost {
                    self.send_to_client(sink, GameError::insufficient_gold(total_cost, player_gold).to_console_packet()).await?;
                    return Ok(());
                }

                let result = self.world.db.buy_market_listing(listing_id, &char_id).await?;
                if result.is_none() {
                    self.send_to_client(sink, GameError::item_not_found("Publicación").to_console_packet()).await?;
                    return Ok(());
                }

                if let Some(mut p) = scene.players.get_mut(&entity_id) {
                    p.gold = crate::gameplay::balance::clamp_gold((p.gold - total_cost) as i64) as i32;
                }

                self.send_to_client(sink, build_console_message("Compra exitosa. Reclama tus items.")).await?;

                let npc_name = self.market_npc_name.clone().unwrap_or_else(|| "Mercado".to_string());
                self.open_market(&npc_name, sink).await?;
            }
            "cancel" => {
                let listing_id = payload.get("listingId").and_then(|v| v.as_str()).unwrap_or("");
                if listing_id.is_empty() {
                    return Ok(());
                }

                let result = self.world.db.cancel_market_listing(listing_id, &char_id).await?;
                if result.is_none() {
                    self.send_to_client(sink, GameError::item_not_found("Publicación").to_console_packet()).await?;
                    return Ok(());
                }

                self.send_to_client(sink, build_console_message("Publicación cancelada. Reclama tus items.")).await?;

                let npc_name = self.market_npc_name.clone().unwrap_or_else(|| "Mercado".to_string());
                self.open_market(&npc_name, sink).await?;
            }
            "claim" => {
                let claim_id = payload.get("claimId").and_then(|v| v.as_str()).unwrap_or("");
                if claim_id.is_empty() {
                    return Ok(());
                }

                let claim = self.world.db.delete_claim(claim_id, &char_id).await?;
                let Some(claim) = claim else {
                    self.send_to_client(sink, GameError::item_not_found("Reclamo").to_console_packet()).await?;
                    return Ok(());
                };

                if claim.claim_type == "gold" {
                    let map_id = self.map_id.unwrap_or(0);
                    let scene = self.world.get_or_create_scene(map_id);
                    if let Some(mut p) = scene.players.get_mut(&entity_id) {
                        p.gold = crate::gameplay::balance::clamp_gold((p.gold + claim.gold_amount) as i64) as i32;
                    }
                    self.send_to_client(sink, build_console_message(&format!("Reclamaste {} de oro.", claim.gold_amount))).await?;
                } else if claim.claim_type == "item"
                    && let (Some(item_id), Some(qty)) = (claim.item_id, claim.item_quantity) {
                        let added = self.world.cache_add_item(&char_id, item_id, qty);
                        if !added {
                            self.send_to_client(sink, GameError::inventory_full().to_console_packet()).await?;
                            return Ok(());
                        }
                        self.send_full_inventory(sink).await?;
                        self.send_to_client(sink, build_console_message("Items reclamados exitosamente.")).await?;
                    }

                let npc_name = self.market_npc_name.clone().unwrap_or_else(|| "Mercado".to_string());
                self.open_market(&npc_name, sink).await?;
            }
            _ => {
                self.send_to_client(sink, GameError::new(GameErrorCode::NotImplemented, "Acción de mercado no reconocida.").to_console_packet()).await?;
            }
        }

        Ok(())
    }
}
