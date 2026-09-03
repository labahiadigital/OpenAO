use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Deserializer};

fn strip_bom(s: &str) -> &str {
    s.strip_prefix('\u{FEFF}').unwrap_or(s)
}

fn deserialize_i32_or_string<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum I32OrString {
        Int(i32),
        Str(String),
    }
    match I32OrString::deserialize(deserializer)? {
        I32OrString::Int(v) => Ok(v),
        I32OrString::Str(s) => s.parse::<i32>().map_err(serde::de::Error::custom),
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct ObjectData {
    pub name: String,
    #[serde(default)]
    pub obj_type: i32,
    #[serde(default)]
    pub grh_index: u16,
    #[serde(default)]
    pub min_hit: i32,
    #[serde(default)]
    pub max_hit: i32,
    #[serde(default)]
    pub min_def: i32,
    #[serde(default)]
    pub max_def: i32,
    #[serde(default)]
    pub valor: i32,
    #[serde(default)]
    pub anim: u16,
    #[serde(default)]
    pub min_def_mag: i32,
    #[serde(default)]
    pub max_def_mag: i32,
    #[serde(default)]
    pub resistencia_magica: i32,
    #[serde(default)]
    pub magic_damage_bonus: i32,
    #[serde(default)]
    pub magic_penetration: i32,
    #[serde(default)]
    pub staff_damage_bonus: i32,
    #[serde(default)]
    pub spell_index: i32,
    #[serde(default)]
    pub proyectil: i32,
    #[serde(default)]
    pub newbie: i32,
    #[serde(default)]
    pub no_se_cae: i32,
    #[serde(default)]
    pub porcentaje: i32,
    #[serde(default)]
    pub tipo_pocion: i32,
    #[serde(default)]
    pub min_modificador: i32,
    #[serde(default)]
    pub max_modificador: i32,
    #[serde(default)]
    pub clases_no_permitidas: Vec<i32>,
    #[serde(default)]
    pub raza_enana: i32,
    #[serde(default)]
    pub agarrable: i32,
    #[serde(default)]
    pub llave: i32,
    #[serde(default)]
    pub cerrada: i32,
    #[serde(default)]
    pub index_abierta: i32,
    #[serde(default)]
    pub index_cerrada: i32,
    #[serde(default)]
    pub apu: i32,
    #[serde(default)]
    pub tier: Option<i32>,
    #[serde(default)]
    pub travel_ticket_destination: Option<TravelTicketDestination>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TravelTicketDestination {
    pub map: i32,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct NpcTemplate {
    pub name: String,
    #[serde(default)]
    pub npc_type: i32,
    #[serde(default)]
    pub hp: i32,
    #[serde(default)]
    pub max_hp: i32,
    #[serde(default)]
    pub min_hit: i32,
    #[serde(default)]
    pub max_hit: i32,
    #[serde(default)]
    pub def: i32,
    #[serde(default, rename = "defM")]
    pub def_m: i32,
    #[serde(default)]
    pub exp: i32,
    #[serde(default)]
    pub gold: i32,
    #[serde(default)]
    pub id_head: u16,
    #[serde(default)]
    pub id_body: u16,
    #[serde(default)]
    pub movement: i32,
    #[serde(default)]
    pub poder_ataque: i32,
    #[serde(default)]
    pub poder_evasion: i32,
    #[serde(default)]
    pub magic_def: i32,
    #[serde(default)]
    pub magic_resistance: i32,
    #[serde(default)]
    pub agua_valida: i32,
    #[serde(default)]
    pub snd1: u16,
    #[serde(default)]
    pub snd2: u16,
    #[serde(default)]
    pub sound_close: u16,
    #[serde(default)]
    pub drop: Vec<NpcDrop>,
    #[serde(default)]
    pub objs: Vec<NpcShopItem>,
    #[serde(default)]
    pub desc: Option<String>,
    #[serde(default)]
    pub spells: Vec<NpcSpellEntry>,
    #[serde(default, rename = "spellCastIntervalMs")]
    pub spell_cast_interval_ms: Option<u64>,
    #[serde(default, rename = "spellRange")]
    pub spell_range: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NpcSpellEntry {
    pub id_spell: i32,
    #[serde(default)]
    #[allow(dead_code)]
    pub cooldown_seconds: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NpcDrop {
    pub item: i32,
    pub cant: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct NpcShopItem {
    pub item: i32,
    pub cant: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct NpcSpawnEntry {
    #[serde(rename = "mapNum")]
    pub map_num: i32,
    pub x: i32,
    pub y: i32,
    #[serde(rename = "npcIndex")]
    pub npc_index: i32,
    #[serde(default)]
    pub movement: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
struct NpcSpawnWrapper {
    value: Vec<NpcSpawnEntry>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct SpellTemplate {
    pub name: String,
    #[serde(rename = "type")]
    pub spell_type: i32,
    #[serde(default)]
    pub wav: u16,
    #[serde(default)]
    pub fx_grh: u16,
    #[serde(default)]
    pub min_skill: i32,
    #[serde(default)]
    pub mana_required: i32,
    #[serde(default)]
    pub target: i32,
    #[serde(default)]
    pub min_hp: i32,
    #[serde(default)]
    pub max_hp: i32,
    #[serde(default, rename = "subeHp")]
    pub sube_hp: i32,
    #[serde(default, rename = "subeAg")]
    pub sube_ag: i32,
    #[serde(default, rename = "minAg")]
    pub min_ag: i32,
    #[serde(default, rename = "maxAg")]
    pub max_ag: i32,
    #[serde(default, rename = "subeFz")]
    pub sube_fz: i32,
    #[serde(default, rename = "minFz")]
    pub min_fz: i32,
    #[serde(default, rename = "maxFz")]
    pub max_fz: i32,
    #[serde(default)]
    pub paraliza: i32,
    #[serde(default)]
    pub inmoviliza: i32,
    #[serde(default)]
    pub remover_paralisis: i32,
    #[serde(default)]
    pub invisibilidad: i32,
    #[serde(default)]
    pub revivir: i32,
    #[serde(default)]
    pub staff_affected: i32,
    #[serde(default)]
    pub palabras_magicas: Option<String>,
    #[serde(default, rename = "numNpc", deserialize_with = "deserialize_i32_or_string")]
    pub num_npc: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct CraftingRecipe {
    #[serde(default)]
    pub id: i32,
    pub profession: String,
    #[serde(default)]
    pub category: String,
    pub item_id: i32,
    #[serde(default)]
    pub skill: i32,
    #[serde(default)]
    pub materials: Vec<CraftingMaterial>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct CraftingMaterial {
    pub item_id: i32,
    pub amount: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct SmeltingRecipe {
    pub id: i32,
    pub mineral_item_id: i32,
    pub ingot_item_id: i32,
    #[serde(default)]
    pub required_skill: i32,
    #[serde(default)]
    pub minerals_per_ingot: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct MapMeta {
    pub name: String,
    #[serde(default)]
    pub pk: i32,
    #[serde(default, rename = "minLevel")]
    pub min_level: i32,
    #[serde(default, rename = "maxLevel")]
    pub max_level: i32,
}

#[derive(Debug, Clone)]
pub struct TileExit {
    pub target_map: i32,
    pub target_x: i32,
    pub target_y: i32,
}

/// Per-tile info extracted from terrain.json palette.
#[derive(Debug, Clone, Copy, Default)]
#[allow(dead_code)]
pub struct TileInfo {
    pub blocked: bool,
    pub graphic_layer1: u16,
    pub graphic_layer2: u16,
}

/// Map-level tile data loaded from frontend map JSONs.
pub struct MapTileData {
    pub exits: HashMap<(i32, i32), TileExit>,
    pub terrain: Option<MapTerrain>,
    /// Tiles with trigger=6 (per-tile safe zone). Loaded from specials.json `safeTiles`.
    pub safe_tiles: std::collections::HashSet<(i32, i32)>,
}

pub struct MapTerrain {
    pub width: i32,
    pub height: i32,
    tiles: Vec<TileInfo>,
}

impl MapTerrain {
    pub fn get(&self, x: i32, y: i32) -> TileInfo {
        if x < 1 || y < 1 || x > self.width || y > self.height {
            return TileInfo { blocked: true, graphic_layer1: 0, graphic_layer2: 0 };
        }
        let idx = ((y - 1) * self.width + (x - 1)) as usize;
        self.tiles.get(idx).copied().unwrap_or_default()
    }

    pub fn is_water(&self, x: i32, y: i32) -> bool {
        let tile = self.get(x, y);
        is_water_graphic(tile.graphic_layer1) && tile.graphic_layer2 == 0
    }

    pub fn is_blocked(&self, x: i32, y: i32) -> bool {
        self.get(x, y).blocked
    }
}

#[allow(dead_code)]
fn is_water_graphic(grh: u16) -> bool {
    (1505..=1520).contains(&grh)
        || (5665..=5680).contains(&grh)
        || (13547..=13562).contains(&grh)
}

#[allow(dead_code)]
pub struct GameData {
    pub objects: HashMap<i32, ObjectData>,
    pub npcs: HashMap<i32, NpcTemplate>,
    pub npc_spawns: HashMap<i32, Vec<NpcSpawnEntry>>,
    pub spells: HashMap<i32, SpellTemplate>,
    pub crafting_recipes: Vec<CraftingRecipe>,
    pub smelting_recipes: Vec<SmeltingRecipe>,
    pub maps_meta: HashMap<i32, MapMeta>,
    pub tile_data: HashMap<i32, MapTileData>,
    pub quests: crate::gameplay::quests::QuestRegistry,
}

impl GameData {
    pub fn load(data_dir: &Path) -> anyhow::Result<Self> {
        let objects = Self::load_objects(data_dir).map_err(|e| anyhow::anyhow!("Loading objects.json: {e}"))?;
        let npcs = Self::load_npcs(data_dir).map_err(|e| anyhow::anyhow!("Loading npcs.json: {e}"))?;
        let npc_spawns = Self::load_npc_spawns(data_dir).map_err(|e| anyhow::anyhow!("Loading npc_spawns.json: {e}"))?;
        let spells = Self::load_spells(data_dir).map_err(|e| anyhow::anyhow!("Loading spells.json: {e}"))?;
        let crafting_recipes = Self::load_json_array(data_dir, "craftingRecipes.json").map_err(|e| anyhow::anyhow!("Loading craftingRecipes.json: {e}"))?;
        let smelting_recipes = Self::load_json_array(data_dir, "smeltingRecipes.json").map_err(|e| anyhow::anyhow!("Loading smeltingRecipes.json: {e}"))?;
        let maps_meta = Self::load_maps_meta(data_dir).map_err(|e| anyhow::anyhow!("Loading maps_meta.json: {e}"))?;

        let maps_source_dir = data_dir.join("..").join("..").join("server").join("mapas_source");
        let tile_data = Self::load_tile_exits(&maps_source_dir);

        let total_exits: usize = tile_data.values().map(|m| m.exits.len()).sum();
        tracing::info!(
            "Game data loaded: {} objects, {} NPCs, {} maps with spawns, {} spells, {} crafting, {} smelting, {} map meta, {} maps with tile data ({} exits)",
            objects.len(), npcs.len(), npc_spawns.len(), spells.len(),
            crafting_recipes.len(), smelting_recipes.len(), maps_meta.len(),
            tile_data.len(), total_exits
        );

        let quests = Self::load_quests(data_dir);

        Ok(Self { objects, npcs, npc_spawns, spells, crafting_recipes, smelting_recipes, maps_meta, tile_data, quests })
    }

    fn load_quests(data_dir: &Path) -> crate::gameplay::quests::QuestRegistry {
        let path = data_dir.join("quests.json");
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                match crate::gameplay::quests::QuestRegistry::load(strip_bom(&content)) {
                    Ok(reg) => {
                        tracing::info!("Loaded {} quests from quests.json", reg.quests.len());
                        reg
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse quests.json: {e}, starting with empty quest registry");
                        crate::gameplay::quests::QuestRegistry::default()
                    }
                }
            }
            Err(_) => {
                tracing::info!("No quests.json found, starting with empty quest registry");
                crate::gameplay::quests::QuestRegistry::default()
            }
        }
    }

    fn load_objects(data_dir: &Path) -> anyhow::Result<HashMap<i32, ObjectData>> {
        let path = data_dir.join("objects.json");
        let content = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", path.display(), e))?;
        let raw: HashMap<String, ObjectData> = serde_json::from_str(strip_bom(&content))?;
        Ok(raw.into_iter()
            .filter_map(|(k, v)| k.parse::<i32>().ok().map(|id| (id, v)))
            .collect())
    }

    fn load_npcs(data_dir: &Path) -> anyhow::Result<HashMap<i32, NpcTemplate>> {
        let path = data_dir.join("npcs.json");
        let content = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", path.display(), e))?;
        let raw: HashMap<String, NpcTemplate> = serde_json::from_str(strip_bom(&content))?;
        Ok(raw.into_iter()
            .filter_map(|(k, v)| k.parse::<i32>().ok().map(|id| (id, v)))
            .collect())
    }

    fn load_npc_spawns(data_dir: &Path) -> anyhow::Result<HashMap<i32, Vec<NpcSpawnEntry>>> {
        let path = data_dir.join("npc_spawns.json");
        let content = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", path.display(), e))?;
        let raw: HashMap<String, NpcSpawnWrapper> = serde_json::from_str(strip_bom(&content))?;
        Ok(raw.into_iter()
            .filter_map(|(k, v)| k.parse::<i32>().ok().map(|id| (id, v.value)))
            .collect())
    }

    fn load_spells(data_dir: &Path) -> anyhow::Result<HashMap<i32, SpellTemplate>> {
        let path = data_dir.join("spells.json");
        let content = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", path.display(), e))?;
        let raw: HashMap<String, SpellTemplate> = serde_json::from_str(strip_bom(&content))?;
        Ok(raw.into_iter()
            .filter_map(|(k, v)| k.parse::<i32>().ok().map(|id| (id, v)))
            .collect())
    }

    fn load_json_array<T: serde::de::DeserializeOwned>(data_dir: &Path, filename: &str) -> anyhow::Result<Vec<T>> {
        let path = data_dir.join(filename);
        if !path.exists() {
            tracing::warn!("Data file not found, skipping: {}", path.display());
            return Ok(vec![]);
        }
        let content = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", path.display(), e))?;
        let data: Vec<T> = serde_json::from_str(strip_bom(&content))?;
        Ok(data)
    }

    fn load_maps_meta(data_dir: &Path) -> anyhow::Result<HashMap<i32, MapMeta>> {
        let path = data_dir.join("maps_meta.json");
        if !path.exists() {
            tracing::warn!("maps_meta.json not found, skipping");
            return Ok(HashMap::new());
        }
        let content = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", path.display(), e))?;
        let raw: HashMap<String, MapMeta> = serde_json::from_str(strip_bom(&content))?;
        Ok(raw.into_iter()
            .filter_map(|(k, v)| k.parse::<i32>().ok().map(|id| (id, v)))
            .collect())
    }

    fn load_tile_exits(maps_dir: &Path) -> HashMap<i32, MapTileData> {
        let mut result = HashMap::new();
        let Ok(entries) = std::fs::read_dir(maps_dir) else {
            tracing::warn!("Map source directory not found: {}", maps_dir.display());
            return result;
        };

        for entry in entries.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let dir_name = entry.file_name();
            let dir_str = dir_name.to_string_lossy();
            let Some(map_id_str) = dir_str.strip_prefix("mapa_") else {
                continue;
            };
            let Ok(map_id) = map_id_str.parse::<i32>() else {
                continue;
            };

            let specials_path = entry.path().join("specials.json");
            if !specials_path.exists() {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&specials_path) else {
                continue;
            };
            let Ok(raw) = serde_json::from_str::<serde_json::Value>(strip_bom(&content)) else {
                continue;
            };

            let mut exits = HashMap::new();
            if let Some(exits_obj) = raw.get("exits").and_then(|v| v.as_object()) {
                for (coord_key, exit_val) in exits_obj {
                    let parts: Vec<&str> = coord_key.split(',').collect();
                    if parts.len() != 2 {
                        continue;
                    }
                    let Ok(src_x) = parts[0].parse::<i32>() else { continue; };
                    let Ok(src_y) = parts[1].parse::<i32>() else { continue; };

                    if let Some(exit) = Self::parse_single_exit(exit_val) {
                        exits.insert((src_x, src_y), exit);
                    } else if let Some(dests) = exit_val.get("destinations").and_then(|v| v.as_array())
                        && let Some(first) = dests.first()
                        && let Some(exit) = Self::parse_single_exit(first)
                    {
                        exits.insert((src_x, src_y), exit);
                    }
                }
            }

            let terrain = Self::load_terrain(&entry.path());

            let mut safe_tiles = std::collections::HashSet::new();
            if let Some(arr) = raw.get("safeTiles").and_then(|v| v.as_array()) {
                for item in arr {
                    let x = item.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                    let y = item.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                    if x > 0 && y > 0 {
                        safe_tiles.insert((x, y));
                    }
                }
            }

            if !exits.is_empty() || terrain.is_some() || !safe_tiles.is_empty() {
                result.insert(map_id, MapTileData { exits, terrain, safe_tiles });
            }
        }

        let terrain_count = result.values().filter(|m| m.terrain.is_some()).count();
        tracing::info!("Loaded terrain data for {} maps", terrain_count);
        result
    }

    fn load_terrain(map_dir: &std::path::Path) -> Option<MapTerrain> {
        let terrain_path = map_dir.join("terrain.json");
        if !terrain_path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(&terrain_path).ok()?;
        let raw: serde_json::Value = serde_json::from_str(strip_bom(&content)).ok()?;

        let width = raw.get("width").and_then(|v| v.as_i64()).unwrap_or(100) as i32;
        let height = raw.get("height").and_then(|v| v.as_i64()).unwrap_or(100) as i32;

        let palette_val = raw.get("palette")?;
        let palette_map = palette_val.as_object()?;

        #[derive(Default)]
        struct PaletteEntry {
            blocked: bool,
            layer1: u16,
            layer2: u16,
        }

        let mut palette: HashMap<u16, PaletteEntry> = HashMap::new();
        for (key, val) in palette_map {
            let Ok(idx) = key.parse::<u16>() else { continue; };
            let blocked = val.get("blocked").and_then(|v| v.as_bool()).unwrap_or(false);
            let (layer1, layer2) = match val.get("graphics") {
                Some(serde_json::Value::Array(arr)) => {
                    let l1 = arr.first().and_then(|v| v.as_u64()).unwrap_or(0) as u16;
                    let l2 = arr.get(1).and_then(|v| v.as_u64()).unwrap_or(0) as u16;
                    (l1, l2)
                }
                Some(serde_json::Value::Number(n)) => {
                    (n.as_u64().unwrap_or(0) as u16, 0)
                }
                _ => (0, 0),
            };
            palette.insert(idx, PaletteEntry { blocked, layer1, layer2 });
        }

        let grid = raw.get("grid").and_then(|v| v.as_array())?;
        let total = (width * height) as usize;
        let mut tiles = Vec::with_capacity(total);

        for row in grid {
            if let Some(cells) = row.as_array() {
                for cell in cells {
                    let idx = cell.as_u64().unwrap_or(0) as u16;
                    let entry = palette.get(&idx);
                    tiles.push(TileInfo {
                        blocked: entry.map(|e| e.blocked).unwrap_or(false),
                        graphic_layer1: entry.map(|e| e.layer1).unwrap_or(0),
                        graphic_layer2: entry.map(|e| e.layer2).unwrap_or(0),
                    });
                }
            }
        }

        if tiles.len() != total {
            tracing::warn!(
                "Terrain grid size mismatch: expected {} tiles, got {} for {}",
                total, tiles.len(), map_dir.display()
            );
            tiles.resize(total, TileInfo::default());
        }

        Some(MapTerrain { width, height, tiles })
    }

    fn parse_single_exit(val: &serde_json::Value) -> Option<TileExit> {
        let m = val.get("map").and_then(|v| v.as_i64())? as i32;
        let x = val.get("x").and_then(|v| v.as_i64()).unwrap_or(50) as i32;
        let y = val.get("y").and_then(|v| v.as_i64()).unwrap_or(50) as i32;
        if m > 0 {
            Some(TileExit { target_map: m, target_x: x, target_y: y })
        } else {
            None
        }
    }

    pub fn get_tile_exit(&self, map_id: i32, x: i32, y: i32) -> Option<&TileExit> {
        self.tile_data.get(&map_id)?.exits.get(&(x, y))
    }

    pub fn is_safe_map(&self, map_id: i32) -> bool {
        self.maps_meta.get(&map_id).map(|m| m.pk == 1).unwrap_or(false)
    }

    /// Per-tile safe zone check: map-level pk=1 OR tile trigger=6.
    /// Trigger=6 safe tiles loaded from specials.json `safeTiles` array.
    pub fn is_safe_position(&self, map_id: i32, x: i32, y: i32) -> bool {
        if self.is_safe_map(map_id) {
            return true;
        }
        if let Some(td) = self.tile_data.get(&map_id) {
            return td.safe_tiles.contains(&(x, y));
        }
        false
    }

    pub fn get_object(&self, id: i32) -> Option<&ObjectData> {
        self.objects.get(&id)
    }

    pub fn get_npc(&self, id: i32) -> Option<&NpcTemplate> {
        self.npcs.get(&id)
    }

    pub fn get_spell(&self, id: i32) -> Option<&SpellTemplate> {
        self.spells.get(&id)
    }

    pub fn get_map_spawns(&self, map_id: i32) -> Option<&Vec<NpcSpawnEntry>> {
        self.npc_spawns.get(&map_id)
    }

    pub fn get_map_meta(&self, map_id: i32) -> Option<&MapMeta> {
        self.maps_meta.get(&map_id)
    }

    #[allow(dead_code)]
    pub fn is_hostile_npc(template: &NpcTemplate) -> bool {
        template.max_hp > 0 && template.max_hit > 0
    }

    #[allow(dead_code)]
    pub fn is_merchant_npc(template: &NpcTemplate) -> bool {
        template.npc_type == 10 || !template.objs.is_empty()
    }

    #[allow(dead_code)]
    pub fn is_water_tile(&self, map_id: i32, x: i32, y: i32) -> bool {
        self.tile_data
            .get(&map_id)
            .and_then(|m| m.terrain.as_ref())
            .map(|t| t.is_water(x, y))
            .unwrap_or(false)
    }

    pub fn get_map_bounds(&self, map_id: i32) -> (i32, i32) {
        self.tile_data.get(&map_id)
            .and_then(|td| td.terrain.as_ref())
            .map(|t| (t.width, t.height))
            .unwrap_or((100, 100))
    }

    pub fn is_blocked_tile(&self, map_id: i32, x: i32, y: i32) -> bool {
        self.tile_data
            .get(&map_id)
            .and_then(|m| m.terrain.as_ref())
            .map(|t| t.is_blocked(x, y))
            .unwrap_or(false)
    }

    #[allow(dead_code)]
    pub fn is_adjacent_to_water(&self, map_id: i32, x: i32, y: i32) -> bool {
        self.is_water_tile(map_id, x + 1, y)
            || self.is_water_tile(map_id, x - 1, y)
            || self.is_water_tile(map_id, x, y + 1)
            || self.is_water_tile(map_id, x, y - 1)
    }
}
