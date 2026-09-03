# OpenAO — AGENTS.md

> Technical reference for AI agents working on this codebase.
> Last updated: 2026-09-03 (v29)

---

## 1. Project Overview

**OpenAO** is an open-source browser-based MMORPG inspired by Argentum Online. It consists of a **SvelteKit frontend** rendered with Pixi.js and a **Rust backend** game server with WebSocket-based real-time gameplay and an HTTP REST API.

The Elura framework migration is **~100% complete** (133/134 items across 10 phases). The Rust backend uses Elura's Gateway/World architecture patterns, ELR2 protocol, session management, and all gameplay primitives (simulation, AOI, netcode, replication, lag-compensation, room, net-sim). The single remaining item is Redis adapter evaluation for multi-process scaling (N/A for current monolith).

Additionally, a **comprehensive enhancement pass** (Phases A–F) has been completed, adding: command registry refactoring, inventory caching, typed error handling, persistence layer modularization, CI pipeline modernization, IP rate limiting, structured logging, SQLite auto-backup, buff system, navigation (boats), P2P trading, admin invisibility, jail system, IP bans, game data hot-reload, PixiApp decomposition, minimap, toast notifications, particle effects, packet batching, packet priority, broadcast deduplication, quest system, pet system, territory control, spell cooldowns, weather/day-night cycle, achievement system, and real-time leaderboard.

A **Parity & Optimization Pass** (Phase 12, 36 items) has also been completed, adding: missing protocol packets (SELF_MAP_META_DELTA, GLOBAL_NOTICE, ACT_MY_LEVEL, PARTY_STATE, CLAN_STATE, PANEL_SNAPSHOT), CC system (paralysis/immobilization), safety toggles, dead world restrictions, balance system with class multipliers, dual faction scores, item tiers & restrictions, floor item auto-cleanup, World Builder/Arenas/Clans/Runtime Config/Character Settings/Moderation REST APIs, multi-character per account, game data admin API, NPC inspector/admin intervals/overview/debug frontend modals, social meta tags, zero-copy batch sending, SQLite WAL tuning, SvelteKit SSR for wiki, and Pixi.js WebGPU preference.

A **Combat Fidelity & Optimization Pass** (Phases 13–16, 50+ items) has been completed, adding: exact balance formulas ported from Node.js (class progression, HP/Mana/Hit per level, EXP curve), complete combat system with skills/evasion/stabbing/body-part armor absorption, dead world 15s transition system, gold clamping (MAX_GOLD=2,147,483,647), working lock anti-multi-bot system, arena instance manager with dynamic map cloning, shared vaults (account/clan bank tabs), connection policy for duplicate accounts, door system with cooldowns, travel tickets, spell visual compositing, NPC respawn cooldowns, faction rank rewards, Dragon Slayer sword logic (one-shot dragons + map restriction), packet builder capacity hints, batch SQLite writes for world saves, SmallVec for NPC loot, and DashMap shard tuning.

A **Combat & Gameplay Fidelity Pass** (Phase 17, 10 items) has been completed, adding: complete magic damage system (applyMagicBonuses, applyMagicResistance for NPCs and players with class modifiers and item bonuses), NPC crowd control (paralysis/immobilization from spells with tick-based expiry), NPC aggro system (attacked NPCs prioritize their attacker), dead world visibility filtering (dead/hidden players properly filtered on connect/teleport), arena combat integration (PvP always enabled in arena maps), faction PvP rules (rival faction attacks don't flag criminal), working lock anti-multi-bot with DashMap (O(1) IP-based gathering lock), hidden skill (stealth) system (chance/duration from skill level, hunter camo exemption, NPC invisibility, expiry in game loop), heal spell PvP targeting (heal other players in range), and all balance data accessible via hot-reload.

A **Gameplay Refinement & Final Parity Pass** (Phase 18, 10 items) has been completed, adding: newbie system (NEWBIE_MAX_LEVEL=12, item restrictions, auto-unequip on level up), potion percentage recovery verification, map level restrictions (min/max level entry validation with fallback teleport), faction portal restrictions (map 151=caos, map 60=armada), item drop position validation (expanding radius search avoiding blocked tiles/exits/existing items), tile occupied check (prevents two entities on same tile), unsafe logout delay (10s quiet period in PvP zones, cancelled on move/attack, instant in safe zones), boat body resolution (dead=87, special boats 85/86 preserved, default 84), complete visibility system (canRenderCharacter with party/clan dead world override), and AGENTS.md update.

A **Further Parity & Refinements Pass** (Phase 19, 8 items) has been completed, adding: complete newbie item stripping on level 13 (full unequip + inventory removal + visual broadcast), Armada faction loss on attacking neutral citizens, citizen clan PvP block (citizen-aligned clan members can't attack other citizens), support spell PvP rules (citizens can't heal criminals outside arenas), admin commands (/quitarnpcpermanente, /verip, /intervalos with real metrics), NPC EXP/gold multipliers (×5 EXP, ×3 gold matching original), and armor race restrictions (razaEnana bidirectional dwarf/non-dwarf equipment check with id_raza persistence).

A **PvP Rewards & Death Mechanics Pass** (Phase 20, 7 items) has been completed, adding: PvP kill faction score (Armada/Caos dual tracking, 10 pts/kill), PvP rekill protection (5-minute window), kill counters (ciudadanos_matados/criminales_matados persisted), dual faction score persistence, PvP exp/gold rewards, safe logout movement cancel, and comprehensive death cleanup (buffs/CC/meditation/stealth cleared).

A **Combat & AI Fidelity Pass** (Phase 21, 10 items) has been completed, adding: action cooldown system (per-action cooldowns ported from vars.timing.actionCooldowns with cross-action gates), party EXP bonus corrected to 15%, NPC spell casting (offensive/healing magic with cooldowns and FX), NPC target scoring (weighted scoring for smarter AI), per-tile safe zones (trigger=6 from specials.json), arena trigger zone integration, NPC summon infrastructure, and chat audit logging.

A **Spell & Summon Parity Pass** (Phases 22–25, 14 items) has been completed, adding: NPC summon system (player-summoned NPCs with max cap, expiry, cleanup on disconnect), missing action cooldowns (drop_item 150ms, equip_toggle 125ms, click 150ms), structured activity logging (combat/economy/progression events at tracing target), spell effect parity (remover_paralisis, invisibilidad, buff spells subeAg/subeFz via BuffManager, minSkill level gating for spells), and summon expiry in game loop.

A **Fidelity & Optimization Pass** (Phase 26, 9 items) has been completed, adding: dragon respawn cooldown (1-hour for npc_type=6), arena combat kill/death tracking with team scores, advanced NPC AI scoring (escape_tiles/attack_tiles/is_current weights matching original), granular equipment slots (id_arrow_slot/id_ring_slot with full persistence), expanded combat formula tests, NPC AI batch read/write separation, and tick time metrics (max/avg in microseconds via /api/metrics and Prometheus).

A **Admin & Testing Pass** (Phase 27, 5 items) has been completed, adding: admin bot system (`/bot NPC_ID [LEVEL]` spawns scaled NPCs with auto-heal, `/bot limpiar` removes owned bots, `admin_bot_owner` tracking on NpcState), runtime timing hot-modification (`/intervalo key [value]` for dynamic game loop tuning via `RuntimeTimings` struct with `AtomicU64` fields), expanded test suite (ActionCooldowns cross-gate tests, RuntimeTimings defaults, combat formula edge cases for dragon slayer/magic bonuses/resistance/dead world/unsafe logout constants, balance formula tests for all-class HP at level 50 and exp curve monotonicity).

A **Parity Audit & Polish Pass** (Phase 28, 4 items) has been completed, adding: comprehensive parity audit of original Node.js codebase (game.ts 9944 LOC, commands.ts 4329 LOC, protocol.ts 4431 LOC, npcs.ts 3125 LOC, handleProtocol.ts 1570 LOC, login.ts 1218 LOC) against Rust port confirming 100% functional parity, frontend accessibility fixes (0 svelte-check warnings, label-for associations on MacroBar/TradeModal forms), 6 new parity verification tests (PvP base rewards match original multipliers, faction rekill window 5min, bail cost formula, all 11 classes have modifiers, unarmed damage uses wrestling range, hidden skill chance formula bounds).

A **Class ID Parity Fix** (Phase 29, 4 items) has been completed, fixing: critical class ID mapping discrepancy between original Node.js game (IDs 1,2,3,4,6,7,8,9 — skips 5) and Rust port (sequential 1-8). Corrected all 14 combat modifier functions and balance data for Bardo/Druida/Paladin/Cazador to use the correct original values mapped to the Rust sequential IDs.

### Codebase Stats (verified 2026-09-03 v29)

| Component | Files | Lines of Code |
|-----------|-------|---------------|
| Backend (Rust src) | 65 `.rs` files | ~20,239 LOC |
| Backend (Rust crates) | 5 `.rs` files | ~479 LOC |
| Frontend (SvelteKit) | 150 `.ts`/`.svelte` files | ~27,990 LOC |
| Protocol (shared) | 8 TS files | ~982 LOC |
| **Total** | **~228 files** | **~49,690 LOC** |

**Tests**: 181 Rust tests (166 src + 15 crates) + 51 protocol tests + 39 frontend netcode tests = **271 total** (all passing).

---

## 2. Tech Stack

### Frontend (`frontend-svelte/`)

| Layer | Technology | Version |
|----------------|----------------------------------|------------|
| Framework | SvelteKit 5 | ^2.21.0 |
| Rendering | Pixi.js 8 | ^8.17.1 |
| Styling | Tailwind CSS 4 | ^4.2.4 |
| Icons | lucide-svelte | ^0.500.0 |
| Audio | Howler.js | ^2.2.4 |
| Language | TypeScript 6 | ^6.0.3 |
| Build | Vite 6 | ^6.3.0 |
| Deployment | Cloudflare Pages (adapter-cloudflare) | ^7.0.0 |
| Protocol | `@openao/protocol` (local link) | workspace |

### Backend (`game-server-rs/`)

| Layer | Technology | Version |
|----------------|----------------------------------|------------|
| Language | Rust (edition 2024) | 1.97+ |
| Framework | Elura (simulation + aoi + netcode + replication + lag-compensation) | ^0.3.1 |
| Async Runtime | Tokio (full) | ^1 |
| WebSocket | tokio-tungstenite | ^0.29 |
| HTTP API | Axum | ^0.8 |
| CORS | tower-http | ^0.6 |
| Database | SQLx (SQLite, WAL mode) | ^0.8 |
| Concurrency | DashMap | ^6 |
| IDs | UUID v4 | ^1 |
| Time | Chrono | ^0.4 |
| Logging | tracing + tracing-subscriber | ^0.1/^0.3 |
| Error Handling | anyhow | ^1 |
| RNG | rand | ^0.9 |
| Password Hash | argon2 | ^0.6 |
| Serialization | serde + serde_json | ^1 |
| Bytes | bytes | ^1 |
| Small Vectors | smallvec | ^1 |
| Futures | futures-util | ^0.3 |
| Async Traits | async-trait | ^0.1 |

### Shared Protocol (`packages/protocol/`)

TypeScript package `@openao/protocol` providing:
- `PacketReader` / `PacketWriter` — binary serialization
- `opcodes.ts` — all client/server packet IDs (must stay in sync with `game-server-rs/crates/protocol/src/opcodes.rs`)
- `elr2.ts` — ELR2 frame encoder/decoder (28-byte header, `elura.v2` subprotocol constants)
- `messages.ts` — typed packet encode/decode with `encodeClientPacket` / `decodeClientPacket`
- `clientPackets.ts` — client packet payload type definitions
- `constants.ts` — shared game constants

### Rust Protocol (`game-server-rs/crates/protocol/`)

Rust crate `openao-protocol` providing:
- `PacketReader` / `PacketWriter` — binary serialization mirrors
- `opcodes.rs` — packet ID constants matching the TypeScript side
- `constants.rs` — game constants (`CLIENT_VIEW_RANGE_X/Y`, `MAP_MAX_COORDINATE`)

### ELR2 Framing (`game-server-rs/src/elr2.rs`)

Server-side ELR2 frame codec (12 tests covering all edge cases):
- `Frame` — encode/decode 28-byte ELR2 headers
- `FrameKind` — Request(1), Response(2), Push(3), Error(4) with `TryFrom<u8>`
- Routes: 1 (auth), 2 (heartbeat), 100 (game)
- Auto-detection: first 4 bytes = `0x454C5232` → ELR2 mode, else legacy

---

## 3. Architecture

### Repository Structure

```
OpenAO/
├── game-server-rs/        ← Active Rust backend (main development)
│   ├── src/               ← Server source (42 .rs files)
│   ├── crates/protocol/   ← Shared protocol crate (5 .rs files)
│   ├── data/              ← JSON game data (objects, NPCs, spells, maps, terrain)
│   └── Dockerfile
├── frontend-svelte/       ← Active SvelteKit frontend (main development)
│   └── src/               ← Frontend source (133 .ts/.svelte files)
├── packages/protocol/     ← Shared TypeScript protocol (8 .ts files)
├── server/                ← LEGACY Node.js WebSocket server (deprecated)
├── api/                   ← LEGACY Node.js REST API (deprecated)
├── frontend/              ← LEGACY React frontend (deprecated)
├── database/              ← LEGACY PostgreSQL schemas (deprecated)
├── tests/                 ← LEGACY integration tests (deprecated)
├── docs/                  ← Documentation
├── screenshots/           ← Game screenshots
├── .github/               ← CI/CD workflows
└── docker-compose.yml     ← Root Docker Compose (active stack)
```

> **Note**: The `server/`, `api/`, `frontend/`, `database/`, and `tests/` directories are the original Node.js/React/PostgreSQL stack. They are **not actively maintained** — all development happens in `game-server-rs/`, `frontend-svelte/`, and `packages/protocol/`.

### Communication Diagram

```
┌──────────────────────────────────┐
│       Browser (SvelteKit)        │
│  PixiApp.svelte  ← rendering    │
│  gameState.svelte.ts ← state    │
│  outgoingRequests.ts ← send     │
│  registerPacketHandlers.ts ← rx │
│  WebSocket (binary)              │
└──────────┬───────────────────────┘
           │ :7666 WebSocket
           │ :7667 HTTP REST
┌──────────▼───────────────────────┐
│        Rust Game Server          │
│                                  │
│  main.rs        ← bootstrap     │
│  gateway/       ← sessions      │
│  world/         ← state model   │
│  simulation/    ← game loop     │
│  replication/   ← packet build  │
│  gameplay/      ← game systems   │
│  game_data/     ← JSON data     │
│  persistence/   ← SQLite        │
│  api/           ← HTTP (Axum)   │
│  routes/        ← packet router │
│  elr2.rs        ← ELR2 codec    │
│  rate_limit.rs  ← rate limiting │
│  reconnect.rs   ← reconnection  │
│  error.rs       ← error system  │
└──────────────────────────────────┘
```

### Communication Model

- **WebSocket** on port `7666`: ELR2-framed binary protocol with `elura.v2` subprotocol negotiation. Falls back to legacy mode (raw binary) for old clients.
  - **ELR2 mode**: 28-byte header (magic, version, kind, route, request_id, sequence, payload_length) wrapping game payload. Auth via Route 1, heartbeat via Route 2, game packets via Route 100.
  - **Legacy mode**: Each message starts with a 1-byte packet ID followed by payload fields (short, int, string, byte).
- **HTTP REST** on port `7667`: Axum-based JSON API for auth, ranking, arenas, health, wiki, metrics.
- **Broadcast**: `tokio::sync::broadcast` per Scene for area-wide packets (auto-wrapped in ELR2 Push frames when in ELR2 mode).
- **Personal channel**: `tokio::sync::mpsc::UnboundedSender` per player for targeted packets (auto-wrapped in ELR2 Push frames when in ELR2 mode).

### Game Loop

Fixed-step game loop at **60 TPS** using **Elura `FixedStepClock`** (`simulation/mod.rs`):
- Every tick: Combat snapshot recording for lag compensation (`record_combat_snapshots`)
- Every 60 ticks (1s): HP/Mana regeneration
- Every 30 ticks (~0.5s): NPC AI (random movement, detect & attack players)
- Every 1800 ticks (30s): NPC respawn check
- Every 3600 ticks (60s): Market listing expiry + reconnect token eviction
- Every 300 ticks (5s): Idle diagnostic logging

The game loop uses `elura::gameplay::simulation::FixedStepClock` with bounded catch-up work (`max_steps_per_update=10`, `max_accumulated_time=500ms`) and dropped-time reporting instead of a manual `Instant`-based loop.

### Area of Interest (AOI)

Each `Scene` contains an **Elura `AoiGrid<EntityId>`** (`elura::gameplay::aoi::AoiGrid`) for spatial indexing. Cell size matches `CLIENT_VIEW_RANGE_X` for efficient 1-4 cell queries. All entity inserts, moves, and removals update the grid. Query methods `entities_in_range`, `players_in_range`, and `npcs_in_range` use the grid for O(1) cell-based lookups instead of full-scan filters.

---

## 4. Module Responsibilities

### `gateway/` (decomposed into 25 sub-modules)

The session handler, split into focused modules:

| File | Responsibility |
|------|----------------|
| `gateway/mod.rs` | `GameSession` struct, `run()` loop, ELR2/legacy dispatch, packet routing, party/clan state sync, disconnect handling, heartbeat, auth deadline, client timeout |
| `gateway/connect.rs` | `handle_connect_character` — login, reconnect, state init, initial data burst (vitals, gold, exp, attrs, flags, color, equipment, inventory, spells), quest/pet/achievement load |
| `gateway/movement.rs` | Movement, heading, teleport, tile exits, resync position, PvP map change block (5s), navigation (boat embark/disembark) |
| `gateway/combat.rs` | Melee/ranged/spell attacks (PvE + PvP), XP granting (with party sharing), NPC loot drops, criminal system, death item drop, lag-compensated ranged/spell validation, spell cooldown checks, buff application, achievement stat tracking |
| `gateway/dialog.rs` | Chat, 50+ commands, respawn (home-based), fianza, factions, social, admin commands, quest/pet/territory/achievement commands, `/bot`, `/intervalo` |
| `gateway/inventory.rs` | Item pickup, use (potions/food/scrolls/crafting tools), equip (with visual broadcast), drop (with ground item spawn), reorder |
| `gateway/party.rs` | Party system: invite, accept, leave, kick (max 4, faction-compatible) |
| `gateway/clan.rs` | Clan system: create, leave, info, kick, transfer leader, delete, apply, accept/reject requests, co-leaders |
| `gateway/bank.rs` | Bank system: deposit/withdraw gold, reorder bank slots, SQLite persistence |
| `gateway/commerce.rs` | NPC commerce (buy/sell), sacerdote/banquero NPC interactions |
| `gateway/crafting.rs` | Recipe-based crafting (serrucho/costurero/martillo), material check, inventory update |
| `gateway/smelting.rs` | `/fundir` mineral→ingot smelting |
| `gateway/fishing.rs` | Fishing rod use, water tile detection, tick-based attempts, weighted rewards |
| `gateway/harvesting.rs` | Woodcutting/mining, terrain tile detection, tick-based, rewards |
| `gateway/challenges.rs` | Challenge system: create, join, cancel, list (1v1/2v2) |
| `gateway/market.rs` | Player-to-player market: publish, buy, cancel, claim via NPC timbero |
| `gateway/packets.rs` | Packet building helpers (console, vitals, position, gold, exp, attributes, flags, color, equipment, etc.) |
| `gateway/trade.rs` | P2P trading: request, offer gold, confirm, cancel, atomic swap, cleanup on disconnect |
| `gateway/quests.rs` | Quest handlers: `/misiones`, `/mision aceptar|abandonar|completar`, `advance_quest_kills/collect/visit_map` |
| `gateway/pets.rs` | Pet handlers: `/mascotas`, `/invocar`, `/despachar`, `/liberar` |
| `gateway/territory.rs` | Territory handlers: `/territorios` (list territories with capture status) |
| `gateway/achievements.rs` | Achievement handlers: `/logros` (list earned/unearned achievements) |
| `gateway/buffs.rs` | Buff application and tick-based expiry handlers |
| `gateway/navigation.rs` | Boat embark/disembark, water/land tile restrictions |
| `gateway/admin_extra.rs` | Extended admin commands: `/invisible`, `/carcel`, `/banip`, `/unbanip`, `/recargar` |

### `world/mod.rs`

Core data structures:
- `PlayerState` — full player state (100+ fields: identity, position, vitals, attributes, equipment, faction, party, clan, home, fishing, harvesting, PvP timer, revive timer, buffs, quest_log, pets, spell_cooldowns, achievements, navegando, invisible, jail_until_ms, trade state, paralizado, inmovilizado, seguro_activado, seguro_clan_activado, faction_score_armada, faction_score_caos, hidden_skill, hidden_skill_expire_tick, hidden_skill_cooldown_tick, logout_expires_at_ms, id_raza, criminales_matados, ciudadanos_matados, meditar, action_cooldowns, summons)
- `ActionCooldowns` — per-action combat cooldowns (melee 950ms, range 950ms, spell 850ms, use_item 250ms, dialog 500ms, cross-action gates: melee→spell 800ms, spell→melee 800ms, melee→use_item 550ms)
- `NpcState` — NPC instance (id, type, position, heading, vitals, damage, defense, exp, movement, dead flag, aggro_target, paralizado, inmovilizado, cc_expire_tick, spells, spell_cast_interval_ms, last_spell_cast_at, spell_range, magic_def, magic_resistance, summoned_by, summon_expires_at_ms)
- `NpcSpellSlot` — NPC spell definition (spell_id)
- `GroundItem` — dropped item on map (id, item_id, amount, position)
- `FishingState`, `HarvestingState`, `HarvestingSkill` — gathering state machines
- `Position` — map + x/y with `to_point2()` for AOI
- `Party`, `Clan`, `PartyInvite` — social group structures
- `BroadcastPacket` — broadcast channel payload with sender_entity_id for self-exclusion
- `Scene` — per-map container with:
  - `DashMap<EntityId, PlayerState>` — concurrent player map
  - `DashMap<EntityId, NpcState>` — concurrent NPC map
  - `DashMap<EntityId, GroundItem>` — concurrent ground items
  - `broadcast_tx: broadcast::Sender<BroadcastPacket>` — area broadcast
  - `personal_tx: DashMap<EntityId, mpsc::UnboundedSender<Vec<u8>>>` — per-player channel
  - `aoi_grid: RwLock<AoiGrid<EntityId>>` — Elura spatial grid
  - `lag_history: Mutex<SceneLagHistory>` — Elura lag compensation
- `RuntimeTimings` — `AtomicU64` fields for dynamic game loop tuning (`melee_ms`, `range_ms`, `spell_ms`, `use_item_ms`, `dialog_ms`, `regen_ticks`, `npc_ai_ticks`)
- `GameWorld` — owns the database, scenes map, entity ID generator (AtomicU32), parties, clans, party invites, game_data, faction_rekill_tracker (DashMap for PvP rekill protection), runtime_timings (RuntimeTimings)
- AOI methods: `aoi_insert`, `aoi_move`, `aoi_remove`, `entities_in_range`
- Query methods: `players_in_range`, `npcs_in_range` (backed by AoiGrid), `broadcast`, `broadcast_in_range`, `send_to_player`

### `simulation/mod.rs`

Game loop (uses **Elura `FixedStepClock`**):
- `run_game_loop` — Elura `FixedStepClock`-driven loop (replaces manual `Instant` loop)
- `process_tick` — central tick dispatcher, calls all subsystems
- `record_combat_snapshots` — records CombatSnapshot for all players+NPCs per tick for lag compensation
- `process_hp_mana_regen` — sends `SELF_VITALS_DELTA` via personal channel
- `process_npc_ai` — random walk, chase, melee attack, spell casting (updates AoiGrid on NPC move, uses `broadcast_in_range` for AOI-filtered packets, sends entity_vitals_delta + console feedback to attacked players, weighted target scoring, helper functions `apply_npc_melee_damage`/`try_npc_move_towards`/`try_npc_cast_spell`)
- `process_npc_respawn` — respawns missing NPC types by map (inserts into AoiGrid, uses `broadcast_in_range`)
- `process_market_expiry` — async SQLite task to expire old market listings
- `process_idle_log` — diagnostic tick logging

### `replication/mod.rs`

Packet construction — all builders are **synchronized with `registerPacketHandlers.ts`** (entity IDs = `u16`, coordinates = `u16`):
- Character/NPC packets: `build_my_character_packet`, `build_character_packet`, `build_npc_packet`
- Movement: `build_move_entity_packet`
- Vitals/combat: `build_self_vitals`, `build_entity_vitals_delta`, `build_anim_fx`
- Inventory: `build_inv_item_packet`, `get_item_data`
- Spells: `build_learn_spell`, `get_spell_data`, `DEFAULT_SPELLS`
- Ground items: `build_render_item`, `build_delete_ground_item`, `get_npc_loot`
- Sound: `build_play_sound` (sound_id only, no entity_id)
- Entity deletion: `build_delete_character_packet`
- Projectiles: `build_spell_projectile`, `build_create_projectile`
- Classes: `get_class_name()`, `class_level_bonus()` — centralized, corrected for all 8 classes

### `game_data/mod.rs`

Data-driven game content loaded from JSON files at startup (hot-reloadable via `/recargar`):
- `ObjectData` — 1062 items with stats, type, visual, damage, defense, flags (newbie, no_se_cae), crafting/potion properties
- `NpcData` — 336 NPC types with stats, loot tables, movement type, body/head IDs, spell lists (NpcSpellEntry with id_spell/cooldown_seconds)
- `SpellData` — 47 spells with damage, mana cost, FX, projectile IDs
- `MapMetadata` — 294 maps with name, pk flag, safe zone info
- `MapSpawns` — per-map NPC spawn positions and types (167 maps with spawns)
- `CraftingRecipe`, `SmeltingRecipe` — crafting/smelting recipe definitions
- `TileSpecials` — map tile exits for auto-teleport
- `QuestRegistry` — 8 quests loaded from `data/quests.json` with objectives, rewards, prerequisites
- Helper methods: `get_npc()`, `get_item()`, `get_spell()`, `get_map_spawns()`, `get_map_metadata()`, `is_safe_position(map_id, x, y)`
- Hot-reload support: `RwLock<Arc<GameData>>` with `gd()` accessor for zero-downtime reload

### `persistence/` (decomposed into sub-modules)

SQLite operations, decomposed into domain-specific sub-modules:
- `mod.rs` — Schema creation + ALTER TABLE migrations (faction_rank, faction_score, ip_bans, character_quests, character_pets, character_achievements), seed test data, ticket consumption
- `characters.rs` — Character CRUD: `load_character_state`, `save_character_state` (all mutable fields including buffs, navigation, jail, invisibility)
- `inventory.rs` — Inventory CRUD: load/save/update slots, inventory cache support
- `bank.rs` — Bank operations: `get_bank_gold`, deposit/withdraw/reorder
- `market.rs` — Market operations: create/buy/cancel/claim listings, `expire_market_listings`
- `accounts.rs` — Account lookup, ranking query (top 50 by level/gold)
- `quests.rs` — Quest persistence: `load_quest_log`, `save_quest_log` (active + completed quests)
- `pets.rs` — Pet persistence: `load_pets`, `save_pets` (individual pet state)
- `achievements.rs` — Achievement persistence: `load_achievements`, `save_achievements` (tracker + stats)

### `api/mod.rs`

HTTP API (Axum):
- `POST /api/auth/login` — email/password → ticket (argon2 + legacy plaintext compat)
- `POST /api/auth/register` — create account (argon2-hashed password) + character
- `GET /api/auth/me` — stub
- `POST /api/auth/request-password-reset` — stub
- `POST /api/auth/signout` — stub
- `GET /api/ranking` — top 50 by level/gold
- `GET /api/arenas` — static arena list
- `GET /api/health` — "OK"
- `GET /api/readiness` — 200/503 based on shutdown state
- `GET /api/metrics` — JSON server metrics (uptime, connections, players, NPCs, scenes, packet counts, per-category counters, reconnect tokens)
- `GET /api/metrics/prometheus` — Prometheus text exposition format
- Wiki endpoints: items, NPCs, spells from GameData
- Users online stats from scenes
- `GET /api/wiki` — combined items/NPCs/spells data for SvelteKit SSR
- `GET/POST /api/arenas` — arena listing + creation
- `GET /api/clans` — clan listing, `GET /api/clans/{id}` — clan detail
- `GET/POST /api/runtime-config` — server runtime toggles (double exp/gold)
- `GET/POST /api/character-settings/{char_id}` — per-character client preferences
- `GET /api/characters/{account_id}` — list characters for account
- `DELETE /api/characters/{char_id}` — delete character
- `POST /api/admin/ban|unban|mute|unmute|ip-ban|ip-unban` — moderation endpoints
- `GET /api/admin/game-data/objects|npcs|spells` — browse game data
- `POST /api/world/maps/{map_id}/spawns|tiles/exits|metadata` — world builder

### `routes/mod.rs`

Typed packet routing system (4 tests):
- `PacketRouter` — `HashMap<u8, RouteInfo>` registry with O(1) lookup
- `RouteInfo` — per-route ID, name, category, and `requires_character` flag
- `RouteCategory` — logical grouping (Auth, Movement, Combat, Dialog, Inventory, Commerce, Social, Crafting, Gathering, Bank, Market, Challenge, Admin, System)
- Used in `handle_legacy_binary` for route-aware tracing, pre-dispatch validation, and per-category metrics

### `rate_limit.rs`

Per-connection rate limiting (3 tests):
- `RateLimiter` — sliding-window rate limiter (default 60 packets/second)
- `CommandRateLimiter` — named per-command cooldowns (500ms default), used in market/crafting/bank/commerce/smelting/challenges handlers

### `reconnect.rs`

Session reconnection support (3 tests):
- `ReconnectManager` — issues/consumes short-lived reconnect tokens (120s TTL, UUID-based)
- `ReconnectState` — captures session state (account, character, entity, map) for seamless resume
- Periodic eviction called from game loop (every 60s)
- Token metrics exposed via `/api/metrics`

### `error.rs`

Structured error system:
- `GameErrorCode` — 30+ machine-readable error codes organized by domain (Auth 1xx, Movement 2xx, Combat 3xx, Inventory 4xx, Social 5xx, General 9xx)
- `GameError` — user-facing error with code + message, backward-compatible console packet conversion
- Integrated in all handlers: combat, inventory, bank, commerce, market, movement, dialog, admin, smelting, party, clan, crafting, challenges

### `game_module.rs`

Elura-inspired `WorldModule` pattern for modular game route registration:
- `GameModule` trait — `name()` + `register()`, mirrors Elura's `WorldModule`
- `CoreGameModule` — movement, combat, dialog, inventory, click (16 routes)
- `CommerceModule` — buy/sell, bank, market (8 routes)
- `SocialModule` — challenges (1 route)
- `GatheringModule` — crafting (1 route)
- `SystemModule` — ping, toggles, resync (4 routes)
- `build_router_from_modules()` — constructs shared `Arc<PacketRouter>` from all modules
- Router is built once at startup and shared across all sessions (was per-session before)
- 3 tests: module registration, route parity with original router, module names

### `gameplay/` (Elura Integration + Game Systems)

Sub-modules for game rules and Elura integration:
- `combat.rs`, `movement.rs`, `items.rs`, `challenges.rs` — stub game rules with tests (total: 7 tests across movement + items)
- `combat_formulas.rs` — Complete combat formula system ported 1:1 from Node.js `game.ts`: `simulated_skill` (min(100, level*3)), `BodyPart` enum with `random_body_part`, class-based modifiers (`mod_evasion`, `mod_escudo`, `mod_ataque_*`, `mod_dmg_*` for all 11 classes), `poder_evasion`, `poder_evasion_escudo`, `poder_ataque_arma` (Unarmed/Melee/Projectile/Stabbing weapon types), `calcular_dmg` (weapon + arrow + str bonus), `melee_hit_chance` (clamped 5–95%), `shield_block_chance`, `body_part_absorption` (head→helmet, body→body+shield), stabbing system (`can_stab`, `try_stab_npc`, `try_stab_pvp` with per-class chance/damage modifiers), `is_dragon_slayer_hit`, Dragon Slayer sword logic, `apply_magic_bonuses` (class modifier + item modifier + INT bonus), `apply_magic_resistance_to_npc` (NPC magic defense), `apply_magic_resistance_to_user` (RMag stat + INT bonus + class modifier), `hidden_skill_chance`/`hidden_skill_duration_ticks` (skill-based stealth), `can_keep_hidden_while_acting` (class exemption), `NPC_EXP_MULTIPLIER=5`, `NPC_GOLD_MULTIPLIER=3`, `is_newbie_character`, `resolve_boat_body_id` (class exemption), `is_newbie_character` (level ≤12 check), `NEWBIE_MAX_LEVEL`, `UNSAFE_LOGOUT_DELAY_MS`, `resolve_boat_body_id` (dead/boat body resolution) (17 tests)
- `factions.rs` — Faction system with Armada/Caos rank configs (5 ranks each with level/score thresholds), `get_faction_color`, `get_max_eligible_rank`, `get_rank_title`, `calculate_faction_score`, `claim_faction_rewards` with rank progression messages
- `arenas.rs` — `ArenaManager` + `ArenaInstance`: dynamic map cloning for PvP arenas, NPC spawning in instances, participant tracking with team/kills/deaths, handover system for account transfers, instance lifecycle (WaitingForPlayers/InProgress/Finished), cleanup on empty (5 tests)
- `balance.rs` — Exact balance formulas ported from `balance.ts`/`balanceData.ts`: `ClassProgress` (vida, mana_inicial, mult_mana, hit_pre/post_36 for all 11 classes), `get_max_hp_for_level`, `get_max_mana_for_level`, `get_hit_modifier_for_level` (pre/post level 36 split), `get_min/max_hit_for_level`, `get_legacy_exp_next_level` (exact EXP curve with 5 breakpoints at levels 15/21/33/41), `clamp_gold` (MAX_GOLD=2,147,483,647), `clamp_level` (1–50) (11 tests)
- `doors.rs` — `DoorManager`: door open/close with cooldown (250ms), range validation (2 tiles), llave (key) requirement support, visual state tracking (indexAbierta/indexCerrada), sound effect, concurrent access via RwLock (4 tests)
- `netcode.rs` — `SceneLagHistory` wrapping Elura `LagCompensationHistory` for server-side hit rewind validation (64 ticks, 30-tick max rewind, 3 tests)
- `entity_replication.rs` — `ObserverReplicator` wrapping Elura `ReplicationSender` for per-observer delta entity replication (spawn/despawn/keyframe/delta/ACK, wired into game loop, 4 tests). Includes `broadcast_announced` set for deduplication with `broadcast_in_range`.
- `input_queue.rs` — `PlayerInputReceiver` wrapping Elura `InputReceiver` for per-player server-side input validation, de-duplication, and reordering (4 tests)
- `rooms.rs` — `ChallengeRoomManager` wrapping Elura `Room` for challenge/arena roster management with readiness, capacity, leader succession, and lifecycle (Open/Active/Closed). Replaces old `ChallengeManager` (7 tests)
- `net_sim.rs` — Deterministic network simulation tests using Elura `SimulatedLink` — validates game protocol resilience under latency, loss, reorder, overflow, bandwidth, jitter, and redundant input recovery (9 tests)
- `buffs.rs` — Tick-based buff system (`BuffManager`): agility/strength/speed buffs with duration tracking, magnitude, tick-based expiry (4 tests)
- `quests.rs` — Quest system (`QuestRegistry`, `PlayerQuestLog`, `ActiveQuest`): 5 objective types (kill_npc, collect_item, visit_map, talk_npc, reach_level), rewards (gold/exp/items), prerequisites, repeatable quests, max 10 active. 8 quests loaded from `data/quests.json` (9 tests)
- `pets.rs` — Pet system (`PetManager`, `Pet`): max 5 pets per player, summon/dismiss/release, level-up with exp, HP/damage, persistence to SQLite (8 tests)
- `territory.rs` — Territory control (`TerritoryManager`, `Territory`): 5 capturable zones tied to maps, capture progress with contestation, clan ownership, bonus exp/gold (5 tests)
- `cooldowns.rs` — Spell cooldown system (`CooldownManager`): per-spell cooldown tracking, remaining time, cleanup, default cooldowns by spell tier (6 tests)
- `achievements.rs` — Achievement system (`AchievementTracker`, `PlayerStats`): 13 achievements across 10 condition types (level, kills, gold, quests, clan, challenge, fish, craft, maps, deaths), persistence to SQLite (6 tests)

---

## 5. Game Mechanics Implemented

| Feature | Status | Notes |
|------------------------|--------|-------|
| User registration | ✅ | email/password, auto-create character |
| User login | ✅ | email → ticket → WebSocket connect |
| Character creation | ✅ | 8 classes, 5 races, class-based stats |
| Character connect | ✅ | ticket consumption, full state load |
| Movement | ✅ | 4 directions, broadcast to area |
| Chat / Commands | ✅ | `/online`, `/pos`, `/hp`, `/tp`, `/revivir`, `/help`, `/global`, `/w`, `/p`, `/c`, `/meditar`, `/stats`, `/fundir`, `/faccion`, `/enlistar`, `/recompensa`, `/fianza`, `/asignarhogar`, `/hogar`, `/salir`, `/party`, `/aceptar`, `/salirparty`, `/expulsarparty`, `/clan crear\|salir\|info\|expulsar\|lider\|eliminar\|postular\|aceptar\|rechazar\|colider`, all `/clan*` aliases, `/ban`, `/unban`, `/mute`, `/globalgm`, `/worldsave`, `/inspect`, `/cambiarclase`, `/limpiarpiso`, `/dobleexp`, `/dobleoro`, `/invocarnpc`, `/quitarnpc`, `/quitarnpcpermanente`, `/devresetmap`, `/misiones`, `/mision aceptar\|abandonar\|completar`, `/mascotas`, `/invocar`, `/despachar`, `/liberar`, `/territorios`, `/logros`, `/embarcar`, `/desembarcar`, `/comerciar`, `/invisible`, `/carcel`, `/banip`, `/unbanip`, `/recargar`, `/seguro`, `/seguroclan`, `/verip`, `/intervalos`, `/paquetes`, `/bot`, `/intervalo` |
| Melee combat | ✅ | PvE + PvP, data-driven damage, safe zone validation |
| Spell casting | ✅ | PvE + PvP, 47 spells from data, mana cost, FX broadcast |
| HP/Mana regen | ✅ | 2% HP / 3% Mana per second |
| Meditation | ✅ | `/meditar`, 8% mana regen + FX |
| NPC spawning | ✅ | Data-driven from 167 maps, 336 NPC types |
| NPC respawn | ✅ | Every 30s, replaces missing types |
| NPC AI | ✅ | Random walk, chase & melee players in range (attack range 5, melee range 1) |
| XP & Level up | ✅ | NPC exp_reward, progressive curve, class-based HP/Mana gains |
| Gold system | ✅ | NPC gold drops, persistence |
| Inventory | ✅ | 20 slots, use (heal), equip, drop, pickup, reorder |
| Ground items (loot) | ✅ | Data-driven NPC loot tables, AOI-filtered visibility |
| Teleportation | ✅ | `/tp map x y`, scene transfer with AOI-filtered entity/ground item loading |
| Death & respawn | ✅ | `/revivir`, half HP/Mana, teleports to home position |
| Safe zones | ✅ | 294 maps metadata, attack blocked in pk=1 maps |
| Whisper chat | ✅ | `/w name message`, cross-scene |
| Global chat | ✅ | `/global message`, all scenes |
| Persistence | ✅ | SQLite, full state save on disconnect (all mutable fields: HP, mana, gold, level, exp, attrs, equipment, dead, faction, criminal, home, class, faction rank/score, navegando, bank_gold) |
| Ranking | ✅ | HTTP endpoint, top 50 |
| Wiki API | ✅ | Items, NPCs, spells from GameData |
| Users online stats | ✅ | Live player count from scenes |
| NPC Commerce | ✅ | Click NPC to open shop, buy/sell items for gold |
| Crafting | ✅ | Recipe-based (serrucho/costurero/martillo), material check, inventory update |
| Smelting | ✅ | `/fundir id`, mineral→ingot conversion |
| Factions | ✅ | `/faccion armada|caos|salir|info`, rank system, persistence (rank + score) |
| Graceful shutdown | ✅ | SIGINT saves all connected players before exit (10s drain timeout) |
| Map names | ✅ | Real names from metadata on connect/teleport |
| Map tile exits | ✅ | Data-driven from `specials.json`, auto-teleport on step |
| Party system | ✅ | `/party name`, `/aceptar`, `/salirparty`, `/expulsarparty`, max 4, faction-compatible, shared XP (10% bonus) |
| Clans | ✅ | `/clan crear|salir|info|expulsar|lider|eliminar|postular|aceptar|rechazar|colider`, aliases, co-leaders, request system, real-time state sync |
| Ranged combat | ✅ | PvE + PvP, range 8 tiles, damage with defense reduction, full loot/XP, lag-compensated (3-tick rewind) |
| PvP combat | ✅ | Melee, ranged, spell attacks against players, death handling |
| Criminal system | ✅ | Attacking non-criminal players marks attacker as criminal, persisted |
| Potions (HP/Mana) | ✅ | Data-driven HP and Mana potions with percentage bonuses, vitals broadcast to AOI |
| Potions (Agi/Str) | ✅ | Agility/Strength buff potions with UPDATE_AGILIDAD/UPDATE_FUERZA packets |
| Drink/eat sounds | ✅ | PLAY_SOUND broadcast on potion/food consumption |
| Inventory reorder | ✅ | Swap any two inventory slots, full refresh to client |
| Spell reorder | ✅ | No-op stub (spells are static/default) |
| Hidden skill toggle | ✅ | Stub with feedback message |
| Clan safety toggle | ✅ | Stub with feedback message |
| Equipment visual sync | ✅ | Equip/unequip broadcasts weapon/body/helmet/shield changes to area |
| Bail system | ✅ | `/fianza pagar`, costs 1000 gold, requires safe zone |
| Home assignment | ✅ | `/asignarhogar` sets current pos, `/hogar` shows home, respawn uses home |
| Faction enlist/reward | ✅ | `/enlistar`, `/recompensa` for faction info |
| Criminal persistence | ✅ | Criminal status saved/loaded from DB |
| Name color sync | ✅ | ACT_COLOR_NAME broadcast on criminal/faction change, connect, teleport |
| Safe zone flags | ✅ | SELF_FLAGS_DELTA sent on connect and teleport with map pk data |
| Bank system | ✅ | Deposit/withdraw gold, reorder bank slots, SQLite persistence |
| Death item drop | ✅ | Non-newbie/non-nodrop items dropped on PvP death |
| Admin commands (basic) | ✅ | `/darexp`, `/daroro`, `/kick`, `/telepme`, `/telepuser`, `/traer`, `/devrevivir`, `/crearitem` |
| Admin: ban/mute | ✅ | `/ban`, `/unban`, `/mute` (toggle), in-memory |
| Admin: global GM msg | ✅ | `/globalgm message`, broadcasts to all scenes |
| Admin: world save | ✅ | `/worldsave`, saves all connected players |
| Admin: inspect | ✅ | `/inspect name`, shows full player stats |
| Admin: change class | ✅ | `/cambiarclase [1-8]`, persisted via save |
| Admin: clean floor | ✅ | `/limpiarpiso`, removes all ground items |
| Admin: double exp/gold | ✅ | `/dobleexp`, `/dobleoro` toggle, applies to all combat rewards |
| Admin: spawn NPC | ✅ | `/invocarnpc id`, spawns NPC at player position |
| Admin: remove NPC | ✅ | `/quitarnpc`, removes nearest NPC |
| Admin: reset map | ✅ | `/devresetmap`, removes all NPCs and ground items |
| Admin: shutdown | ✅ | `/apagar` (redirects to Ctrl+C graceful shutdown) |
| NPC Sacerdote interact | ✅ | Click sacerdote: revive (if dead) or full heal (if alive) |
| NPC Banquero interact | ✅ | Click banquero: shows bank gold and slot info |
| Non-attackable NPCs | ✅ | NPCs with maxHp=0 (sacerdotes, comerciantes, etc.) cannot be targeted |
| Mute on all channels | ✅ | Mute blocks chat, /global, /w, /p, /c |
| Click player info | ✅ | Click on player shows name, class, level, faction/criminal status |
| Click NPC info | ✅ | Click on NPC shows name, HP, description |
| Spell projectiles | ✅ | SPELL_PROJECTILE + CREATE_PROJECTILE packets, frontend handlers |
| Ranged projectiles | ✅ | Arrow projectile visual on ranged attacks |
| Crafting modal | ✅ | Using crafting tool (serrucho/costurero/martillo) opens crafting UI |
| Pergaminos (scrolls) | ✅ | Using scroll items teaches new spells (class validation) |
| Class-based level up | ✅ | HP/Mana gains per level based on character class (corrected for all 8) |
| Party EXP sharing | ✅ | XP distributed among party members in range with 10% bonus |
| PvP map change block | ✅ | 5s block on map change after PvP combat |
| Bail UI (openBail) | ✅ | `/fianza` sends openBail packet with full bail data to frontend |
| Cast bar packets | ✅ | `startCastBar`/`stopCastBar` fully wired: backend sends, frontend renders progress bar |
| Entity vitals w/ mana | ✅ | `entityVitalsDelta` now includes mana/maxMana fields; broadcast to AOI observers on all HP/mana changes |
| Market system | ✅ | Player-to-player market via NPC timbero, SQLite listings/claims, auto-expiry in game loop |
| Challenges (1v1/2v2) | ✅ | ChallengeManager: create, join, cancel, list; ELR2 wired |
| Fishing | ✅ | Rod equip, water tile detection, tick-based attempts, weighted rewards |
| Harvesting | ✅ | Woodcutting/mining, terrain tile detection, tick-based, rewards |
| Server-side collision | ✅ | Player + NPC movement validated against `is_blocked_tile()` from terrain data |
| Ban/mute persistence | ✅ | SQLite tables `bans`/`mutes`, loaded on startup/connect, admin commands persist |
| Password auto-migration | ✅ | Legacy plaintext passwords re-hashed to argon2 on successful login |
| Party leader transfer | ✅ | Leadership auto-transfers on disconnect instead of disbanding party |
| Entity ID recycling | ✅ | `next_id()` wraps at u32::MAX, skips 0 sentinel |
| Dev ticket flag | ✅ | `OPENAO_DEV_TICKETS=1` env var gates ticket reuse (off by default) |
| Buff system | ✅ | Tick-based buffs (agi/str/speed) with duration, magnitude, auto-expiry in game loop |
| Navigation (boats) | ✅ | `/embarcar`/`/desembarcar`, water tile detection, movement restrictions, visual change |
| P2P Trading | ✅ | `/comerciar name`, offer gold, confirm/cancel, atomic swap, cleanup on disconnect |
| Admin: invisibility | ✅ | `/invisible` toggle, AOI-filtered (hidden from non-admins), persisted |
| Jail system | ✅ | `/carcel name time`, `jail_until_ms` field, blocks TP/Hogar, auto-release |
| IP ban system | ✅ | `/banip`/`/unbanip`, `ip_bans` SQLite table, check on connection |
| Game data hot-reload | ✅ | `/recargar`, `RwLock<Arc<GameData>>` + `gd()` accessor for zero-downtime reload |
| Quest system | ✅ | 8 quests from JSON, 5 objective types, rewards, `/misiones`, `/mision aceptar\|abandonar\|completar`, SQLite persistence (9 tests) |
| Pet system | ✅ | Max 5 pets, summon/dismiss/release, level/exp, `/mascotas`, `/invocar`, `/despachar`, `/liberar`, SQLite persistence (8 tests) |
| Territory control | ✅ | 5 capturable zones, clan ownership, capture progress, bonus exp/gold, `/territorios` (5 tests) |
| Spell cooldowns | ✅ | Per-spell `CooldownManager`, tier-based defaults, integrated in combat handlers (6 tests) |
| Weather system | ✅ | Client-side rain/snow/fog/storm particle effects via `WeatherSystem` + `WeatherOverlay.svelte` |
| Day/Night cycle | ✅ | Client-side 20min cycle (dawn/day/dusk/night), tint overlay via `DayNightCycle` class |
| Achievement system | ✅ | 13 achievements, 10 condition types, `AchievementTracker` + `PlayerStats`, `/logros`, SQLite persistence (6 tests) |
| Real-time leaderboard | ✅ | Top-5 online players broadcast every 30s via WebSocket push from game loop |
| Packet batching | ✅ | `sink.feed()` + `sink.flush()` for efficient multi-packet WebSocket sends |
| Packet priority | ✅ | Critical/High/Normal/Low priority for congestion management, `outbound_pressure` counters |
| Broadcast dedup | ✅ | `broadcast_announced` set in `ObserverReplicator` prevents redundant spawn packets with `broadcast_in_range` |
| Inventory caching | ✅ | In-memory inventory cache reduces SQLite queries on frequent item operations |
| IP rate limiting | ✅ | Per-IP connection rate limiting (separate from per-session packet rate) |
| Structured logging | ✅ | Correlation IDs for session tracing across handlers |
| SQLite auto-backup | ✅ | Periodic `.backup` command via game loop for data safety |
| Bots (admin) | ✅ | `/bot NPC_ID [LEVEL]` spawns scaled NPC, `/bot limpiar` removes owned bots, auto-heal in game loop |
| Map editor (frontend) | ✅ | `/construccion` route with tile painting, NPC placement, teleport triggers, blocked tiles, undo/redo, layer selection |
| Password reset flow | ✅ | `/forgot-password` request + `/reset-password/[token]` completion |
| Frontend: Minimap | ✅ | Real-time minimap overlay showing player position, nearby entities, NPCs |
| Frontend: Toast system | ✅ | Non-blocking toast notifications for game events |
| Frontend: Particle FX | ✅ | `ParticleOverlay.svelte` for visual spell/combat effects |
| Frontend: PixiApp decomposition | ✅ | Refactored monolithic `PixiApp.svelte` into focused rendering sub-modules |
| CC system (paralysis/immobilization) | ✅ | `paralizado`/`inmovilizado` flags, tick-based expiry, blocks movement/combat, spell application |
| Safety toggles (PvP) | ✅ | `seguro_activado` blocks player attacks, `seguro_clan_activado` blocks clan-member attacks, `/seguro`/`/seguroclan` commands |
| Dead world restrictions | ✅ | Dead players cannot attack, cast spells, or use items; must `/revivir` first |
| Balance system | ✅ | `gameplay/balance.rs`: `compute_player_stats`, `compute_damage`, `compute_spell_damage`, `compute_exp_for_kill`, class attack/defense multipliers, weapon/armor scaling, gold clamping by level |
| Dual faction scores | ✅ | Both `faction_score_armada` and `faction_score_caos` tracked and persisted per character |
| Item tiers & restrictions | ✅ | Class and race restrictions on equipment, magic item stats (min/max modifiers), item tier system |
| Floor item cleanup | ✅ | Ground items auto-expire after configurable lifetime (180s default), cleanup runs periodically in game loop |
| Runtime config API | ✅ | `GET/POST /api/runtime-config` for double exp/gold toggles, server-side `RuntimeConfig` state |
| Character settings API | ✅ | `GET/POST /api/character-settings/{id}` for per-character client preferences (persisted to SQLite) |
| Multi-character per account | ✅ | `GET /api/characters/{account_id}` lists all characters, `DELETE /api/characters/{char_id}` deletes character |
| Moderation REST API | ✅ | `POST /api/admin/ban|unban|mute|unmute|ip-ban|ip-unban` endpoints for admin moderation |
| Game data admin API | ✅ | `GET /api/admin/game-data/objects|npcs|spells` for browsing game data |
| Wiki SSR | ✅ | Wiki pages fetch real game data from backend via SvelteKit `+page.server.ts` SSR |
| NPC inspector modal | ✅ | `NpcInspectorModal.svelte` — click NPC to inspect stats, body/head IDs, HP, exp |
| Admin intervals modal | ✅ | `AdminIntervalsModal.svelte` — toggle double exp/gold via REST API from game UI |
| Overview modal | ✅ | `OverviewModal.svelte` — character overview (level, gold, map, vitals, attributes) |
| Debug overlay | ✅ | `DebugOverlay.svelte` — position, map, entity counts, tick, RTT, vitals (F3 toggle) |
| Social meta tags | ✅ | Open Graph and Twitter meta tags in SvelteKit layout for social sharing |
| WebGPU rendering | ✅ | Pixi.js 8 `preference: "webgpu"` with automatic WebGL fallback |
| Batch packet sending | ✅ | `send_batch_to_client()` with `feed()+flush()` reduces syscalls on connect (2 flushes vs 30+) |
| SQLite tuning | ✅ | WAL mode + 256 stmt cache + 8MB page cache + NORMAL sync + MEMORY temp_store + 256MB mmap |
| Exact balance formulas | ✅ | `balance.rs`: class progression (11 classes), HP/Mana/Hit per level, EXP curve (5 breakpoints), gold clamp (11 tests) |
| Complete combat formulas | ✅ | `combat_formulas.rs`: simulated skills, evasion/attack power, hit chance, shield block, body part absorption, stabbing (11 tests) |
| Dead world system (15s) | ✅ | `DEAD_WORLD_DELAY_MS=15000`, `dead_world_active` flag, visibility filter for dead players |
| Gold clamp | ✅ | MAX_GOLD=2,147,483,647, applied on all gold mutations (add/subtract/trade/market) |
| Working lock (anti-bot) | ✅ | Prevents simultaneous fishing/harvesting from same IP, per-entity IP tracking |
| Arena instance manager | ✅ | Dynamic map cloning, NPC spawning in instances, participant tracking, handover system (5 tests) |
| Shared vaults | ✅ | Account-wide + clan-wide bank tabs, SQLite persistence, deposit/withdraw/reorder |
| Connection policy | ✅ | Penalizes idle characters when duplicate account sessions detected |
| Door system | ✅ | Open/close with cooldown (250ms), key requirement, visual state, range validation (4 tests) |
| Travel tickets | ✅ | Items with `travelTicketDestination` teleport player on use, consumed after travel |
| NPC respawn cooldowns | ✅ | Per-NPC individual respawn timers, persisted across restarts |
| Faction rank rewards | ✅ | `/recompensa` with rank progression, level/score thresholds, reward messages |
| Dragon Slayer sword | ✅ | One-shot kill on dragons (npcType=6), sword consumed after hit, Clan Ring map entry restriction |
| Packet builder capacity hints | ✅ | Pre-allocated capacity for hot-path packets (MOVE_ENTITY, DELETE_CHAR, VITALS, ANIM_FX, etc.) |
| Batch SQLite world saves | ✅ | Single transaction for `/worldsave`, `begin_transaction` + `save_character_state_in_tx` |
| SmallVec for NPC loot | ✅ | `SmallVec<[(i32,i32,u16);4]>` avoids heap allocation for common loot tables |
| DashMap shard tuning | ✅ | `scenes`, `inventory_cache`, `inventory_dirty` configured with 32 shards for reduced contention |
| Magic damage system | ✅ | `apply_magic_bonuses`, `apply_magic_resistance_to_npc/user`, class modifiers, item bonuses, magic penetration (6 tests) |
| NPC crowd control | ✅ | Spells apply paralysis/immobilization to NPCs with tick-based expiry, integrated in NPC AI |
| NPC aggro system | ✅ | `aggro_target` on NpcState, NPCs prioritize attacker, validated in AI loop |
| Dead world visibility filtering | ✅ | Dead/hidden/invisible players properly filtered on connect/teleport entity loading |
| Arena combat integration | ✅ | `is_arena_map()` bypasses safe zone PvP checks in melee/ranged/spell handlers |
| Faction PvP rules | ✅ | Rival faction attacks (Armada vs Caos) don't flag criminal; same-faction attacks do |
| Hidden skill (stealth) | ✅ | Chance/duration from skill level, hunter camo exemption, NPC invisibility, movement/attack reveal, expiry in game loop |
| Heal spell PvP targeting | ✅ | Heal spells target nearest player in range, level-scaled healing, target receives vitals/console feedback |
| Newbie system | ✅ | NEWBIE_MAX_LEVEL=12, newbie items blocked when level >12, notification on level up |
| Map level restrictions | ✅ | min_level/max_level on MapMeta, validated on teleport/tile exits, descriptive deny messages |
| Faction portal restrictions | ✅ | Map 151 caos-only, map 60 armada-only, admin bypass |
| Item drop position validation | ✅ | Expanding radius search, avoids blocked tiles/exits/existing items |
| Tile occupied check | ✅ | Prevents two living entities on same tile (players + NPCs) |
| Unsafe logout delay | ✅ | `/salir` 10s quiet in PvP zones, instant in safe zones, cancelled on move/attack |
| Boat body resolution | ✅ | Dead=87, special boats 85/86 preserved, default 84 |
| Complete visibility system | ✅ | canRenderCharacter: party/clan override dead world, invisible/stealth filtering |
| Newbie item stripping (Lvl 13) | ✅ | Full unequip + removal of newbie items from inventory on reaching level 13, visual broadcast |
| Armada faction loss | ✅ | Attacking neutral non-criminal citizen removes Armada enlistment, clears faction/rank |
| Citizen clan PvP block | ✅ | Citizen-aligned clan members cannot attack other citizen-aligned players |
| Support spell PvP filter | ✅ | Citizens cannot heal criminals outside arenas |
| Admin: /verip | ✅ | Shows target player's IP address (admin command) |
| Admin: /quitarnpcpermanente | ✅ | Permanently removes nearest NPC from map |
| Admin: /intervalos, /paquetes | ✅ | Real server metrics: uptime, connections, tick, packets per category |
| NPC EXP/gold multipliers | ✅ | ×5 EXP, ×3 gold on all NPC kills (matching original multiplicadorExp/multiplicadorGold) |
| Armor race restriction | ✅ | razaEnana bidirectional check: dwarves can only equip dwarf armor, non-dwarves blocked |
| Class equip restriction | ✅ | clases_no_permitidas enforced on equip in addition to use |
| PvP kill faction score | ✅ | Armada/Caos faction score awarded on rival PvP kills (10 pts/kill), dual tracking |
| PvP rekill protection | ✅ | 5-minute window prevents farming faction score from same player |
| Kill counters | ✅ | ciudadanos_matados/criminales_matados tracked and persisted to SQLite |
| PvP exp/gold rewards | ✅ | Base 50 exp × 5, 10 gold × 3 on player kills, double exp/gold respected |
| PvP death cleanup | ✅ | Buffs, CC, meditation, hidden skill cleared on death |
| Enhanced bail system | ✅ | Bail cost = ciudadanos_matados × multiplicadorGold × 5000 (ported from original) |
| Action cooldown system | ✅ | Per-action cooldowns (melee 950ms, range 950ms, spell 850ms, use_item 250ms, dialog 500ms) with cross-action gates, ported from vars.timing.actionCooldowns |
| Party EXP bonus (15%) | ✅ | Corrected from 10% to 15% matching original partyExpBonusPct |
| NPC spell casting | ✅ | NPCs with spells cast offensive/healing magic with cooldowns, FX, projectiles, damage resistance |
| NPC target scoring | ✅ | Weighted scoring (distance×3, adjacent -6, aggro -14) for smarter AI target selection |
| Per-tile safe zones | ✅ | trigger=6 tiles loaded from specials.json, is_safe_position(map, x, y) replaces map-only check |
| Chat audit logging | ✅ | Structured tracing at chat_audit target for moderation audit trail |
| NPC Summon system | ✅ | Player-summoned NPCs via spells (num_npc), max 3, 2min expiry, cleanup on disconnect, 0 EXP |
| Drop item cooldown | ✅ | 150ms cooldown on item drops preventing rapid-fire drop spam |
| Equip toggle cooldown | ✅ | 125ms cooldown on equip/unequip preventing rapid toggling |
| Click cooldown | ✅ | 150ms cooldown on entity clicks preventing click spam |
| Activity logging | ✅ | Structured tracing for combat, economy, progression events (npc_kill, pvp_kill, buy/sell, level_up, connect/disconnect) |
| Remove paralysis spell | ✅ | Spells with remover_paralisis=1 clear paralysis and immobilization on caster |
| Invisibility spell | ✅ | Spells with invisibilidad=1 make caster invisible (DELETE_CHARACTER broadcast to AOI) |
| Buff spells (Agi/Str) | ✅ | Spells with sube_ag/sube_fz=1 apply agility/strength buffs via BuffManager with min/max range |
| Spell minSkill gate | ✅ | simulated_skill(level) < minSkill check prevents casting spells above player skill level |
| Dragon respawn cooldown (1hr) | ✅ | NPCs with npc_type=6 (dragons) have 1-hour respawn cooldown matching original DRAGON_RESPAWN_COOLDOWN_MS |
| Arena kill/death tracking | ✅ | record_kill, get_team_scores, get_winning_team on ArenaManager for match scoring |
| Advanced NPC AI scoring | ✅ | escape_tiles, attack_tiles, is_current weights matching original NPC_TARGET_SCORE_WEIGHTS |
| Equipment slots (arrow/ring) | ✅ | id_arrow_slot, id_ring_slot fields with full persistence roundtrip |
| Tick time metrics | ✅ | tick_time_max_us, tick_time_avg_us in /api/metrics JSON and Prometheus |
| Admin bot system | ✅ | `/bot NPC_ID [LEVEL]` spawns scaled NPCs with auto-heal, `/bot limpiar` removes owned bots |
| Runtime timing tuning | ✅ | `/intervalo key [value]` dynamic game loop tuning (melee/range/spell/item/dialog ms, regen/npc_ai ticks) |

---

## 6. Elura Framework Reference

### What is Elura?

Elura (v0.3.1) is a Rust framework for authoritative multiplayer game servers. It separates **Gateway** (transport, sessions, authentication) from **World** (business logic, game handlers).

**Documentation:**
- Guides: https://elura.rustyspottedcat.dev/guides/
- Concepts: https://elura.rustyspottedcat.dev/concepts/
- Adapters: https://elura.rustyspottedcat.dev/adapters/
- Providers: https://elura.rustyspottedcat.dev/providers/
- API docs: https://docs.rs/elura/latest/elura/
- All items: https://docs.rs/elura/latest/elura/all.html

### Key Elura Concepts

#### Architecture

```
Client ── TCP/UDP/WebSocket/WebTransport/QUIC ──> Gateway ── ELR2/TCP ──> World
                                                    │                      │
                                                    ├── session state      ├── handlers
                                                    ├── admission          ├── middleware
                                                    └── admin HTTP         └── persistence
```

- **Gateway**: Trust boundary. Handles transports, rate limits, auth tickets, sessions, heartbeats, reconnect, routing to World.
- **World**: Business execution. Runs typed handlers, middleware, identity context. Can host `SceneRuntime` for multiplayer rooms/maps.
- **Monolith**: In-process Gateway+World for local dev.

#### ELR2 Protocol

Binary framing protocol. 28-byte header:
- Magic: `0x454C5232` ("ELR2")
- Version: 2
- Kind: Request(1), Response(2), Push(3), Error(4)
- Route: runtime routes 1-4 (auth, heartbeat, renew, session control), app routes 100+
- Request ID, Sequence, Payload length

WebSocket requires `elura.v2` subprotocol. Payloads are protobuf or JSON.

#### Sessions & Routing

- Identity: `account_id`, `region_id`, `realm_id`, `user_id`, `generation`
- Login tickets: short-lived, single-use (default 60s)
- Reconnect tickets: rotated on auth (default 30min)
- Online directory: session leases, duplicate login policy
- Ownership: shard-based player→World instance mapping

#### Gameplay Primitives (used by OpenAO)

| Primitive | What it does | OpenAO Usage |
|-------------------------|-------------|-------------|
| `FixedStepClock` | Bounded deterministic simulation timing | ✅ Game loop at 60 TPS |
| `AoiGrid` | 2D visibility queries + entered/left deltas | ✅ Per-scene spatial indexing |
| `LagCompensationHistory`| Bounded historical snapshots + rewind queries | ✅ 3-tick rewind for ranged/spell combat |
| `ReplicationSender/Rx` | Per-observer Spawn/Despawn/Delta/Keyframe with ACK | ✅ Infrastructure ready (ObserverReplicator), game loop wiring pending |

#### Gameplay Primitives (integrated)

| Primitive | What it does | OpenAO Usage |
|-------------------------|-------------|-------------|
| `Room` | Roster, readiness, leader succession | ✅ `ChallengeRoomManager` wraps Room for 1v1/2v2 challenges |
| `SimulatedLink` | Deterministic weak-network testing | ✅ 9 unit tests covering latency, loss, reorder, overflow, jitter |
| `InterpolationBuffer` | Remote entity interpolation + adaptive jitter delay | ✅ Ported to TypeScript + **wired** into `moveEntity()` + `PixiApp` render loop (per-entity buffers, smooth interpolation) |
| `PredictionBuffer` | Client prediction + authoritative reconciliation | ✅ Ported to TypeScript + **wired** into `tryMove()` + `applyServerPosition()` (instant prediction, server reconciliation with replay) |
| `InputSender` | Client-side redundant input packaging | ✅ Ported to TypeScript + **wired** into `tryMove()` + `applyServerPosition()` (sequence tracking, cumulative ACK) |

#### Gameplay Primitives (not yet used)

| Primitive | What it does |
|-------------------------|-------------|
| `PredictedEntityMatcher`| Temporary→authoritative entity matching |

#### Adapters

Dependency-inverted infrastructure:

| Contract | Built-in options |
|----------------------|-----------------|
| `WorldDiscovery` | DNS, Redis, Kubernetes |
| `ReplayStore` | Memory, Redis |
| `OnlineDirectory` | Memory, Redis |
| `PushTransport` | In-process, Redis Streams |
| `AccountVersionStore`| Memory, Redis, SQL |
| `AdmissionController`| Realm policy, Redis |
| Outbox | Memory, Redis, PostgreSQL, MySQL |

Rule: Start in-memory, add shared infra only when needed across replicas.

#### Providers

Identity, OTP, notifications, payments — all optional:
- Identity: Guest, password, phone, OAuth 2.0, WeChat, Douyin, QuickSDK
- OTP: `OtpService` with pluggable storage
- Notifications: Aliyun SMS
- Payments: Alipay, Apple, Douyin, WeChat

### Elura Feature Flags (currently enabled)

```toml
[dependencies]
elura = { version = "0.3.1", features = [
  "simulation",       # FixedStepClock — ✅ used in game loop
  "aoi",              # Area-of-Interest grid — ✅ used per scene
  "netcode",          # Input/tick sync, prediction, interpolation — ✅ InputReceiver + TS ports
  "replication",      # Per-observer state replication — ✅ ObserverReplicator ready
  "lag-compensation", # Historical rewind — ✅ SceneLagHistory in combat
  "room",             # Room roster/readiness/lifecycle — ✅ ChallengeRoomManager
  "net-sim",          # SimulatedLink deterministic network tests — ✅ 9 tests
] }
```

---

## 7. Current Backend vs. Elura Best Practices — Gap Analysis

### 7.1 Architecture

| Aspect | Current | Elura Best Practice | Gap |
|--------|---------|-------------------|-----|
| Gateway/World separation | ✅ `GameModule` trait + modular route registration via `build_router_from_modules()` + shared `Arc<PacketRouter>` | Separate Gateway (transport) from World (logic); or `Monolith` for dev | **Done**: Elura-inspired `WorldModule` pattern implemented; full Elura transport swap deferred |
| Transport | ✅ **ELR2 framing** over `tokio-tungstenite` with `elura.v2` subprotocol | ELR2 framing over WebSocket with `elura.v2` subprotocol | **Done** (custom binary payload inside ELR2 frames) |
| Session management | ✅ ELR2 Route 1 auth + auth deadline (15s) + heartbeat (30s) + timeout (90s) + **reconnect tokens** (periodic eviction, metrics) + **client-side reconnect** | Elura session lifecycle (auth deadline, heartbeat, reconnect tickets) | **Done**: Full reconnect flow (server token issuance + periodic eviction + metrics + client auto-reconnect with backoff) |
| Authentication | ✅ ELR2 Route 1 JSON ticket auth + HTTP ticket + reconnect token auth | `TicketService` with login/reconnect tickets, `HttpAuthApi` | **Minor**: Ticket model + reconnect tokens work |
| Rate limiting | ✅ Per-connection sliding-window (60 pkt/s) + per-command cooldowns (500ms) + packet size limit (8KB) | Gateway-level rate/burst limits per route | **Done** |
| Graceful shutdown | ✅ 10s drain timeout, rejects new connections, saves all players | Drain timeout, stop accepting new work | **Done** |

### 7.2 Realtime Gameplay

| Aspect | Current | Elura Best Practice | Gap |
|--------|---------|-------------------|-----|
| Game loop | ✅ **Elura `FixedStepClock`** with bounded catch-up (10 steps, 500ms max) | `FixedStepClock` with bounded catch-up | **Done** |
| AOI | ✅ **Elura `AoiGrid`** per Scene + `broadcast_in_range` for **all** game events + AOI-filtered initial data on connect/teleport | `AoiGrid` with entered/left deltas | **Done** (AOI-filtered broadcasting, AOI-filtered connect/teleport entity lists, ground item visibility; entered/left deltas not yet used) |
| Replication | ✅ `ObserverReplicator` wired into game loop (`process_entity_replication` every 3 ticks) — reconciles visible entities, generates spawn/despawn/keyframe/delta packets via `ReplicationSender` | `ReplicationSender/Receiver` with Spawn/Despawn/Delta/Keyframe/ACK | **Done**: Wired into game loop, complementing existing broadcast system |
| Input handling | ✅ `PlayerInputReceiver` wrapping Elura `InputReceiver` per player — validates, de-duplicates, reorders inputs by sequence (4 tests) | Queue by target tick, consume in simulation step | **Done**: Infrastructure ready, per-player receivers created on connect, tick-synced in game loop |
| Tick synchronization | ✅ Client-side `TickSynchronizer` + server heartbeat includes `server_tick`, `server_received_at`, `server_sent_at` — separates network RTT from server processing time for accurate offset (5s interval, EWMA) | `TickSynchronizer` for client tick estimation | **Done**: Full Elura-style heartbeat with server processing timestamps, client separates network RTT |
| Prediction | ✅ Client-side `PredictionBuffer` wired into `tryMove()` — instant local prediction, server reconciliation in `applyServerPosition()` with replay of unconfirmed inputs | `PredictionBuffer`, `TickSynchronizer` support | **Done**: Fully wired to player movement (predict on keypress, reconcile on server response) |
| Interpolation | ✅ Client-side `InterpolationBuffer` wired into `PixiApp.svelte` — per-entity buffer, insert on `moveEntity()`, sample in render loop with `tickSync.estimatedServerTick` | `InterpolationBuffer` for remote entity smoothing | **Done**: Fully wired to entity/NPC rendering (smooth position interpolation between server ticks) |
| Input sender | ✅ Client-side `InputSender` wired into `tryMove()` — records inputs with sequence/tick, cumulative ACK on server response, redundancy tracking ready | `InputSender` for client-side input | **Done**: Fully wired to movement (record on input, ACK on server confirm, reset on teleport/disconnect) |
| Room management | ✅ `ChallengeRoomManager` wrapping Elura `Room` — roster, readiness, capacity, leader succession, lifecycle (Open/Active/Closed) | `Room` for match/arena management | **Done**: Challenges refactored to use Elura Room (7 tests) |
| Network simulation | ✅ `SimulatedLink` tests — deterministic latency, loss, reorder, overflow, bandwidth, jitter, redundancy recovery (9 tests) | `SimulatedLink` for adverse network testing | **Done**: Test suite validates game protocol resilience |
| Lag compensation | ✅ `SceneLagHistory` + `lag_validate_target()` wired into ranged/spell combat handlers (3 tests passing) | `LagCompensationHistory` for rewind hit validation | **Done** (3-tick rewind validation for ranged and spell attacks, graceful fallback) |

### 7.3 Infrastructure

| Aspect | Current | Elura Best Practice | Gap |
|--------|---------|-------------------|-----|
| Discovery | N/A (single process) | DNS/Redis/K8s `WorldDiscovery` | N/A for monolith |
| Replay protection | Reset tickets in dev mode | `ReplayStore` (memory/Redis) | **Moderate**: Dev shortcut |
| Online presence | ✅ Player count via `ServerMetrics` + `/api/metrics` | `OnlineDirectory` with leases | **Minor**: Basic but functional |
| Push delivery | `broadcast` + `personal_tx` channels | `PushTransport` (in-process or Redis) | **Moderate**: Custom but functional |
| Metrics | ✅ `ServerMetrics` (connections, packets_in, packets_out **all channels**, packets_rejected, packets_dropped_no_char, uptime, **per-category counters**, **reconnect_tokens_active**) + `/api/metrics` JSON + `/api/metrics/prometheus` text exposition | Prometheus metrics | **Done** |
| Health/Readiness | ✅ `/api/health` + `/api/readiness` (503 during drain) | Admin server with health/readiness | **Done** |

### 7.4 Code Organization

| Aspect | Current | Elura Best Practice | Gap |
|--------|---------|-------------------|-----|
| Handler size | ✅ Decomposed into 17 sub-modules (~200 lines each) | Thin handlers + shared logic functions | **Done** |
| Middleware | ✅ Rate limiting per-connection + pre-dispatch `requires_character` validation + per-category metrics | World middleware for auth, logging, etc. | **Done** (custom middleware pattern, Elura middleware chain pending) |
| Module system | ✅ `GameModule` trait + 5 modules (`CoreGameModule`, `CommerceModule`, `SocialModule`, `GatheringModule`, `SystemModule`) + shared `Arc<PacketRouter>` built from modules at startup | `WorldModule` with typed routes | **Done**: Elura-inspired `WorldModule` pattern, modular registration |
| Error handling | ✅ `GameErrorCode` enum (30+ codes) + `GameError` integrated in all handlers: combat, inventory, bank, commerce, market, movement, dialog, admin, smelting, party, clan, crafting, challenges | Structured errors with codes | **Done** |

---

## 8. Migration Roadmap (Backend → Elura)

**Overall migration: ~100% complete** (133/134 items done). Remaining 1 item is future infrastructure scaling work that does not affect current functionality.

### Phase 1: Foundation ✅ COMPLETED (5/5)
1. ✅ Added `elura` dependency with `simulation` + `aoi` + `netcode` + `replication` + `lag-compensation` features
2. ✅ Replaced custom game loop with `FixedStepClock`
3. ✅ Replaced manual AOI with `AoiGrid` (per-scene, integrated in all handlers)
4. ✅ Decomposed `gateway/mod.rs` into 17 focused sub-modules
5. ✅ Keeping existing WebSocket transport temporarily

### Phase 2: ELR2 Protocol ✅ COMPLETED (16/16)
1. ✅ Created `elr2.rs` module with Frame encode/decode (28-byte ELR2 header, 12 tests)
2. ✅ WebSocket server negotiates `elura.v2` subprotocol with backwards compat
3. ✅ ELR2 Route 1 (AUTHENTICATE): JSON ticket auth before game packets
4. ✅ ELR2 Route 2 (HEARTBEAT): Ping/pong at framing level
5. ✅ ELR2 Route 100 (GAME): Existing binary protocol inside ELR2 payload
6. ✅ All gateway handlers use `send_to_client()` for transparent ELR2 wrapping
7. ✅ Broadcast and personal_tx channels auto-wrap in ELR2 Push frames
8. ✅ Frontend `@openao/protocol` has `elr2.ts` encoder/decoder
9. ✅ Frontend `gameSession.svelte.ts` negotiates ELR2 subprotocol, sends auth frame, wraps game packets
10. ✅ Legacy protocol fallback: clients without `elura.v2` work unchanged
11. ✅ Auth deadline (15s timeout for unauthenticated ELR2 connections)
12. ✅ Server-initiated heartbeat (30s interval, ELR2 or WS ping)
13. ✅ Client timeout (90s inactivity disconnects)
14. ✅ ELR2 subprotocol negotiation passed from handshake to GameSession constructor
15. ✅ Zero compiler warnings (all dead code annotated with `#[allow(dead_code)]`)
16. ✅ All Rust tests passing (51/51), all protocol tests passing (55/55)

### Phase 3: Architecture Split ✅ COMPLETED (10/10)
1. ✅ Created `routes/mod.rs` — typed `PacketRouter` with `RouteInfo`, `RouteCategory`, per-packet metadata
2. ✅ Integrated `PacketRouter` into `GameSession` for route-aware dispatch
3. ✅ All 30+ packet types registered with name and category
4. ✅ `PacketRouter` actively used in `handle_legacy_binary` for route-aware debug tracing (logs route name + category)
5. ✅ Run loop refactored: eliminated duplicated `tokio::select!` block using dummy broadcast channel pattern
6. ✅ `PacketRouter` refactored to `HashMap<u8, RouteInfo>` for O(1) lookups (was linear scan `Vec`)
7. ✅ `RouteInfo` extended with `requires_character` flag — middleware auto-drops packets for routes requiring a connected character when `entity_id` is None (pre-dispatch validation)
8. ✅ Per-route-category packet metrics (`CategoryCounters`) tracked at dispatch level and exposed in JSON+Prometheus APIs
9. ✅ `GameModule` trait + 5 domain modules (`CoreGameModule`, `CommerceModule`, `SocialModule`, `GatheringModule`, `SystemModule`) — Elura-inspired `WorldModule` pattern for modular route registration
10. ✅ `build_router_from_modules()` constructs shared `Arc<PacketRouter>` at startup — router shared across all sessions (was per-session `PacketRouter::new()`), Route 100 dispatch decomposed via module system

### Phase 4: Session Management ✅ COMPLETED (17/17)
1. ✅ `rate_limit.rs` — per-connection sliding-window rate limiter (60 pkt/s default)
2. ✅ `rate_limit.rs` — `CommandRateLimiter` with named commands and per-command cooldowns (500ms)
3. ✅ Rate limiting integrated into `GameSession::handle_message()` (drops packets over limit)
4. ✅ `CommandRateLimiter` integrated into all expensive handlers: market, crafting, bank deposit/withdraw, buy/sell, smelting, challenges
5. ✅ `reconnect.rs` — `ReconnectManager` with short-lived tokens (120s TTL, UUID-based)
6. ✅ Reconnect tokens issued automatically on player disconnect
7. ✅ `ReconnectState` captures full session state for seamless resume
8. ✅ ELR2 auth supports both `{"ticket": "..."}` and `{"reconnect_token": "..."}` payloads
9. ✅ Auth deadline (15s) + server heartbeat (30s) + client timeout (90s) from Phase 2
10. ✅ `ServerMetrics` passed to `GameSession` — tracks `packets_in`/`packets_out`/`packets_rejected` per session
11. ✅ Packet size validation: oversized packets (>8KB) rejected with metrics tracking
12. ✅ **Client-side reconnect flow**: Frontend auto-reconnects with token on unexpected disconnect (3 attempts, exponential backoff)
13. ✅ Server sends reconnect token to client via ELR2 Push on Route 1 after character connect
14. ✅ Reconnect auth response includes new token for chained reconnect sessions
15. ✅ Frontend shows "Reconectando..." overlay during reconnect attempts, falls back to disconnect if all fail
16. ✅ **Periodic token eviction**: `ReconnectManager::evict_expired()` called every 60s from game loop, removes stale tokens
17. ✅ **Reconnect token metrics**: `reconnect_tokens_active` gauge exposed in `/api/metrics` JSON and Prometheus

### Phase 5: Advanced Gameplay ✅ COMPLETED (16/16)
1. ✅ `broadcast_in_range()` — AOI-filtered broadcasting: NPC AI movement, chase, attack, death, and respawn packets sent only to players within view range instead of all players on the map
2. ✅ `Scene::broadcast_in_range()` queries `AoiGrid` for nearby entities and sends via `personal_tx` channels, with fallback to full broadcast on AOI lock failure
3. ✅ **Full AOI migration**: All game broadcasts migrated to `broadcast_in_range` — combat (melee/ranged/spell damage, vitals, death, projectiles, FX, loot drops, death item drops), movement (player move, heading change), connect/disconnect (character announce, delete), teleport (new map announcements, equipment sync, color, **delete on old map**), inventory (equip visual broadcast, use item sounds, ground item pickup), dialog (local chat, meditation FX, revive), faction/bail color changes, fishing/harvesting sounds, admin NPC spawn/remove/reset map, commerce sacerdote revive. Only global broadcasts (`/global`, `GLOBAL_NOTICE`, global GM) remain as full-map broadcasts by design.
4. ✅ `ObserverReplicator` wrapping Elura `ReplicationSender` — per-observer entity replication with spawn/despawn/keyframe/delta/ACK (4 tests passing)
5. ✅ `SceneLagHistory` wrapping Elura `LagCompensationHistory` — 64-tick history, 30-tick max rewind for server-side hit validation (3 tests passing)
6. ✅ `SceneLagHistory` integrated into `Scene` struct and `process_tick` game loop — combat snapshots recorded every tick for all players and NPCs across all active scenes
7. ✅ Combat `find_target` refactored to use `entities_in_range()` (AOI grid) instead of full linear scan of all entities
8. ✅ **AOI-filtered initial data**: Connect and teleport only send entities (players, NPCs, ground items) within AOI range to the connecting/teleporting player, instead of all entities on the map
9. ✅ **Lag-compensated ranged/spell combat**: `lag_validate_target()` uses `SceneLagHistory` to rewind 3 ticks and verify target was alive and in range at the time of attack; graceful fallback if no history available
10. ✅ **Ground items AOI on teleport**: Teleport now filters ground items by view range (matching connect behavior)
11. ✅ **Market listing expiry in game loop**: `expire_market_listings` runs every 60s via `process_tick`, returning expired items to sellers automatically (was only triggered on market access)
12. ✅ **Entity vitals broadcast completeness**: All HP/mana changes now broadcast `entity_vitals_delta` to AOI observers — NPC AI attacks, player-vs-NPC attacks, heal spells, sacerdote revive/heal, `/revivir` respawn, and admin `/devrevivir` all send vitals updates to nearby players
13. ✅ **NPC AI hit feedback**: NPC melee attacks now send damage and death messages to the target player via console
14. ✅ **ObserverReplicator wired into game loop**: `process_entity_replication()` runs every 3 ticks — reconciles visible entity sets per observer, generates spawn/despawn/keyframe/delta packets via `ReplicationSender`, auto-ACKs batches. Per-player replicators and input receivers created on connect, removed on disconnect, transferred on teleport.
15. ✅ **PlayerInputReceiver (tick-aligned input)**: `input_queue.rs` wraps Elura `InputReceiver` — per-player server-side input validation, sequence-based de-duplication, reorder window (256), bounded past/future tick acceptance. `update_input_receiver_ticks()` syncs all receivers with game tick every frame. 4 tests passing.
16. ✅ **Client-side TickSynchronizer**: `tickSync.ts` estimates server tick from ELR2 heartbeat probe RTT — EWMA-smoothed offset with bounded correction (4 ticks max), 5s probe interval. Server heartbeat response includes `server_tick` from `ServerMetrics.current_tick` (updated by game loop). `GameSession` sends probes, processes responses, and exposes `estimatedServerTick` / `recommendedInputTick`.

### Phase 6: Production Readiness (40/41 — ✅ COMPLETED)
1. ✅ `ServerMetrics` — atomic counters for connections, packets_in, packets_out (all channels: direct, broadcast, personal), packets_rejected, uptime
2. ✅ `GET /api/health` — simple health check
3. ✅ `GET /api/readiness` — returns 503 during shutdown drain
4. ✅ `GET /api/metrics` — JSON metrics (uptime, connections, players, NPCs, scenes, packet counts, rejected packets, per-category, reconnect tokens)
5. ✅ Graceful shutdown with 10s drain timeout (rejects new connections, waits for active to close)
6. ✅ `error.rs` — structured `GameErrorCode` enum (30+ codes) + `GameError` type with console packet conversion
7. ✅ `GameError` integrated into combat handlers (safe zone, dead, mana, target not found)
8. ✅ `GameError` integrated into inventory handlers (slot empty, inventory full, item not found)
9. ✅ `GameError` integrated into bank handlers (insufficient gold, rate limit)
10. ✅ `GameError` integrated into commerce handlers (insufficient gold, invalid slot, insufficient items, rate limit, no trade target)
11. ✅ `GameError` integrated into movement handlers (PvP map change blocked)
12. ✅ `GameError` integrated into spell attack handlers (safe zone, target not found — migrated from console msgs)
13. ✅ `GameError` integrated into dialog handlers (whisper player not found)
14. ✅ `GameError` integrated into admin handlers (kick, telepuser, bring, ban, mute, inspect — player not found)
15. ✅ Packet size validation (8KB max) with rejected packet metrics
16. ✅ Server stats logged on shutdown (uptime, total connections)
17. ✅ `GET /api/metrics/prometheus` — Prometheus text exposition format (uptime, connections, packets, players, NPCs, scenes, shutdown status, per-category counters)
18. ✅ `GameError` integrated into smelting handlers (rate limit, recipe not found, insufficient minerals)
19. ✅ `GameError` integrated into party handlers (player not found, not leader, party full)
20. ✅ `GameError` integrated into clan handlers (not in clan, not leader, player not found, request not found)
21. ✅ `GameError` integrated into crafting handlers (rate limit, recipe not found, insufficient materials)
22. ✅ `GameError` integrated into challenges handlers (rate limit, invalid ID, not found, unknown action)
23. ✅ `GameError` integrated into market handlers (rate limit, insufficient gold, invalid slot, item not found, inventory full)
24. ✅ **Full character state persistence**: `save_character_state` now persists all mutable fields — max_hp, max_mana, dead, min_hit, max_hit, all 4 attributes, equipment visual IDs (id_head, id_body, id_helmet, id_weapon, id_shield), navegando, bank_gold, id_clase, faction_rank, faction_score, in addition to existing position/gold/level/exp/faction/criminal/home fields
25. ✅ **Per-category packet metrics**: `CategoryCounters` in `ServerMetrics` tracks packets_in per `RouteCategory` (auth, movement, combat, dialog, inventory, commerce, social, crafting, gathering, bank, market, challenge, admin, system), exposed in `/api/metrics` JSON and `/api/metrics/prometheus`
26. ✅ **Pre-dispatch middleware**: `requires_character` validation at dispatch level drops packets for routes needing a connected character, with `packets_dropped_no_char` counter exposed in metrics
27. ✅ **id_clase persistence**: `save_character_state` now includes `id_clase`, ensuring `/cambiarclase` admin command persists across disconnects
28. ✅ **Faction rank/score persistence**: Added `faction_rank` and `faction_score` columns to characters table (ALTER migration), loaded from DB on connect, saved on disconnect/worldsave/shutdown
29. ✅ **Class name fix**: Corrected `get_class_name()` mapping (was: 5=Ladron, 6=Bardo, 7=Druida, 8=Paladin → now: 5=Bardo, 6=Druida, 7=Paladin, 8=Cazador) to match class definitions
30. ✅ **Class level bonus fix**: Corrected `class_level_bonus()` — Bardo (5) now gains mana per level (was 0 like "Ladrón"), Paladin (7) gets correct HP/mana, Cazador (8) gets own progression
31. ✅ **Drop item creates ground item**: `handle_drop_item` now spawns item on the ground near player position with AOI-filtered `build_render_item` broadcast (was only removing from inventory)
32. ✅ **Entity vitals broadcast audit**: Systematic audit of all HP/mana mutation points ensured `entity_vitals_delta` is broadcast to AOI observers for: NPC AI attacks on players, player melee/ranged/spell attacks on NPCs, heal spell self-casts, sacerdote NPC revive/heal, `/revivir` respawn, admin `/devrevivir`, HP potions, mana potions
33. ✅ **NPC attack console feedback**: Players now receive "X te golpea por Y de daño" messages when attacked by NPCs
34. ✅ **USE_ITEM_U deserialization fix**: Server was reading `get_int()` (4 bytes) but frontend sends `writeByte()` (1 byte) — corrected to `get_byte()` matching `USE_ITEM_CLICK`
35. ✅ **Potion vitals broadcast**: HP and Mana potions now broadcast `entity_vitals_delta` to AOI observers so other players see health/mana bar updates when a player drinks a potion
36. ✅ **Challenges class name consistency**: Removed hardcoded class name mappings in `challenges.rs`, now uses centralized `get_class_name()` to ensure consistency with corrected class definitions
37. ✅ **Removed stale `#[allow(dead_code)]`**: `entities_in_range` on `Scene` is actively used; removed unnecessary `#[allow(dead_code)]` annotation
38. ✅ **Navegando + bank_gold persistence**: `save_character_state` now includes `navegando` (boat state) and `bank_gold` fields, persisted on disconnect, worldsave, and graceful shutdown
39. ✅ **Release binary optimized**: Cargo.toml `[profile.release]` with LTO, `codegen-units=1`, `strip=true`, `opt-level=3`
40. ✅ **Full disconnect save**: `handle_disconnect` saves all mutable fields + issues reconnect token + broadcasts delete to AOI + cleans up party state
41. 🔧 Evaluate Redis adapters if scaling beyond single process

### Phase 7: Elura Full Integration ✅ COMPLETED (7/7)
1. ✅ **Enabled `room` + `net-sim` Elura features** in Cargo.toml — now uses 7 features: simulation, aoi, netcode, replication, lag-compensation, room, net-sim
2. ✅ **ChallengeRoomManager (Elura Room)**: `gameplay/rooms.rs` wraps `elura::gameplay::room::Room` — capacity, minimum_to_start, leader succession, lifecycle state. Replaces custom `ChallengeManager`. Gateway handler (`gateway/challenges.rs`) refactored to use new manager. 7 tests passing.
3. ✅ **Enhanced TickSync heartbeat**: Server heartbeat response now includes `server_received_at` and `server_sent_at` (monotonic ms since uptime_start) alongside `server_tick`. Client `tickSync.ts` separates network RTT from server processing time for accurate one-way delay estimation. Exposes `totalRttMs`, `networkRttMs`, `oneWayDelayMs`.
4. ✅ **SimulatedLink network tests**: `gameplay/net_sim.rs` — 9 deterministic tests using `elura::gameplay::net_sim::SimulatedLink` covering fixed latency, total/partial packet loss, reordering, queue overflow, bandwidth throttling, deterministic replay, redundant input survival, and jitter.
5. ✅ **InterpolationBuffer (TS) — WIRED**: `frontend-svelte/src/lib/game/network/interpolation.ts` — port of Elura's `InterpolationBuffer` to TypeScript. Wired into `gameState.moveEntity()` (insert on server update) and `PixiApp.svelte` `renderRemoteEntities()`/`renderNpcs()` (sample with `tickSync.estimatedServerTick` for smooth interpolated positions). Per-entity buffers created on first move, cleaned on entity removal/teleport/disconnect.
6. ✅ **PredictionBuffer (TS) — WIRED**: `frontend-svelte/src/lib/game/network/prediction.ts` — port of Elura's `PredictionBuffer` to TypeScript. Wired into `GameView.svelte` `tryMove()` (instant local prediction) and `gameState.applyServerPosition()` (reconciliation on MOVE_ENTITY/ACT_POSITION for self — replays unconfirmed inputs on top of authoritative state). Reset on teleport/connect/disconnect.
7. ✅ **InputSender (TS) — WIRED**: `frontend-svelte/src/lib/game/network/inputSender.ts` — port of Elura's `InputSender` to TypeScript. Wired into `tryMove()` (record with tick/heading) and `applyServerPosition()` (cumulative ACK). Redundancy tracking infrastructure ready for protocol extension. Reset on teleport/connect/disconnect.

### Phase 8: Netcode Wiring ✅ COMPLETED (6/6)
1. ✅ **InterpolationBuffer wired to entity rendering**: Per-entity `InterpolationBuffer` in `gameState.interpolationBuffers` — insert on `moveEntity()` with estimated server tick, sample in `PixiApp.svelte` `renderRemoteEntities()`/`renderNpcs()` using `tickSync.estimatedServerTick` for smooth alpha-blended position interpolation. Buffers cleaned on entity removal, teleport, and disconnect.
2. ✅ **PredictionBuffer wired to player movement**: `gameState.predictionBuffer` records predicted state in `tryMove()` (instant local position update), `applyServerPosition()` reconciles on server `MOVE_ENTITY`/`ACT_POSITION` for self by replaying unconfirmed inputs on top of authoritative state. Monotonic move tick counter for consistent ordering.
3. ✅ **InputSender wired to movement input**: `gameState.inputSender` records inputs in `tryMove()` with tick/heading, cumulative ACK in `applyServerPosition()`. Redundancy tracking infrastructure ready for protocol extension when backend supports `InputReceiver` redundant packets.
4. ✅ **Frontend netcode unit tests**: 26 tests in `frontend-svelte/src/lib/game/network/__tests__/` — `interpolation.test.ts` (9 tests: insert/sample roundtrip, late insertion, holdingNewest, capacity, reset, adaptive delay), `prediction.test.ts` (8 tests: record/reconcile, replay correction, backwards tick rejection, capacity), `inputSender.test.ts` (9 tests: record/packet, redundancy window, cumulative ACK, sequence tracking, capacity, reset).
5. ✅ **Buffer cleanup on teleport/disconnect/connect**: `resetNetcodeBuffers()` clears all interpolation buffers, resets prediction buffer and input sender on `readTelepMe` (teleport), `readGetMyCharacter` (new connection), and `gameState.reset()` (disconnect). Entity-level cleanup on `removeEntity()`/`removeNpc()`.
6. ✅ **AGENTS.md updated**: Primitives marked as wired, test counts updated (136 total: 59 Rust + 51 protocol + 26 frontend netcode), netcode architecture documented.

### Phase 9: Netcode Polish ✅ COMPLETED (8/8)
1. ✅ **Client-side movement rate limiting**: `tryMove()` in `GameView.svelte` throttled to 60 TPS (`MOVE_INTERVAL_MS = 1000/60`) preventing packet flood from fast key-repeat.
2. ✅ **TickSynchronizer unit tests**: 13 tests in `tickSync.test.ts` covering initial state, local tick, valid/invalid samples, server timestamps for RTT separation, offset smoothing, non-negative estimatedServerTick, recommendedInputTick, reset, and edge cases.
3. ✅ **Server echoes moveId in ACT_POSITION**: `build_act_position_with_move_id()` includes `move_id: u16` — client `applyServerPosition()` uses it for precise `PredictionBuffer.reconcile()` instead of fallback heuristic.
4. ✅ **Server includes serverTick in MOVE_ENTITY**: `build_move_entity_packet_with_tick()` appends `server_tick: u16` — client `moveEntity()` passes real server tick to `InterpolationBuffer.insert()` for accurate interpolation timing.
5. ✅ **ObserverReplicator sends equipment visuals on spawn**: `process_entity_replication` `Spawn` branch now calls `send_entity_visuals()` to send `CHANGE_BODY`, `CHANGE_ROPA`, `CHANGE_WEAPON`, `CHANGE_HELMET`, `CHANGE_SHIELD` packets after character packet.
6. ✅ **PlayerInputReceiver validates movement in backend**: `handle_movement()` wraps movement logic with `InputReceiver.receive()` — validates, de-duplicates, and reorders client inputs by sequence. Rejected/duplicate inputs are dropped before movement processing.
7. ✅ **Redundant input protocol**: `sendPosition()` appends redundant inputs from `InputSender.packet()` to `POSITION` packet (count:byte + [seq:short, heading:byte]*). Backend `POSITION` handler reads redundant frames and feeds them to `PlayerInputReceiver` for loss recovery.
8. ✅ **AGENTS.md v13**: Updated test counts (149 total: 59 Rust + 51 protocol + 39 frontend netcode), packet protocol reference updated (MOVE_ENTITY +serverTick, ACT_POSITION +moveId, POSITION +redundant inputs), Phase 9 documented.

### Phase 10: Hardening & Quality ✅ COMPLETED (8/8)
1. ✅ **Server-side tile collision**: Player and NPC movement validated against `GameData::is_blocked_tile()` — blocked terrain tiles reject movement server-side (anti-cheat).
2. ✅ **Map bounds from terrain data**: `GameData::get_map_bounds()` returns real `(width, height)` from `MapTerrain`; replaces all hardcoded `1-100` clamps in `handle_movement` and `process_npc_ai`.
3. ✅ **Ban/mute persistence**: New SQLite tables `bans` and `mutes` with full CRUD (`add_ban`, `remove_ban`, `load_all_bans`, `add_mute`, `remove_mute`, `is_muted`). Bans loaded on server startup; mutes loaded on character connect. Admin `/ban`, `/unban`, `/mute` commands persist to DB.
4. ✅ **Party leadership transfer**: On leader disconnect, leadership transfers to the next member in the party instead of disbanding. Party only disbands when ≤1 member remains.
5. ✅ **CHANGE_ROPA in send_entity_visuals**: `send_entity_visuals()` now sends `CHANGE_ROPA` (head/ropa visual ID) alongside CHANGE_BODY, CHANGE_WEAPON, CHANGE_HELMET, CHANGE_SHIELD — complete visual representation on entity spawn.
6. ✅ **Entity ID recycling**: `next_id()` wraps naturally at `u32::MAX` (AtomicU32 overflow) and skips ID 0 (reserved sentinel for broadcasts). Safe for long-running servers.
7. ✅ **Dev ticket reuse flag**: `consume_game_ticket` now gated behind `OPENAO_DEV_TICKETS=1` environment variable. Off by default — tickets are single-use in production.
8. ✅ **Legacy password auto-migration**: On successful login with plaintext password, the hash is transparently upgraded to argon2 in-place. Existing sessions unaffected.

### Phase 11: Enhancement Pass (A–F) ✅ COMPLETED (30/30)

#### Phase A: Code Quality & Architecture (6/6)
1. ✅ **Command registry refactoring**: Replaced monolithic `if-else` command chain in `dialog.rs` with a `CommandRegistry` pattern for O(1) command lookup
2. ✅ **Inventory caching**: In-memory inventory cache (`InventoryCache`) reduces SQLite queries on frequent item operations (pickup, drop, use, equip)
3. ✅ **Typed error handling for gateway handlers**: Custom `HandlerResult` type replaces `Box<dyn Error>` for gateway handlers, enabling pattern-matched error recovery
4. ✅ **Persistence layer abstraction**: Decomposed monolithic `persistence/mod.rs` into domain sub-modules (`characters.rs`, `inventory.rs`, `bank.rs`, `market.rs`, `accounts.rs`, `quests.rs`, `pets.rs`, `achievements.rs`)
5. ✅ **CI pipeline modernization**: GitHub Actions workflows updated for Rust backend + SvelteKit frontend + protocol tests
6. ✅ **IP-based rate limiting**: Per-IP connection rate limiting (separate from per-session packet rate), prevents connection flooding

#### Phase B: Infrastructure Improvements (4/4)
1. ✅ **Structured logging with correlation IDs**: Each `GameSession` gets a unique correlation ID, propagated through all handlers for cross-handler request tracing
2. ✅ **SQLite auto-backup**: Periodic `.backup` command executed from game loop for data safety (configurable interval)
3. ✅ **Packet batching**: `sink.feed()` + `sink.flush()` pattern for efficient multi-packet WebSocket sends (reduces syscalls)
4. ✅ **Packet priority system**: Critical/High/Normal/Low priority levels for congestion management, `outbound_pressure` counters in `ServerMetrics`

#### Phase C: Gameplay Systems (6/6)
1. ✅ **Buff system**: Tick-based buffs (`BuffManager`) with agility/strength/speed types, duration tracking, magnitude, auto-expiry in game loop. Integrated with potion consumption.
2. ✅ **Navigation system (boats)**: `/embarcar`/`/desembarcar` commands, water tile detection, restricted movement on water/land tiles, visual change (boat sprite), `navegando` flag persisted
3. ✅ **P2P Trading system**: `/comerciar name` to request trade, offer gold, confirm/cancel, atomic gold swap, cleanup on disconnect, cross-scene validation
4. ✅ **Admin invisibility**: `/invisible` toggle, AOI-filtered (invisible players excluded from `broadcast_in_range` for non-admins), persisted flag
5. ✅ **Jail system**: `/carcel name time` command, `jail_until_ms` field, blocks `/tp`, `/hogar`, auto-release when timer expires, feedback messages
6. ✅ **IP ban system**: `/banip`/`/unbanip` commands, `ip_bans` SQLite table, `banned_ips` DashMap loaded on startup, check during WebSocket connection accept

#### Phase D: Backend Enhancements (4/4)
1. ✅ **Game data hot-reload**: `/recargar` command triggers `RwLock<Arc<GameData>>` swap — zero-downtime reload of objects, NPCs, spells, maps, crafting recipes. `gd()` accessor throughout codebase.
2. ✅ **Broadcast deduplication**: `broadcast_announced` set in `ObserverReplicator` prevents redundant spawn packets when `process_entity_replication` and `broadcast_in_range` both announce the same entity
3. ✅ **Enhanced NPC AI**: NPC chase and attack improved with AOI grid queries, NPC attack feedback (damage messages), death notifications
4. ✅ **Spell cooldown integration**: `CooldownManager` checked in `handle_attack_spell` before casting; prevents spell spam

#### Phase E: Frontend Polish (4/4)
1. ✅ **PixiApp decomposition**: Refactored monolithic `PixiApp.svelte` into focused rendering sub-modules for maintainability
2. ✅ **Minimap component**: Real-time minimap overlay showing player position, nearby entities, NPCs on the current map
3. ✅ **Toast notification system**: Non-blocking toast notifications for game events (level up, achievement unlock, quest progress)
4. ✅ **Particle effects overlay**: `ParticleOverlay.svelte` for visual spell/combat effects (spell impacts, level-up sparkles)

#### Phase F: Advanced Game Systems (6/6)
1. ✅ **Quest system (F1)**: 8 quests loaded from `data/quests.json`, `QuestRegistry` + `PlayerQuestLog` with 5 objective types (kill_npc, collect_item, visit_map, talk_npc, reach_level), gold/exp/item rewards, prerequisites, max 10 active. Commands: `/misiones`, `/mision aceptar|abandonar|completar`. SQLite persistence (`character_quests_active`, `character_quests_completed`). 9 tests.
2. ✅ **Pet system (F2)**: `PetManager` with max 5 pets per player, `Pet` struct (name, pet_type, level, exp, hp, active). Commands: `/mascotas`, `/invocar id`, `/despachar`, `/liberar id`. SQLite persistence (`character_pets`). 8 tests.
3. ✅ **Territory control (F3)**: `TerritoryManager` with 5 capturable zones tied to specific maps. `Territory` struct (map_id, owner_clan, capture_progress, capture_threshold, capturing_clan, bonuses). Commands: `/territorios`. 5 tests.
4. ✅ **Spell cooldowns (F4)**: `CooldownManager` with per-spell tracking, tier-based default cooldowns (1.5s/3s/5s/8s). Integrated into combat `handle_attack_spell`. 6 tests.
5. ✅ **Weather + Day/Night (F5+F6)**: Client-side `WeatherSystem` (rain/snow/fog/storm particles) + `DayNightCycle` (20min cycle: dawn/day/dusk/night tint overlay). Both rendered via `WeatherOverlay.svelte` canvas components.
6. ✅ **Achievements + Leaderboard (F7+F8)**: `AchievementTracker` with 13 achievements across 10 condition types, `PlayerStats` tracking, SQLite persistence. Real-time leaderboard: top-5 online players broadcast every 30s from game loop. 6 tests.

#### Post-Phase: Full Integration Wiring
1. ✅ **Quest advancement wired to game handlers**: `advance_quest_kills()` called on NPC kill in `attack_npc`, `advance_quest_collect()` called on item pickup in `handle_agarrar_item`, `advance_quest_visit_map()` called on teleport in `do_teleport`
2. ✅ **Spell cooldowns enforced in combat**: `CooldownManager.is_ready()`/`trigger()` checked in `handle_attack_spell` before spell cast — prevents spam, shows remaining cooldown
3. ✅ **Achievement stat tracking wired to handlers**: `total_npc_kills` incremented on NPC kill, `total_maps_visited` incremented on teleport, `check_and_unlock()` called after kill and level-up with "Logro desbloqueado" messages
4. ✅ **Territory bonus integrated in combat rewards**: `bonus_exp_pct`/`bonus_gold_pct` applied to XP/gold rewards when player's clan owns the territory of the current map
5. ✅ **Zero warnings build**: All `dead_code` warnings resolved via `#[allow(dead_code)]` on gameplay module API surfaces intended for future integration

### Phase 12: Parity & Optimization Pass ✅ COMPLETED (36/36)

Comprehensive audit porting missing features from the original Node.js/React stack and optimizing the new Rust/SvelteKit stack.

#### P0: Protocol Parity (6/6)
1. ✅ **Batch snapshot packets**: `SELF_MAP_META_DELTA` (map_id:short, map_name:string, pk_flag:byte), `GLOBAL_NOTICE` (text:string), `ACT_MY_LEVEL` (level:short) — new opcodes added to both TS and Rust protocol
2. ✅ **Party/Clan state sync packets**: `PARTY_STATE` (count:byte, [name:string, hp:short, maxHp:short]*), `CLAN_STATE` (clanName:string, clanId:string) — full sync on connect and changes
3. ✅ **Panel snapshot packet**: `PANEL_SNAPSHOT` (gold:int, exp:int, expNext:int, level:short, hp:short, maxHp:short, mana:short, maxMana:short, str:short, agi:short, int:short, con:short) — replaces 4 separate initial-state packets
4. ✅ **Frontend packet handlers registered**: All new incoming packets (`selfMapMetaDelta`, `globalNotice`, `actMyLevel`, `partyState`, `clanState`, `panelSnapshot`) wired in `registerPacketHandlers.ts` with gameState integration
5. ✅ **Rust opcodes synced**: All new packet IDs added to `game-server-rs/crates/protocol/src/opcodes.rs` matching TypeScript side
6. ✅ **Backend builders**: `build_self_map_meta_delta`, `build_global_notice`, `build_act_level`, `build_party_state`, `build_clan_state`, `build_panel_snapshot` implemented in `replication/mod.rs` and `gateway/packets.rs`

#### P1: Combat Parity (6/6)
1. ✅ **CC system (paralysis/immobilization)**: `paralizado` and `inmovilizado` fields on `PlayerState`, tick-based expiry via `process_cc_expiry` in game loop, blocks movement (`handle_movement`) and attacks (`handle_attack_melee/range/spell`)
2. ✅ **Spell effects for CC**: Spells with `efecto: "paralizar"` or `efecto: "inmovilizar"` apply CC based on `duration_ticks` from spell data
3. ✅ **Invisibility via spell**: Spells with `efecto: "invisibilidad"` set `invisible=true` on caster, auto-removed when attacking or receiving damage
4. ✅ **Dead world restrictions**: Dead players cannot attack (melee/ranged/spell), use items, or fish/harvest — all handlers check `dead` flag with appropriate error messages
5. ✅ **Safety toggle (seguro)**: `seguro_activado` flag prevents attacking players, `/seguro` command toggles, console feedback
6. ✅ **Clan safety toggle**: `seguro_clan_activado` flag prevents attacking clan members, `/seguroclan` command toggles

#### P2: API Parity (6/6)
1. ✅ **World Builder API**: `POST /api/world/maps/{map_id}/spawns`, `POST /api/world/maps/{map_id}/tiles/exits`, `POST /api/world/maps/{map_id}/metadata` — map editing endpoints
2. ✅ **Arenas API**: `GET/POST /api/arenas` for arena management with persistence
3. ✅ **Clans HTTP API**: `GET /api/clans`, `GET /api/clans/{id}` — clan listing and detail endpoints
4. ✅ **Runtime Config API**: `GET/POST /api/runtime-config` — server-side `RuntimeConfig` struct for double exp/gold toggles
5. ✅ **Character Settings API**: `GET/POST /api/character-settings/{char_id}` — per-character client preferences persisted to `character_settings` SQLite table
6. ✅ **Wiki SSR data endpoint**: Backend `/api/wiki` endpoint returns items, NPCs, spells from GameData for SvelteKit SSR consumption

#### P3: Balance & Game Rules (6/6)
1. ✅ **Balance module**: `gameplay/balance.rs` — `CombatStats` struct, `compute_player_stats` (class multipliers, weapon/armor scaling, attribute bonuses), `compute_damage` (melee with defense reduction), `compute_spell_damage` (magic resistance), `compute_exp_for_kill` (level diff scaling), gold clamping (5 tests)
2. ✅ **Dual faction scores**: `faction_score_armada` and `faction_score_caos` tracked independently per character, persisted via ALTER TABLE migration, loaded/saved correctly
3. ✅ **Item tiers & restrictions**: `item_tier`, `class_restriction`, `race_restriction`, `magic_min`/`magic_max` fields on `ObjectData` — equip checks validate class/race, magic items apply bonus stats
4. ✅ **Floor item auto-cleanup**: Ground items have `created_at_tick` timestamp, `process_ground_item_cleanup` runs in game loop every 30s, items older than 10800 ticks (180s) are auto-removed with AOI-filtered `build_delete_ground_item` broadcast
5. ✅ **Exp/gold scaling**: `compute_exp_for_kill` applies diminishing returns for high level-diff kills, `compute_gold_clamp` limits gold drops by attacker level
6. ✅ **Balance integration**: `compute_player_stats` integrated into combat handlers for data-driven melee/ranged/spell damage calculation

#### P4: Frontend Polish (6/6)
1. ✅ **NPC Inspector modal**: `NpcInspectorModal.svelte` — click NPC to inspect stats (HP, body/head IDs, exp reward, loot table), toggled via admin command
2. ✅ **Admin Intervals modal**: `AdminIntervalsModal.svelte` — toggle double exp/gold via REST API (`/api/runtime-config`) from in-game admin UI
3. ✅ **Overview modal**: `OverviewModal.svelte` — character overview showing level, gold, map name, HP/mana bars, attributes, faction info
4. ✅ **Debug overlay**: `DebugOverlay.svelte` — real-time display of position, map, entity/NPC counts, server tick, RTT, vitals (toggled with F3 key)
5. ✅ **Social meta tags**: Open Graph (`og:title`, `og:description`, `og:image`) and Twitter Card meta tags in SvelteKit root `+layout.svelte`
6. ✅ **Frontend error fixes**: Fixed `outgoingRequests.ts` (missing `clientTick` argument to `InputSender.packet()`), fixed `assetStore.svelte.ts` (unsafe `Record<string, number>` casts replaced with proper `keyof` type assertions)

#### P5: Multi-Character & Admin (6/6)
1. ✅ **Multi-character per account**: `GET /api/characters/{account_id}` lists all characters for an account, enabling character selection UI
2. ✅ **Character deletion**: `DELETE /api/characters/{char_id}` soft-deletes a character (cascades inventory, bank, quest, pet, achievement data)
3. ✅ **Moderation REST API**: `POST /api/admin/ban`, `POST /api/admin/unban`, `POST /api/admin/mute`, `POST /api/admin/unmute`, `POST /api/admin/ip-ban`, `POST /api/admin/ip-unban` — admin-authenticated moderation endpoints
4. ✅ **Game Data Admin API**: `GET /api/admin/game-data/objects`, `GET /api/admin/game-data/npcs`, `GET /api/admin/game-data/spells` — browse game data for admin tools
5. ✅ **Vault/storage REST stub**: Account-level vault storage API endpoints scaffolded for future item vault feature
6. ✅ **Admin auth middleware**: API admin endpoints validate `is_admin` flag from account, return 403 for non-admin access

#### Optimizations (6/6 — New Stack Advantages)
1. ✅ **Zero-copy batch send**: `send_batch_to_client()` method on `GameSession` collects packets into `Vec<Vec<u8>>`, sends via `sink.feed()+flush()` — reduces connect burst from 30+ individual sends to 2 batched flushes. `PacketWriter` enhanced with `with_capacity()` and `with_packet_id_and_capacity()` for precise pre-allocation.
2. ✅ **SQLite WAL + prepared stmt caching**: `SqliteConnectOptions` configured with `journal_mode(Wal)`, `busy_timeout(5s)`, `cache_size(-8000)` (8MB), `synchronous(NORMAL)`, `temp_store(MEMORY)`, `mmap_size(256MB)`, `statement_cache_capacity(256)` — significantly improved read concurrency and query latency.
3. ✅ **SvelteKit SSR for wiki**: Created `+page.server.ts` for wiki section — fetches items/NPCs/spells from backend `/api/wiki` endpoint, passes data to component via server load function. Wiki content is now SEO-friendly and server-rendered.
4. ✅ **Svelte 5 runes verification**: Confirmed entire frontend uses `$state`, `$derived`, `$effect` exclusively — no legacy `writable()`/`readable()` stores found.
5. ✅ **Pixi.js 8 WebGPU preference**: Added `preference: "webgpu"` to `Application.init()` — Pixi.js 8 attempts WebGPU backend first with automatic WebGL fallback for unsupported browsers.
6. ✅ **Protocol batching optimization**: Confirmed `feed()+flush()` pattern in game loop broadcast path. Extended to connect burst via `send_batch_to_client()`. Precise capacity hints on high-frequency packets (`MOVE_ENTITY` 10 bytes, `SELF_VITALS_DELTA` 9 bytes, `ACT_POSITION` 9 bytes) eliminate buffer reallocations.

### Phase 13: Combat Fidelity Pass ✅ COMPLETED (4/4)

Exact port of all combat formulas and balance data from the Node.js `game.ts` and `balance.ts` originals.

1. ✅ **Exact balance formulas**: `balance.rs` — `ClassProgress` for all 11 classes (vida, mana_inicial, mult_mana, hit_pre36, hit_post36), `get_max_hp_for_level`, `get_max_mana_for_level`, `get_hit_modifier_for_level` (pre/post level 36 split), `get_min/max_hit_for_level`, `get_legacy_exp_next_level` (exact 5-breakpoint EXP curve matching Node.js: ×1.4 <15, ×1.35 <21, ×1.3 <33, ×1.225 <41, ×1.25 after), `clamp_gold` (MAX_GOLD=2,147,483,647), `clamp_level` (1–50). 11 tests.
2. ✅ **Complete combat system**: `combat_formulas.rs` — `simulated_skill(level)` = min(100, level×3), `BodyPart` enum (Head/LeftLeg/RightLeg/RightArm/LeftArm/Torso) with `random_body_part`, class-based modifiers for all 11 classes (`mod_evasion`, `mod_escudo`, `mod_ataque_wrestling/armas/proyectiles`, `mod_dmg_armas/proyectiles/wrestling`), `poder_evasion`, `poder_evasion_escudo`, `poder_ataque_arma` with `WeaponType` enum, `calcular_dmg` (weapon + arrow + strength bonus), `melee_hit_chance` (clamped 5–95%), `roll_melee_hit`, `shield_block_chance`, `body_part_absorption` (head→helmet, body→body+shield), stabbing system (`can_stab`, `try_stab_npc` with npc min/max modifiers, `try_stab_pvp` with PvP damage modifiers). 11 tests.
3. ✅ **Dead World system**: `DEAD_WORLD_DELAY_MS=15000`, `dead_world_active` flag on PlayerState, 15-second visual transition after death before entering dead world state, visibility filtering (dead world players only see other dead players).
4. ✅ **Gold clamp**: `MAX_GOLD=2_147_483_647` (i32 max), `clamp_gold()` applied on all gold mutations — add, subtract, trade, market buy/sell, NPC commerce, loot pickup.

### Phase 14: Game Systems Pass ✅ COMPLETED (4/4)

Additional game systems ported from the Node.js original.

1. ✅ **Working Lock (anti-multi-bot)**: Per-entity IP tracking prevents simultaneous fishing/harvesting from the same IP address. `hasAnotherActiveWorkOnSameIp()` checks all active gathering sessions. `shouldCancelForSameIpConflict()` cancels newer sessions when conflicts detected. Integrated into fishing/harvesting handlers.
2. ✅ **Arena Instance Manager**: `ArenaManager` in `gameplay/arenas.rs` — dynamic map cloning via `create_instance()`, NPC spawning from base map spawn data, participant tracking with team/kills/deaths, account-level handover system (`begin_handover`/`end_handover`/`has_pending_handover`), instance lifecycle (WaitingForPlayers/InProgress/Finished), cleanup on empty (`destroy_instance`), unique map ID generation starting at 10,000. 5 tests.
3. ✅ **Shared Vaults**: Account-wide and clan-wide bank tabs with SQLite persistence. Extended bank system with `BankTab` enum (Personal/Account/Clan), deposit/withdraw/reorder operations per tab, clan vault access checks, `shared_vaults` table in SQLite.
4. ✅ **Connection Policy**: `getDuplicateAccountIdlePenalizedClientIds()` ported from `connectionPolicy.ts` — groups connected characters by account_id, detects multi-boxing (2+ sessions from same account with at least one actively gathering), penalizes idle sessions. Integrated into game loop for periodic checking.

### Phase 15: Polish Pass ✅ COMPLETED (7/7)

Smaller but faithful ports of remaining Node.js mechanics.

1. ✅ **Door system**: `DoorManager` in `gameplay/doors.rs` — open/close toggle with 250ms cooldown, range validation (max 2 tiles), key (`llave`) requirement, visual state tracking (`indexAbierta`/`indexCerrada`), `SND_PUERTA` sound effect, concurrent access via `RwLock<HashMap>`. 4 tests.
2. ✅ **Travel tickets**: Items with `travelTicketDestination` field (map/x/y) teleport the player on use. Ticket consumed after successful teleport. Destination validation (positive coordinates, valid map).
3. ✅ **Spell visual compositing**: Combined spell effects into efficient packet sending using flags.
4. ✅ **NPC respawn cooldowns**: Per-NPC individual respawn timers replacing the fixed 30s global respawn. Cooldowns tracked by (map, x, y, npcIndex) key, persisted for cross-restart survival.
5. ✅ **Faction rank rewards**: `claim_faction_rewards()` in `gameplay/factions.rs` — validates faction membership, checks level and score thresholds against 5-rank progression, returns rank title and progression messages. Wired to `/recompensa` command.
6. ✅ **Dragon Slayer sword logic**: `DRAGON_SLAYER_SWORD_ITEM_ID=402`, `is_dragon_slayer_hit()` — one-shot kill on dragons (npcType=6), sword consumed after hit with console message, `consume_dragon_slayer_sword()` removes from inventory and broadcasts weapon visual change. Clan Ring map (273) entry restriction when carrying the sword.
7. ✅ **Snapshot chunking**: Large initial data bursts split into manageable batches for WebSocket frame limits.

### Phase 16: Rust Optimizations ✅ COMPLETED (4/4)

Performance optimizations specific to the Rust stack.

1. ✅ **Packet builder capacity hints**: Pre-allocated `PacketWriter::with_packet_id_and_capacity()` on hot-path builders — `build_delete_character_packet` (3 bytes), `build_entity_vitals_delta` (11 bytes), `build_anim_fx` (5 bytes), `build_play_sound` (3 bytes), `build_delete_item` (5 bytes), `build_act_gold` (5 bytes), `build_act_color_name` (4 bytes), `build_change_equipment` (5 bytes).
2. ✅ **Batch SQLite writes**: `begin_transaction()` and `save_character_state_in_tx()` in `persistence/character.rs` — `/worldsave` now collects all player states, begins a single SQLite transaction, saves all characters atomically, and commits. Flushes inventory caches after commit.
3. ✅ **SmallVec for NPC loot**: `get_npc_loot()` returns `SmallVec<[(i32, i32, u16); 4]>` instead of `Vec` — common loot tables (≤4 items) stay on the stack, avoiding heap allocation on every NPC kill.
4. ✅ **DashMap shard tuning**: `scenes`, `inventory_cache`, and `inventory_dirty` `DashMap` instances in `GameWorld::new()` configured with `DashMap::with_shard_amount(32)` — reduces lock contention on high-frequency access patterns compared to default 16 shards.

### Phase 17: Combat & Gameplay Fidelity Pass ✅ COMPLETED (10/10)

Faithful port of remaining combat and gameplay systems from the original Node.js codebase.

1. ✅ **Magic Damage System**: `combat_formulas.rs` — `mod_dmg_magia` (class-based magic damage modifiers for 11 classes), `mod_resistencia_magica` (class-based magic resistance bonuses), `MagicBonusResult` struct, `apply_magic_bonuses` (caster level scaling + weapon/ring magic bonus + magic penetration), `apply_magic_resistance_to_npc` (NPC magic resistance/defense reduction with penetration offset), `apply_magic_resistance_to_user` (player item/class magic resistance with penetration offset). Integrated into `handle_attack_spell`. 6 tests.
2. ✅ **NPC Crowd Control**: Spells with `paraliza`/`inmoviliza` flags now apply CC to NPCs — `paralizado`, `inmovilizado`, `cc_expire_tick` fields on `NpcState`. Paralyzed NPCs skip all AI processing; immobilized NPCs can still attack but not move. CC expires after configurable tick duration. Integrated into `process_npc_ai` game loop.
3. ✅ **NPC Aggro System**: `aggro_target: Option<EntityId>` on `NpcState` — NPCs prioritize attacking the player who last hit them. Set in `attack_npc` on damage, checked first in `process_npc_ai` before searching for closest player. Aggro target validated (alive, in range) before use.
4. ✅ **Dead World Visibility Filtering**: Players in dead world state only see other dead world players on connect and teleport. Living players don't see dead world players. NPCs not visible to dead world players. Hidden (`invisible`, `hidden_skill`, `invisible_spell`) players filtered from visibility in `connect.rs` and `movement.rs`.
5. ✅ **Arena Combat Integration**: `is_arena_map(map_id)` function in `gameplay/arenas.rs` — PvP always enabled in arena maps (safe zone check bypassed). Integrated into `handle_attack_melee`, `handle_attack_range`, and `handle_attack_spell`.
6. ✅ **Faction PvP Rules**: Rival faction attacks (Armada vs Caos) on non-criminal targets don't flag the attacker as criminal. Same-faction attacks on non-criminals still flag criminal. Ported from original Node.js faction logic in `attack_player`.
7. ✅ **Working Lock Anti-Multi-Bot (DashMap)**: `working_lock: DashMap<String, EntityId>` on `GameWorld` — O(1) IP-based lock for simultaneous gathering prevention. `acquire_working_lock`/`release_working_lock` methods. Integrated into fishing and harvesting handlers (start/cancel/move/re-click). Replaces previous full-scan approach.
8. ✅ **Hidden Skill (Stealth) System**: `hidden_skill`, `hidden_skill_expire_tick`, `hidden_skill_cooldown_tick` fields on `PlayerState`. Chance formula ported from original: `clamp((((0.000002*skill - 0.0002)*skill + 0.0064)*skill + 0.1124)*100, 1, 99)`. Duration formula ported with step function. `stop_hidden_skill()` broadcasts character re-appearance. Movement removes stealth (unless Hunter class 8). Attacking removes stealth with 150-tick cooldown. NPCs can't detect hidden/invisible players. Hidden players filtered from connect/teleport visibility. Stealth expiry in `process_cc_expiry` game loop.
9. ✅ **Heal Spell PvP Targeting**: Heal spells now target nearest non-dead player in range (8 tiles), falling back to self-heal. Heal amount includes caster level scaling (`+3% per level`). Target receives vitals update + console message. Caster receives confirmation. FX broadcast to area.
10. ✅ **Balance Data Hot-Reloadable**: All balance formulas accessible via `GameData` hot-reload (`/recargar`). Class progression constants in `balance.rs` match original Node.js exactly. Combat formulas in `combat_formulas.rs` use game data for NPC stats. All 137 Rust src tests passing.

### Phase 18: Gameplay Refinement & Final Parity Pass ✅ COMPLETED (10/10)

Final parity pass porting remaining gameplay systems from the original Node.js codebase.

1. ✅ **Newbie System**: `NEWBIE_MAX_LEVEL=12` constant, `is_newbie_character()` helper. Newbie items (`obj.newbie != 0`) blocked from `USE_ITEM` when player level exceeds 12. Level-up notification when reaching level 13 warns about newbie item removal.
2. ✅ **Potion Recovery Percentage**: Verified HP and Mana potions use `porcentaje` field — HP potions apply `max_hp * porcentaje / 100` bonus on top of base random heal, Mana potions apply `max_mana * porcentaje / 100` bonus. Matches original `getPotionRecoveryAmount` logic.
3. ✅ **Map Level Restrictions**: `MapMeta` extended with `min_level`/`max_level` fields (deserialized from `minLevel`/`maxLevel` in JSON). `check_map_entry_denied()` validates player level against map requirements on all teleports/tile exits. Returns descriptive messages matching original format.
4. ✅ **Faction Portal Restrictions**: `FACTION_PORTAL_RESTRICTIONS` array with map 151 (caos-only) and map 60 (armada-only). Checked in `check_map_entry_denied()` — wrong-faction players see "Solo los miembros de X pueden usar este portal." Admins bypass all restrictions.
5. ✅ **Item Drop Position Validation**: `find_nearest_drop_position()` implements expanding-radius search (0–10 tiles) avoiding blocked tiles, tile exits, and existing ground items. `can_drop_at()` validates each candidate position. Falls back to "No hay espacio" if no valid position found.
6. ✅ **Tile Occupied Check**: `is_tile_occupied()` prevents two living entities from occupying the same tile. Checks both players (excluding self, excluding dead) and NPCs (excluding dead) at target position before allowing movement.
7. ✅ **Unsafe Logout Delay**: `/salir` command with `UNSAFE_LOGOUT_DELAY_MS=10_000`. In safe zones or when dead: instant disconnect. In PvP zones: 10-second quiet period (`logout_expires_at_ms` on PlayerState). Cancelled on movement (with message) or when attacked (with message). `check_pending_logout()` in session run loop triggers disconnect when timer expires. Blocks if paralyzed/immobilized.
8. ✅ **Boat Body Resolution**: `resolve_boat_body_id(current_body, dead)` — dead on boat returns body 87, existing special boats (85/86) preserved, default boat body is 84. Matches original `resolveBoatBodyId` exactly.
9. ✅ **Complete Visibility System**: `can_render_character()` function matching original `canRenderCharacter` — party members and clan members always visible regardless of dead world state; dead world viewers only see dead entities; invisible/hidden/stealth players filtered. Applied in both `connect.rs` and `movement.rs` (teleport) entity loading.
10. ✅ **AGENTS.md v20**: Updated with Phase 18 completion, LOC counts, new features documented, migration progress table updated.

### Phase 19: Further Parity & Refinements ✅ COMPLETED (8/8)

Additional parity items and gameplay refinements identified through deep audit of the original Node.js codebase.

1. ✅ **Complete Newbie Item Stripping**: `strip_newbie_items()` function in `combat.rs` — on reaching level 13 (NEWBIE_MAX_LEVEL+1), iterates player inventory, unequips all newbie items (`obj.newbie != 0`), removes from inventory cache, updates player visual IDs (weapon/body/helmet/shield → 0), refreshes full client inventory (20 slot packets), broadcasts visual equipment changes to AOI. Triggered from `check_level_up_and_notify` which now accepts `&GameWorld` for inventory access.
2. ✅ **Armada Faction Loss**: In `attack_player()`, when an Armada-aligned player attacks a neutral non-criminal citizen, their faction is cleared (`faction = "none"`, `faction_rank = 0`, `faction_rank_armada = 0`) with console notification. Player is still flagged criminal. Matches original `clearCharacterFaction` behavior.
3. ✅ **Citizen Clan PvP Block**: In `find_target()`, citizen-aligned players (armada or non-criminal neutral) who are in a clan cannot target other citizen-aligned players. Prevents friendly fire within citizen-aligned clan groups.
4. ✅ **Support Spell PvP Rules**: Heal spell targeting now filters out criminal players when caster is citizen-aligned and not in an arena map. Citizens cannot heal criminals outside arenas, matching original `isCitizenAlignedCharacter` behavior.
5. ✅ **Admin Commands Implemented**: `/quitarnpcpermanente` (permanent NPC removal from map), `/verip name` (shows target player's IP), `/intervalos`/`/paquetes` (real server packet metrics: uptime, connections, tick, packets in/out/rejected, per-category breakdown). Existing commands updated: `/recargarobjs`/`/recargarnpcs`/`/recargarbalance`/`/recargarcrafting` all route through unified hot-reload. `/resetaciertos` updated message (N/A in Rust engine).
6. ✅ **NPC EXP/Gold Multipliers**: `NPC_EXP_MULTIPLIER=5` and `NPC_GOLD_MULTIPLIER=3` constants in `combat_formulas.rs`, applied to all NPC kill rewards before double exp/gold bonuses and territory bonuses. Matches original `vars.multiplicadorExp` and `vars.multiplicadorGold` values.
7. ✅ **Armor Race Restriction**: `id_raza` field added to `PlayerState`, `CharacterData`, and characters SQLite table (ALTER migration). `raza_enana` bidirectional check in `handle_equip_item`: dwarf races (4=enano, 5=gnomo) can only equip `raza_enana=1` body armor; non-dwarf races cannot equip `raza_enana=1` items. Also enforces `clases_no_permitidas` class restriction on equip.
8. ✅ **AGENTS.md v21**: Updated with Phase 19 completion, LOC counts (~20,840 Rust), new features documented, migration progress table updated.

### Phase 20: PvP Rewards & Death Mechanics ✅ COMPLETED (7/7)

Faithful port of PvP kill rewards, faction score system, death cleanup, and bail system integration from the original Node.js codebase.

1. ✅ **PvP Kill Faction Score**: Ported `shouldAwardArmadaScore`/`shouldAwardCaosScore` from `respawn.ts`. Armada/citizen players earn Armada faction score when killing criminals/Caos. Criminal/Caos players earn Caos faction score on any kill. `calculateBaseFactionScore` returns 10 points per kill (matches original). Score awarded to both `faction_score_armada`/`faction_score_caos` (dual tracking) and combined `faction_score`. Console message "¡Has ganado X puntos de facción!" sent to attacker.
2. ✅ **PvP Rekill Protection**: `faction_rekill_tracker: DashMap<(EntityId, EntityId), u64>` on `GameWorld` — 5-minute window (`FACTION_REKILL_WINDOW_MS = 300,000ms`) prevents farming faction score by killing the same player repeatedly. Keyed by `(attacker_entity, victim_entity)` with timestamp tracking. Also prevents duplicate kill counter increments.
3. ✅ **Kill Counters**: `ciudadanos_matados` and `criminales_matados` fields on `PlayerState`, persisted to SQLite via ALTER TABLE migrations. Incremented on PvP kills (non-newbie, non-rekill). Loaded from DB on connect, saved on disconnect/worldsave/shutdown.
4. ✅ **Dual Faction Score Persistence**: `faction_score_armada` and `faction_score_caos` columns added to characters table (ALTER TABLE migration). `COALESCE` in SELECT for backwards compat. Saved in `save_character_state` and `save_character_state_in_tx` (4 new bind parameters). Loaded from DB and initialized in PlayerState on connect.
5. ✅ **PvP Exp/Gold Rewards**: Base PVP_BASE_EXP=50, PVP_BASE_GOLD=10, multiplied by NPC_EXP_MULTIPLIER(5) and NPC_GOLD_MULTIPLIER(3) respectively, matching original `vars.exp * vars.multiplicadorExp`. Double exp/gold flags respected. Gold clamped via `clamp_gold`. Console messages + `build_act_gold`/`build_act_exp` packets sent. Level-up check after PvP exp gain. Newbie victims (level ≤12) yield no rewards.
6. ✅ **Safe Logout Movement Cancel**: Already implemented — `/salir` sets `logout_expires_at_ms`, movement handler in `movement.rs` resets it to 0 with console message "La salida se canceló porque te moviste", and attack handler in `combat.rs` resets it with "La salida se canceló porque recibiste un ataque". Safe zones and dead players get instant logout.
7. ✅ **Death Cleanup**: On PvP death, victim's buffs cleared (`buffs.clear()`), paralysis/immobilization reset (`paralizado=false`, `inmovilizado=false`, timers zeroed), meditation cancelled (`meditar=false`), hidden skill removed (`hidden_skill=false`, expire/cooldown ticks zeroed). Comprehensive cleanup matches original `respawn.ts` behavior.

Additionally:
- ✅ **Bail System Enhanced**: `/fianza` now uses `ciudadanos_matados` counter with `BAIL_COST_PER_CITIZEN=5000` formula ported from original `commands.ts getBailCost()`. Cost = `ciudadanos_matados * NPC_GOLD_MULTIPLIER * 5000` (or `2500` base if no citizen kills). `build_open_bail` now sends actual citizen kill count. Matches original bail mechanics exactly.
- ✅ **New PlayerState Fields**: `logout_origin_x`, `logout_origin_y`, `criminales_matados`, `ciudadanos_matados`, `meditar` added with proper initialization and persistence.

### Phase 21: Combat & AI Fidelity Pass ✅ COMPLETED (10/10)

Combat timing, NPC AI, and safe zone parity items ported from the original Node.js codebase.

1. ✅ **Action Cooldown System**: `ActionCooldowns` struct on `PlayerState` with differentiated per-action cooldowns ported from `vars.timing.actionCooldowns` — melee (950ms), range (950ms), spell (850ms), use_item (250ms), dialog (500ms), cross-action gates (melee→spell 800ms, spell→melee 800ms, melee→use_item 550ms). `can_X(now)`/`trigger_X(now)` methods using `world.uptime_ms()`. Silently drops actions during cooldown (no error packet, matches original).
2. ✅ **Cooldowns Integrated in Combat**: `handle_attack_melee`, `handle_attack_range`, `handle_attack_spell` check and trigger appropriate cooldowns before processing attacks. Cross-action gates prevent rapid melee→spell switching.
3. ✅ **Cooldowns Integrated in USE_ITEM + Dialog**: `handle_use_item` in `inventory.rs` and `handle_dialog` in `dialog.rs` check `can_use_item`/`can_dialog` before processing. Prevents potion spam and chat flood at source.
4. ✅ **Party EXP Bonus Corrected**: Changed from 10% (`1.1`) to 15% (`1.15`) matching original `partyExpBonusPct = 15`.
5. ✅ **NPC Spell Casting System**: NPCs with spells can cast offensive and healing magic. `NpcSpellSlot` on `NpcState` loaded from `NpcTemplate.spells`. AI self-heals when HP < 50%, otherwise casts random offensive spell. `spell_cast_interval_ms` cooldown between casts. Spell damage applies `apply_magic_resistance_to_user`. FX broadcast + projectile + vitals + death handling. Casts both in melee range and at spell range.
6. ✅ **NPC Target Scoring**: Replaced simple closest-player selection with weighted scoring system ported from `NPC_TARGET_SCORE_WEIGHTS`: distance×3, adjacent bonus (-6), aggro bonus (-14). Aggro target still has absolute priority. Refactored chase movement into `try_npc_move_towards` and melee damage into `apply_npc_melee_damage` helper functions.
7. ✅ **Per-Tile Safe Zone (trigger=6)**: New `is_safe_position(map_id, x, y)` method checks both map-level `pk=1` AND tile-level `safe_tiles` set (loaded from `specials.json safeTiles`). All combat handlers (melee/ranged/spell) and `/fianza` now use position-aware safe zone checking instead of map-level only. Matches original `isInSafeZone` behavior.
8. ✅ **Arena Trigger Zones**: Arena maps bypass safe zone via existing `is_arena_map()` integration in all combat handlers. Position-aware safe zones don't interfere with arena PvP.
9. ✅ **NPC Summon Infrastructure**: `NpcState` extended with `spells`, `spell_cast_interval_ms`, `last_spell_cast_at`, `spell_range`, `magic_def`, `magic_resistance` fields. `NpcTemplate` extended with `NpcSpellEntry` (id_spell + cooldown_seconds). Full NPC summon system (player-owned combat NPCs) deferred to Phase 22.
10. ✅ **Chat Audit Logger**: Structured tracing at `chat_audit` target for all chat messages and commands — logs player name, entity ID, map ID, and message/command text. Enables grep-able audit trail for moderation.

### Phase 22: NPC Summon System ✅ COMPLETED (4/4)

Player-summoned NPC system ported from the original Node.js codebase.

1. ✅ **Summon Spell Integration**: Spells with `num_npc > 0` in `SpellTemplate` trigger `handle_summon_spell()` — spawns the specified NPC type near the caster. Mana consumed, FX broadcast. `num_npc` field added to `SpellTemplate` (deserialized from `numNpc` in JSON).
2. ✅ **Summon Lifecycle**: `summoned_by: Option<EntityId>` and `summon_expires_at_ms: u64` fields on `NpcState`. `summons: Vec<EntityId>` on `PlayerState`. `MAX_SUMMONS_PER_USER=3`, `SUMMON_DURATION_MS=120_000` (2 min). Oldest summon auto-despawned when limit exceeded.
3. ✅ **Summon Expiry**: `process_summon_expiry()` in `simulation/mod.rs` runs every 60 ticks — removes expired summons, broadcasts `DELETE_CHARACTER` to AOI, cleans up AOI grid.
4. ✅ **Summon Cleanup on Disconnect**: `cleanup_summons_on_disconnect()` in `gateway/mod.rs` — removes all NPCs summoned by a disconnecting player, broadcasts deletion to area viewers. Summoned NPCs yield 0 EXP.

### Phase 23: Missing Cooldowns ✅ COMPLETED (3/3)

Additional per-action cooldowns identified from the original `vars.timing.actionCooldowns`.

1. ✅ **Drop Item Cooldown (150ms)**: `can_drop_item`/`trigger_drop_item` on `ActionCooldowns`, integrated in `handle_drop_item` in `inventory.rs`. Prevents rapid item dropping.
2. ✅ **Equip Toggle Cooldown (125ms)**: `can_equip_toggle`/`trigger_equip_toggle` on `ActionCooldowns`, integrated in `handle_equip_item` in `inventory.rs`. Prevents rapid equip/unequip toggling.
3. ✅ **Click Cooldown (150ms)**: `can_click`/`trigger_click` on `ActionCooldowns`, integrated in `handle_click` in `commerce.rs`. Prevents click spam on NPCs/players.

### Phase 24: Activity Logging ✅ COMPLETED (3/3)

Structured activity logging for moderation, auditing, and analytics.

1. ✅ **Combat Activity Logging**: `tracing::info!(target: "activity")` for `npc_kill` (NPC type, exp, gold awarded) and `pvp_kill` (attacker/victim, faction score) events in `combat.rs`.
2. ✅ **Economy Activity Logging**: `tracing::info!(target: "activity")` for `buy_item`, `sell_item` (NPC commerce), `bank_deposit`, `bank_withdraw` events with gold deltas.
3. ✅ **Progression Activity Logging**: `tracing::info!(target: "activity")` for `level_up` (new level, class), `character_connect`, `character_disconnect` events with session metadata.

### Phase 25: Spell Effects Parity ✅ COMPLETED (4/4)

Missing spell effect handlers ported from original Node.js spell system.

1. ✅ **Remove Paralysis Spell**: Spells with `remover_paralisis=1` clear `paralizado` and `inmovilizado` flags on caster. Mana consumed, console feedback.
2. ✅ **Invisibility Spell**: Spells with `invisibilidad=1` set `invisible=true` on caster, broadcast `DELETE_CHARACTER` to AOI (caster disappears from other players). Mana consumed.
3. ✅ **Buff Spells (Agility/Strength)**: Spells with `sube_ag=1` apply agility buff via `BuffManager::apply(BuffType::Agility, ...)` using `min_ag`/`max_ag` range. Spells with `sube_fz=1` apply strength buff similarly. FX broadcast, mana consumed, console feedback.
4. ✅ **Spell MinSkill Level Gate**: `simulated_skill(player.level) < st.min_skill` check before any spell cast — prevents casting spells above player's skill level with descriptive "Necesitas ser nivel X" message. Matches original `getSimulatedSkill` / `getRequiredLevelForSpell` logic.

### Phase 26: Fidelity & Optimization Pass ✅ COMPLETED (9/9)

Additional fidelity items and performance optimizations identified through deep audit.

1. ✅ **Dragon Respawn Cooldown**: NPCs with `npc_type == 6` (dragons) have a 1-hour respawn cooldown (`3_600_000ms`) instead of the standard 30s, matching original `DRAGON_RESPAWN_COOLDOWN_MS`. Implemented in `process_npc_respawn`.
2. ✅ **Arena Combat Kill/Death Tracking**: `ArenaManager` extended with `record_kill(arena_id, killer, victim)`, `get_team_scores(arena_id)` returning per-team kill/death totals, and `get_winning_team(arena_id)` for match resolution.
3. ✅ **Advanced NPC AI Scoring**: Target scoring in `process_npc_ai` now includes `escape_tiles` (walkable adjacent tiles for player), `attack_tiles` (adjacent tiles NPC can attack from), `is_current` (current target bonus -8), and adjusted `is_aggro` (-14). Weights: `distance*3 + escape_tiles*2 - attack_tiles*4 - is_adjacent*6 - is_current*8 - is_aggro*14`. Matches original `NPC_TARGET_SCORE_WEIGHTS` from `vars.ts`.
4. ✅ **Granular Equipment Slots (Arrow/Ring)**: `id_arrow_slot` and `id_ring_slot` fields added to `PlayerState`, `CharacterData`, and characters SQLite table (ALTER migration). Loaded from DB on connect, saved on disconnect/worldsave/shutdown. Full persistence roundtrip including `COALESCE` for backwards compatibility.
5. ✅ **Combat Formula Tests Expanded**: 7 new tests in `combat_formulas.rs` — `stab_pvp_returns_stabresult`, `stabbing_class_modifiers_differ`, `npc_evasion_scales_with_level`, `newbie_check`, `resolve_boat_body_dead`, `resolve_boat_body_special_preserved`, `npc_multipliers_match_original`. Total src tests: 144.
6. ✅ **NPC AI Batch Read/Write Separation**: NPC AI processing refactored to collect all position/state reads before applying writes, reducing DashMap lock contention during multi-NPC tick processing.
7. ✅ **Tick Time Metrics**: `tick_time_max_us`, `tick_time_sum_us`, `tick_time_count` atomic counters on `ServerMetrics`. Each game tick measures processing time in microseconds. Exposed as `tick_time_max_us` and `tick_time_avg_us` in `/api/metrics` JSON and as `openao_tick_time_max_us`/`openao_tick_time_avg_us` gauges in `/api/metrics/prometheus`.

### Phase 27: Admin & Testing Pass ✅ COMPLETED (5/5)

Admin tools and expanded test coverage.

1. ✅ **Admin Bot System**: `/bot NPC_ID [LEVEL]` command spawns a scaled NPC at the admin's position. NPC stats scale with level (HP = base × level/10, dmg proportional). `admin_bot_owner: Option<EntityId>` field on `NpcState` tracks ownership. Bots give 0 EXP. `process_admin_bot_heal()` in game loop auto-heals bots at 5% maxHP every 30 ticks. `/bot limpiar` (aliases: `desinvocarbots`, `quitarbots`, `borrarbots`) removes all bots owned by the calling admin with AOI-filtered DELETE_CHARACTER broadcast.
2. ✅ **Runtime Timing Hot-Modification**: `/intervalo key [value]` admin command for dynamic game loop tuning. `RuntimeTimings` struct on `GameWorld` with `AtomicU64` fields: `melee_ms` (950), `range_ms` (950), `spell_ms` (850), `use_item_ms` (250), `dialog_ms` (500), `regen_ticks` (60), `npc_ai_ticks` (30). `process_tick` reads `regen_ticks` and `npc_ai_ticks` atomically each frame. Query without value shows current setting, set with value updates atomically.
3. ✅ **ActionCooldowns Tests**: 5 new tests in `world/mod.rs` — melee blocks until ready, cross-gate melee→spell (800ms), cross-gate spell→melee (800ms), use_item after melee gate (550ms), cooldown constants match original `vars.timing.actionCooldowns`. Plus `runtime_timings_defaults` test validating all default values.
4. ✅ **Combat Formula Edge Case Tests**: 4 new tests in `combat_formulas.rs` — `dragon_slayer_hit_only_on_dragons`, `dead_world_delay_is_15s`, `unsafe_logout_is_10s`, `magic_bonuses_zero_level_returns_base`, `magic_resistance_npc_never_negative`, `magic_resistance_user_never_negative`.
5. ✅ **Balance Formula Tests**: 3 new tests in `balance.rs` — `all_classes_have_positive_hp_at_level_50` (all 11 classes), `exp_curve_breakpoints_monotonically_increasing` (levels 1-50), `exp_curve_level_1_is_300` (base value), `clamp_level_bounds` (min/max validation).

### Phase 28: Parity Audit & Polish Pass ✅ COMPLETED (4/4)

Comprehensive parity verification of the original Node.js codebase against the Rust port, frontend polish, and expanded parity tests.

1. ✅ **Full Codebase Parity Audit**: Systematic comparison of all critical original files — `game.ts` (9944 LOC, 88 functions), `commands.ts` (4329 LOC), `protocol.ts` (4431 LOC), `npcs.ts` (3125 LOC), `handleProtocol.ts` (1570 LOC), `login.ts` (1218 LOC), `respawn.ts` (316 LOC), `fishing.ts` (458 LOC), `harvesting.ts` (537 LOC), `crafting.ts` (454 LOC) — against the Rust port. Confirmed: all 88 game.ts functions ported, all 81 server→client opcodes synchronized, all 30+ client→server packet routes registered, combat formulas (`calcularDmg`, `poderEvasion`, `poderAtaqueArma`, `melee_hit_chance`, `shield_block_chance`, `body_part_absorption`, stabbing system) match 1:1, death mechanics/respawn/faction score/PvP rewards fully ported, fishing/harvesting/crafting tick systems faithful to original.
2. ✅ **Frontend Accessibility Fixes**: Resolved all 8 svelte-check warnings (was: 8 warnings, now: 0 errors, 0 warnings). Added `for`/`id` label associations in `MacroBar.svelte` (key binding, type select, item/spell/command fields) and `TradeModal.svelte` (amount input). Added `tabindex="-1"` and `a11y-no-static-element-interactions` pragma on MacroBar dialog overlay.
3. ✅ **Parity Verification Tests**: 6 new tests in `combat_formulas.rs` — `pvp_base_rewards_match_original` (50×5=250 exp, 10×3=30 gold), `faction_rekill_window_is_5_minutes` (300,000ms), `bail_cost_formula_matches_original` (3 kills × 3 × 5000 = 45,000), `all_11_classes_have_class_modifiers` (evasion/shield/weapon for all 11 classes), `calcular_dmg_unarmed_uses_wrestling_range` (positive damage with no weapon), `hidden_skill_chance_formula_bounds` (skill 0 low, skill 100 high).
4. ✅ **Test Suite Expansion**: Total tests increased from 265 to 271 (166 Rust src + 15 Rust crates + 51 protocol TS + 39 frontend netcode TS). All passing. Frontend svelte-check: 0 errors, 0 warnings.

### Phase 29: Class ID Parity Fix ✅ COMPLETED (4/4)

Critical fix for class ID mapping discrepancy between original Node.js game (class IDs 1,2,3,4,6,7,8,9 — skips 5) and Rust port (sequential 1,2,3,4,5,6,7,8). All combat modifiers and balance data for classes 5-8 now use the correct values from their original counterparts.

1. ✅ **Balance Data Fix (balance.rs)**: Corrected `ClassProgress` array entries for Bardo (ID 5 → original ID 6: vida=8.5, manaInicial=2.5, multMana=2.0), Druida (ID 6 → original ID 7: vida=8.5, manaInicial=2.5, multMana=2.0), Paladin (ID 7 → original ID 8: vida=10.0, manaInicial=2.5, multMana=1.0), Cazador (ID 8 → original ID 9: vida=10.0, manaInicial=0, multMana=0). Previously Bardo had 0 mana and Cazador had mana — now matches original exactly.
2. ✅ **Combat Modifier Fix (combat_formulas.rs)**: Corrected all 14 class-indexed combat modifier functions to map the correct original values for classes 5-8: `mod_evasion`, `mod_escudo`, `mod_ataque_wrestling`, `mod_ataque_armas`, `mod_ataque_proyectiles`, `mod_dmg_armas`, `mod_dmg_proyectiles`, `mod_dmg_wrestling`, `mod_dmg_magia`, `mod_resistencia_magica`, `stabbing_chance_by_class`, `stabbing_dmg_mod_pvp`, `stabbing_npc_min_mod`, `stabbing_npc_max_mod`. Each function now has inline comments documenting the original class ID mapping.
3. ✅ **API Class Mapping Verified**: Confirmed `api/mod.rs` register endpoint correctly maps class names to sequential IDs (mago=1, clerigo=2, guerrero=3, asesino=4, bardo=5, druida=6, paladin=7, cazador=8). Initial creation stats in `persistence/characters.rs` verified to produce correct HP/Mana for all 8 classes based on the corrected balance data.
4. ✅ **All Tests Passing**: 271 tests (166 Rust src + 15 Rust crates + 51 protocol TS + 39 frontend netcode TS) all passing after the fix, confirming no regressions.

---

## 9. Frontend Key Files

| File | Purpose |
|------|---------|
| `src/lib/components/game/PixiApp.svelte` | Main Pixi.js rendering: map tiles (4 layers), entity sprites, roof/tree visibility |
| `src/lib/components/game/GameView.svelte` | Game page layout, connection UI, keyboard input, client-side movement prediction (PredictionBuffer + InputSender wired in tryMove) |
| `src/lib/game/state/gameState.svelte.ts` | Reactive game state (Svelte 5 runes): HudState, RemoteEntity, RemoteNpc, GroundItem |
| `src/lib/game/state/assetStore.svelte.ts` | Asset store: graphicsDB, objectsDB, npcsDB, bodiesDB, headsDB, spellsDB |
| `src/lib/game/state/mapState.svelte.ts` | Map data store: load/decode/cache map data, tile blocked checks |
| `src/lib/game/session/gameSession.svelte.ts` | WebSocket connection + ELR2 framing + reconnect logic |
| `src/lib/game/session/outgoingRequests.ts` | Functions to send packets to server |
| `src/lib/game/session/registerPacketHandlers.ts` | Incoming packet dispatch table |
| `src/lib/game/session/useOutgoingRequests.ts` | Outgoing request hooks |
| `src/lib/game/session/useGameSession.ts` | Game session hooks |
| `src/lib/game/session/incomingWorldPackets.ts` | World packet handlers |
| `src/lib/game/session/incomingCharacterPackets.ts` | Character packet handlers |
| `src/lib/game/session/incomingPacketTypes.ts` | Packet type definitions |
| `src/lib/game/utils/gameLoader.ts` | Asset loading pipeline (JSON fetching, map decompression) |
| `src/lib/game/engine/assetLoader.ts` | Lightweight map decoder + graphic info resolution for PixiApp |
| `src/lib/game/engine/Engine.ts` | Core game engine |
| `src/lib/game/rendering/sceneRenderer.ts` | Original map tile rendering (legacy, not currently used by PixiApp) |
| `src/lib/game/rendering/effectsRenderer.ts` | Visual FX rendering |
| `src/lib/game/rendering/characterRenderer.ts` | Character sprite rendering |
| `src/lib/game/rendering/entityOverlays.ts` | Entity name/health bar overlays |
| `src/lib/game/rendering/visibility.ts` | Visibility calculations |
| `src/lib/game/rendering/weatherSystem.ts` | Weather particle system (rain/snow/fog/storm), singleton `weatherSystem` |
| `src/lib/game/rendering/dayNightCycle.ts` | Day/Night cycle (20min: dawn/day/dusk/night), tint overlay, singleton `dayNightCycle` |
| `src/lib/components/game/WeatherOverlay.svelte` | Canvas overlay integrating WeatherSystem + DayNightCycle |
| `src/lib/game/rendering/rowLayerContainers.ts` | Row-based layer containers for depth sorting |
| `src/lib/game/rendering/debugGrid.ts` | Debug grid overlay |
| `src/lib/game/lib/graphicTextures.ts` | Graphic texture management |
| `src/lib/game/network/ping.ts` | Network ping calculator |
| `src/lib/game/network/tickSync.ts` | Client-side tick synchronizer (estimates server tick from heartbeat probes, separates network RTT from server processing) |
| `src/lib/game/network/interpolation.ts` | InterpolationBuffer (port of Elura) — wired into moveEntity() + PixiApp render loop for smooth remote entity/NPC interpolation |
| `src/lib/game/network/prediction.ts` | PredictionBuffer (port of Elura) — wired into tryMove() for instant local prediction + applyServerPosition() for server reconciliation |
| `src/lib/game/network/inputSender.ts` | InputSender (port of Elura) — wired into tryMove() for input tracking + applyServerPosition() for cumulative ACK |
| `src/lib/game/network/__tests__/` | Unit tests for InterpolationBuffer (9), PredictionBuffer (8), InputSender (9) — 26 tests total |
| `src/lib/game/network/packetQueue.ts` | Packet queue management |
| `src/lib/game/input/combatTargeting.ts` | Combat targeting input |
| `src/lib/game/config/animationTiming.ts` | Animation timing configuration |
| `src/lib/game/core/useSceneController.ts` | Scene management hooks |
| `src/lib/game/core/useRendererBootstrap.ts` | Renderer initialization |
| `src/lib/game/core/useRemoteEntityController.ts` | Remote entity management |
| `src/lib/game/core/useHudStateController.ts` | HUD state management |
| `src/lib/game/core/useCombatController.ts` | Combat system hooks |
| `src/lib/game/core/useAssetPipeline.ts` | Asset pipeline management |
| `src/lib/game/core/useKeyboardGameplay.ts` | Keyboard input for gameplay |
| `src/lib/game/core/useNpcAdminTools.ts` | NPC admin tool hooks |
| `src/lib/game/assets/scenePreload.ts` | Scene asset preloading |
| `src/lib/components/game/StatsPanel.svelte` | Player stats display panel |
| `src/lib/components/game/ChatPanel.svelte` | Chat display and input |
| `src/lib/components/game/InventoryPanel.svelte` | Inventory UI |
| `src/lib/components/game/SpellBar.svelte` | Spell bar UI |
| `src/lib/components/game/MacroBar.svelte` | Macro shortcuts bar |
| `src/lib/components/game/BuffStatusSidebar.svelte` | Buff/status sidebar |
| `src/lib/components/game/CharacterStatsModal.svelte` | Character stats modal |
| `src/lib/components/game/CraftingModal.svelte` | Crafting interface |
| `src/lib/components/game/MarketModal.svelte` | Player market UI |
| `src/lib/components/game/RetosModal.svelte` | Challenges UI |
| `src/lib/components/game/BailModal.svelte` | Bail payment modal |
| `src/lib/components/game/TradeModal.svelte` | NPC trading modal |
| `src/lib/components/game/ItemGraphic.svelte` | Item icon renderer |
| `src/lib/components/game/NpcInspectorModal.svelte` | NPC stat inspector (admin tool) |
| `src/lib/components/game/AdminIntervalsModal.svelte` | Double exp/gold toggle via REST |
| `src/lib/components/game/OverviewModal.svelte` | Character overview panel |
| `src/lib/components/game/DebugOverlay.svelte` | Debug info overlay (F3 toggle) |
| `src/routes/wiki/[section]/+page.server.ts` | Wiki SSR data loader (fetches from backend /api/wiki) |
| `src/routes/play/+page.svelte` | Game page |
| `src/routes/wiki/[section]/+page.svelte` | Wiki pages |
| `src/routes/ranking/+page.svelte` | Ranking page |
| `src/routes/login/+page.svelte` | Login page |
| `src/routes/+page.svelte` | Landing page |
| `src/routes/+layout.svelte` | Root layout |
| `src/routes/register/+page.svelte` | Registration page |
| `src/routes/forgot-password/+page.svelte` | Password reset request page |
| `src/routes/reset-password/[token]/+page.svelte` | Password reset page |
| `src/routes/characters/+page.svelte` | Character list page |
| `src/routes/createcharacter/+page.svelte` | Character creation page |
| `src/routes/character/+page.svelte` | Character detail page |
| `src/routes/updates/+page.svelte` | Game updates/changelog page |
| `src/routes/construccion/+page.svelte` | Map editor page (lazy-loads EditorView) |
| `src/routes/arenas/+page.svelte` | Arenas list page |
| `src/routes/arenas/join/[joinToken]/+page.svelte` | Arena join page |
| `src/routes/users-online-stats/+page.svelte` | Online users stats page |
| `src/lib/components/AppChrome.svelte` | Main navigation bar/chrome |
| `src/lib/components/editor/EditorView.svelte` | Map editor UI (tile paint, NPC placement, teleport triggers) |
| `src/lib/editor/editorStore.svelte.ts` | Map editor reactive state (tools, layers, undo/redo) |
| `src/lib/editor/editorApi.ts` | Map editor API client |
| `src/lib/editor/useGameDataAdmin.ts` | Admin game data management hooks |
| `src/lib/server/db.ts` | Server-side SQLite database setup |
| `src/lib/server/migrate.ts` | Server-side database migrations |
| `src/lib/server/api-proxy.ts` | Backend API proxy for SSR |
| `src/lib/server/repositories/auth.ts` | Auth repository (server-side) |
| `src/lib/server/repositories/characters.ts` | Characters repository (server-side) |
| `src/lib/server/repositories/ranking.ts` | Ranking repository (server-side) |
| `src/lib/auth.ts` | Client-side auth utilities |
| `src/lib/game/lib/auth-session.ts` | Auth session management |
| `src/lib/game/lib/characterCreation.ts` | Character creation logic |
| `src/lib/game/lib/character-settings.ts` | Character settings/preferences |
| `src/lib/game/lib/arenas.ts` | Arena data and utilities |
| `src/lib/game/lib/clientDiagnostics.ts` | Client diagnostics/debugging |
| `src/lib/game/lib/hardware-acceleration.ts` | GPU acceleration detection |
| `src/lib/game/lib/hotkeys.ts` | Keyboard hotkey bindings |
| `src/lib/game/lib/runtime-config.ts` | Runtime configuration (WS/API URLs from env) |
| `src/lib/game/lib/sound.ts` | Sound manager (Howler.js) |
| `src/lib/game/lib/viewport.ts` | Viewport calculations |
| `src/lib/game/lib/users-online-stats.ts` | Online stats data fetching |
| `src/lib/game/lib/wiki.ts` | Wiki data utilities |
| `src/lib/game/lib/wiki-data.ts` | Wiki data types |
| `src/lib/game/lib/wiki-sections.ts` | Wiki section definitions |
| `src/lib/game/lib/ranking.ts` | Ranking data utilities |
| `src/lib/game/lib/ranking-heads.ts` | Ranking character heads data |
| `src/lib/game/lib/authPayloadEncryption.ts` | Auth payload encryption |
| `src/lib/data/objectTypes.ts` | Object type definitions |
| `src/lib/game/types/game.ts` | Game type definitions |

### Frontend Rendering Architecture

The rendering pipeline uses Pixi.js 8 with layered containers:
1. **groundLayer** — Layer 1 (floor tiles, 32x32 each)
2. **belowLayer** — Layer 2 (objects, decorations, bottom-anchored)
3. **aboveLayer** — Layer 3 (trees, tall objects, bottom-anchored, sortable by Y for depth)
4. **entityLayer** — Players, NPCs, ground items (sortable by Y for depth)
5. **roofLayer** — Layer 4 (roof tiles, hidden when player is inside)

Entity sprites are composed from:
- **bodies.json** — Maps bodyId → directional grhIds (keys "1"-"4") + headOffset
- **heads.json** — Maps headId → directional grhIds (keys "1"-"4")
- **npcs_optimized.json** — Maps npcType → { idBody, idHead, name, ... }

The backend sends `CHANGE_BODY`/`CHANGE_ROPA`/`CHANGE_WEAPON`/`CHANGE_HELMET`/`CHANGE_SHIELD` packets to communicate equipment visual IDs. These are now sent to the player themselves on connect (not just broadcast to others).

### Frontend State Flow

- `HudState` includes `idBody`, `idHead`, `idWeapon`, `idHelmet`, `idShield` for the local player's visual equipment
- `RemoteEntity` includes `bodyGrh`, `headGrh`, `weaponGrh`, `shieldGrh`, `helmetGrh` for other players
- `setEntityEquipment()` routes to HudState when `id === hud.id`, otherwise updates RemoteEntity
- `changeRopa` packet maps to `headGrh` (head/ropa visual), `changeBody` maps to `bodyGrh`

### Frontend Architecture Patterns

- **Svelte 5 runes** (`$state`, `$derived`, `$effect`) for reactive state management
- **Composition hooks** pattern (`use*.ts` files in `game/core/`) for modular game logic
- **Session separation**: `incomingWorldPackets.ts` + `incomingCharacterPackets.ts` split packet handling
- **Asset pipeline**: JSON-based game data loaded via `gameLoader.ts` + `scenePreload.ts`
- **ELR2 framing**: `gameSession.svelte.ts` handles subprotocol negotiation, auth frame, and transparent game packet wrapping
- **Reconnect overlay**: "Reconectando..." UI during reconnect attempts with exponential backoff
- **Elura netcode wired**: `InterpolationBuffer` wired to entity rendering (smooth interpolation), `PredictionBuffer` wired to player movement (instant prediction + server reconciliation), `InputSender` wired to input tracking (sequence/ACK). All in `game/network/` with 39 unit tests.
- **Server-side auth**: `hooks.server.ts` reads base64-encoded JSON session cookie into `event.locals.session`, enabling SSR-aware auth state
- **API proxy**: `api-proxy.ts` + SvelteKit API routes (`/api/auth/[...path]`, `/api/ranking`, `/api/market/[...path]`, `/api/arenas`) proxy requests to the Rust HTTP API at port 7667
- **Map editor**: Full-featured tile editor at `/construccion` with tools (select, paint, erase, fill, block, NPC, spawn, teleport), layer selection, undo/redo history, backed by `editorStore.svelte.ts`

---

## 10. Database Schema (SQLite)

### `accounts`
| Column | Type | Notes |
|--------|------|-------|
| id | TEXT PK | UUID |
| username | TEXT UNIQUE | |
| email | TEXT UNIQUE | |
| password_hash | TEXT | Argon2 hash (new accounts), plaintext (legacy) |
| is_admin | INTEGER | 0/1 |

### `characters`
| Column | Type | Notes |
|--------|------|-------|
| id | TEXT PK | UUID |
| account_id | TEXT FK | → accounts.id |
| name | TEXT UNIQUE | |
| id_clase | INTEGER | Class ID (1-8) |
| map_id, pos_x, pos_y | INTEGER | Position |
| gold, hp, max_hp, mana, max_mana | INTEGER | Resources |
| level, exp, exp_next_level | INTEGER | Progression |
| dead, criminal, navegando | INTEGER/BOOL | Flags |
| attr_fuerza/agilidad/inteligencia/constitucion | INTEGER | Attributes |
| min_hit, max_hit | INTEGER | Damage range |
| id_head, id_body, id_helmet, id_weapon, id_shield | INTEGER | Equipment visual IDs |
| home_map, home_x, home_y | INTEGER | Respawn point |
| faction_rank | INTEGER | Faction rank (0 = none) — ALTER migration |
| faction_score | INTEGER | Faction score — ALTER migration |
| faction_score_armada | INTEGER | Armada faction score |
| faction_score_caos | INTEGER | Caos faction score |
| bank_gold | INTEGER | Bank gold (persisted) |
| paralizado | INTEGER | CC paralysis flag |
| inmovilizado | INTEGER | CC immobilization flag |
| seguro_activado | INTEGER | PvP safety toggle |
| seguro_clan_activado | INTEGER | Clan safety toggle |
| id_raza | INTEGER | Race ID (1=humano, 2=elfo, 3=elfoDrow, 4=enano, 5=gnomo) — ALTER migration |
| faction_score_armada | INTEGER | Armada faction score — ALTER migration |
| faction_score_caos | INTEGER | Caos faction score — ALTER migration |
| criminales_matados | INTEGER | PvP criminal kills counter — ALTER migration |
| ciudadanos_matados | INTEGER | PvP citizen kills counter — ALTER migration |

### `game_tickets`
| Column | Type | Notes |
|--------|------|-------|
| ticket | TEXT UNIQUE | Login credential |
| account_id, character_id | TEXT FK | |
| consumed_at | TEXT | NULL until used (reset in dev mode) |
| expires_at | TEXT | RFC3339 |

### `character_inventory`
| Column | Type | Notes |
|--------|------|-------|
| character_id | TEXT FK | |
| slot | INTEGER | 0-19 |
| item_id | INTEGER | Maps to `get_item_data()` |
| amount | INTEGER | Stack count |
| equipped | INTEGER | 0/1 |
| UNIQUE(character_id, slot) | | |

### `character_bank`
| Column | Type | Notes |
|--------|------|-------|
| character_id | TEXT FK | |
| slot | INTEGER | Bank slot index |
| item_id | INTEGER | Maps to `get_item_data()` |
| amount | INTEGER | Stack count |
| UNIQUE(character_id, slot) | | |

### `bans`
| Column | Type | Notes |
|--------|------|-------|
| id | INTEGER PK | Auto-increment |
| account_id | TEXT UNIQUE | Account/character ID |
| reason | TEXT | Ban reason |
| banned_by | TEXT | Admin who issued ban |
| created_at | TEXT | Timestamp |

### `mutes`
| Column | Type | Notes |
|--------|------|-------|
| id | INTEGER PK | Auto-increment |
| account_id | TEXT UNIQUE | Character ID |
| reason | TEXT | Mute reason |
| muted_by | TEXT | Admin who issued mute |
| created_at | TEXT | Timestamp |

### `market_listings`
| Column | Type | Notes |
|--------|------|-------|
| id | TEXT PK | UUID |
| seller_character_id | TEXT FK | |
| item_id, amount, price | INTEGER | Listing details |
| created_at, expires_at | TEXT | RFC3339 timestamps |
| buyer_character_id | TEXT | NULL until purchased |
| claimed | INTEGER | 0/1 |

### `ip_bans`
| Column | Type | Notes |
|--------|------|-------|
| id | INTEGER PK | Auto-increment |
| ip_address | TEXT UNIQUE | Banned IP |
| reason | TEXT | Ban reason |
| banned_by | TEXT | Admin who issued ban |
| created_at | TEXT | Timestamp |

### `character_quests_active`
| Column | Type | Notes |
|--------|------|-------|
| character_id | TEXT FK | → characters.id |
| quest_id | INTEGER | Quest definition ID |
| objectives_json | TEXT | JSON array of `ObjectiveProgress` |
| UNIQUE(character_id, quest_id) | | |

### `character_quests_completed`
| Column | Type | Notes |
|--------|------|-------|
| character_id | TEXT FK | → characters.id |
| quest_id | INTEGER | Quest definition ID |
| completed_at | TEXT | Timestamp |
| UNIQUE(character_id, quest_id) | | |

### `character_pets`
| Column | Type | Notes |
|--------|------|-------|
| character_id | TEXT FK | → characters.id |
| pet_type | INTEGER | Pet type ID |
| name | TEXT | Pet name |
| level | INTEGER | Pet level |
| exp | INTEGER | Pet experience |
| hp | INTEGER | Pet current HP |
| active | INTEGER | 0/1 (summoned) |
| UNIQUE(character_id, pet_type) | | |

### `character_achievements`
| Column | Type | Notes |
|--------|------|-------|
| character_id | TEXT FK | → characters.id |
| achievement_id | INTEGER | Achievement definition ID |
| unlocked_at | TEXT | Timestamp |
| stats_json | TEXT | JSON `PlayerStats` snapshot |
| UNIQUE(character_id, achievement_id) | | |

### `character_settings`
| Column | Type | Notes |
|--------|------|-------|
| character_id | TEXT PK | → characters.id |
| settings_json | TEXT | JSON client preferences |

---

## 11. Running the Project

### Backend
```bash
cd game-server-rs
cargo run
# WS: 0.0.0.0:7666, HTTP: 0.0.0.0:7667
# SQLite: openao.db (auto-created)
# Env vars: DATABASE_URL, BIND_ADDR, HTTP_ADDR
```

### Frontend
```bash
cd frontend-svelte
pnpm install
pnpm dev
# Default: http://localhost:5173
```

### Docker
```bash
docker compose up -d
# WS: localhost:7666, HTTP: localhost:7667
# SQLite persisted in named volume 'gamedata'
```

### Environment Variables (Backend)
| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `sqlite:openao.db` | SQLite connection string |
| `BIND_ADDR` | `0.0.0.0:7666` | WebSocket listen address |
| `HTTP_ADDR` | `0.0.0.0:7667` | HTTP API listen address |
| `RUST_LOG` | `openao_server=info` | Log filter |
| `CORS_ALLOWED_ORIGINS` | `*` (any) | Comma-separated allowed origins for HTTP API |
| `OPENAO_DEV_TICKETS` | `false` | Set to `1` or `true` to allow ticket reuse (dev only) |

### Environment Variables (Frontend)
| Variable | Default | Description |
|----------|---------|-------------|
| `VITE_GAME_WS_URL` | `ws://localhost:7666` | WebSocket URL for game server |
| `VITE_API_BASE_URL` | `http://localhost:7667` | HTTP API base URL |

---

## 12. Known Issues & Technical Debt

1. ~~**Password storage**: Stored as plaintext.~~ ✅ Fixed: New accounts use argon2 hashing. Login supports both hashed and legacy plaintext for backwards compat.
2. ~~**Static game data**: Items, spells, NPC types are hardcoded in Rust.~~ ✅ Fixed: All game data (1062 objects, 336 NPCs, 47 spells, 294 maps, crafting, smelting) loaded from JSON files via `game_data/mod.rs`.
3. ~~**No reconnect**: If WebSocket drops, the session is lost.~~ ✅ Fixed: Full reconnect flow — server issues reconnect tokens on connect and disconnect via ELR2 Push, client auto-reconnects with exponential backoff (3 attempts).
4. ~~**Dev-mode ticket reuse**: `consume_game_ticket` resets `consumed_at` to allow reuse.~~ ✅ Fixed: Gated behind `OPENAO_DEV_TICKETS=1` env var (off by default).
5. ~~**No map bounds validation**: Movement clamped to 1-100 hardcoded.~~ ✅ Fixed: `GameData::get_map_bounds()` returns real map width/height from terrain data; fallback 100x100.
6. **Stub gameplay modules**: `combat.rs`, `movement.rs`, `items.rs`, `challenges.rs`, `arenas.rs`, `factions.rs` contain basic validation rules with tests — future candidates for Elura-native game rules migration.
7. ~~**Entity ID overflow**: `AtomicU32` counter never resets.~~ ✅ Fixed: `next_id()` wraps naturally at `u32::MAX` and skips 0 (reserved sentinel).
8. ~~**No input validation** on WebSocket packets beyond basic reading.~~ ✅ Fixed: Rate limiting (60 pkt/s) + packet size validation (8KB max) now protects against packet flood and oversized packet abuse.
9. ~~**Legacy plaintext passwords**: Existing accounts in DB still have plaintext passwords.~~ ✅ Fixed: Automatic migration — on successful login with plaintext password, re-hashes to argon2 transparently.
10. ~~**NPC movement bounds**: NPC AI movement clamps to 1-100 hardcoded.~~ ✅ Fixed: Uses `GameData::get_map_bounds()` + `is_blocked_tile()` for both player and NPC movement.
11. ~~**Party cleanup on disconnect**: Party is disbanded if leader disconnects.~~ ✅ Fixed: Leadership transfers to next member; party only disbands when ≤1 member remains.
12. ~~**In-memory ban/mute**: Ban and mute states are not persisted to DB.~~ ✅ Fixed: `bans` and `mutes` SQLite tables with full CRUD. Bans loaded on startup, mutes loaded on character connect.

---

## 13. Packet Protocol Quick Reference

All packets are binary. First byte = packet ID. The **authoritative parser** is `registerPacketHandlers.ts` — all backend packet builders in `replication/mod.rs` and `gateway/packets.rs` are synchronized to match it exactly. Entity IDs and coordinates use `u16` (short). Strings use length-prefixed encoding.

### Server → Client (selected)
| ID | Name | Fields (types) |
|----|------|--------|
| 1 | GET_MY_CHARACTER | id:short, map:short, x:short, y:short, heading:byte, name:string, hp:short, maxHp:short, dead:byte, level:short |
| 2 | GET_CHARACTER | id:short, x:short, y:short, heading:byte, name:string, hp:short, maxHp:short, dead:byte, level:short |
| 3 | MOVE_ENTITY | id:short, x:short, y:short, heading:byte, serverTick:short |
| 5 | DELETE_CHARACTER | id:short |
| 7 | SELF_VITALS_DELTA | hp:short, maxHp:short, mana:short, maxMana:short |
| 8 | CONSOLE | text:string |
| 10 | ACT_POSITION | id:short, x:short, y:short, moveId:short |
| 20 | AGREGAR_USER_INV_ITEM | slot:byte, idItem:short, name:string, amount:short, equipped:byte, grhIndex:short, objType:byte, maxHit:short, minHit:short, maxDef:short, minDef:short, value:int |
| 30 | APRENDER_SPELL | slot:byte, spellId:short, name:string, manaCost:short |
| 40 | SELF_MAP_META_DELTA | mapId:short, mapName:string, pkFlag:byte |
| 41 | GLOBAL_NOTICE | text:string |
| 42 | ACT_MY_LEVEL | level:short |
| 43 | PARTY_STATE | count:byte, [name:string, hp:short, maxHp:short]* |
| 44 | CLAN_STATE | clanName:string, clanId:string |
| 45 | PANEL_SNAPSHOT | gold:int, exp:int, expNext:int, level:short, hp:short, maxHp:short, mana:short, maxMana:short, str:short, agi:short, int:short, con:short |

### Client → Server (selected)
| ID | Name | Fields |
|----|------|--------|
| 1 | CONNECT_CHARACTER | ticket, type, char_id |
| 2 | POSITION | heading, move_id, redundant_count, [seq:short, heading:byte]* |
| 5 | DIALOG | message |
| 10 | ATTACK_MELE | (none) |
| 12 | ATTACK_SPELL | spell_slot |
| 20 | USE_ITEM_CLICK | slot (byte) |
| 22 | TIRAR_ITEM | slot (byte), qty (short) |
| 23 | AGARRAR_ITEM | (none) |

Full opcodes: `packages/protocol/src/opcodes.ts` and `game-server-rs/crates/protocol/src/opcodes.rs`

---

## 14. Critical Fixes Applied (Conversation History)

This section documents significant bugs found and fixed during the migration process, preserved as institutional knowledge for future agents.

### 14.1 Protocol Desynchronization (Critical)

The backend inconsistently used `write_short` and `write_double` for entity IDs and coordinates. Many packets had different field orders or data types than expected by the frontend's `registerPacketHandlers.ts`. **All packet builders in `replication/mod.rs` and `gateway/packets.rs` were systematically rewritten** to precisely match the frontend parsing logic. Entity IDs and coordinates now consistently use `write_short` (u16). Call sites across all gateway modules were updated.

### 14.2 Initial State Burst on Connect

After simplifying `build_my_character_packet` to match frontend expectations, a new system was implemented to send supplementary initial data via separate packets immediately after character connection:
- `build_self_vitals` (HP, maxHP, mana, maxMana)
- `build_act_gold` (gold amount)
- `build_act_exp` (exp, exp_next_level)
- `build_self_attributes` (6 attributes)
- `build_self_flags` (safe zone, etc.)
- `build_act_color_name` (criminal/faction color)
- `build_change_equipment` (5 equipment slots: head, body, helmet, weapon, shield)
- `build_inv_item_packet` (for each inventory item)
- `build_learn_spell` (for each default spell, if class has mana)

### 14.3 USE_ITEM_U Deserialization Mismatch

Server was reading `get_int()` (4 bytes) for USE_ITEM_U slot but frontend sends `writeByte()` (1 byte). Fixed to `get_byte()`.

### 14.4 Entity Vitals Broadcast Gaps

Multiple code paths mutating HP/mana (NPC attacks, player attacks on NPCs, heal spells, sacerdote healing, respawn, admin revive, potions) were only updating the affected player's state without broadcasting `entity_vitals_delta` to nearby observers. All paths now include AOI-filtered broadcasts.

### 14.5 Drop Item Not Creating Ground Items

`handle_drop_item()` only removed items from inventory without spawning a ground item entity. Fixed to create `GroundItem` and broadcast `build_render_item` to AOI.

### 14.6 Class Name/Level Bonus Misalignment

`get_class_name()` had incorrect mappings for IDs 5-8 (was Ladron/Bardo/Druida/Paladin, fixed to Bardo/Druida/Paladin/Cazador). `class_level_bonus()` had incorrect mana gains for Bardos and misaligned HP/Mana for Paladins and Hunters.

### 14.7 Persistence Gaps

Multiple mutable player fields were not being saved: `max_hp`, `max_mana`, `dead`, `min_hit`, `max_hit`, all 4 attributes, equipment visual IDs, `navegando`, `bank_gold`, `id_clase`, `faction_rank`, `faction_score`. `save_character_state` was expanded to persist all fields. `faction_rank` and `faction_score` required new columns via ALTER TABLE migration.

### 14.8 Ground Items AOI on Teleport

Teleport was not sending ground items within view range to the teleporting player (connect did). Fixed to match connect behavior with AOI-filtered ground item packets.

### 14.9 Challenges Class Name Inconsistency

Challenges module had hardcoded class name mappings that didn't match the corrected `get_class_name()`. Replaced with centralized function call.

### 14.10 Potion Vitals Not Broadcast

HP and Mana potions were updating player state but not broadcasting `entity_vitals_delta` to AOI observers. Fixed so nearby players see health/mana bar changes when someone drinks a potion.

### 14.11 Per-Session PacketRouter Waste

Each `GameSession` was creating its own `PacketRouter::new()` (HashMap allocation + route registration on every connection). Refactored to build the router once at startup via `build_router_from_modules()` and share it across all sessions as `Arc<PacketRouter>`.

### 14.12 Missing Server-Side Movement Validation

Player and NPC movement had no collision check against terrain data. Players could walk through walls and blocked tiles. Fixed by checking `GameData::is_blocked_tile()` before applying movement in `handle_movement` and `process_npc_ai`.

### 14.13 Hardcoded Map Bounds (1-100)

All movement clamping used hardcoded `1..=100` instead of actual map dimensions. Fixed with `GameData::get_map_bounds()` that reads `MapTerrain.width/height` from terrain.json, falling back to 100x100 for maps without terrain data.

### 14.14 Party Disbands on Leader Disconnect

When a party leader disconnected, the entire party was disbanded regardless of remaining members. Fixed to transfer leadership to the next member. Party only disbands when 1 or fewer members remain.

### 14.15 In-Memory Ban/Mute Lost on Restart

Ban and mute states were stored only in `DashMap` and lost on server restart. Fixed with persistent SQLite tables (`bans`, `mutes`). Bans loaded on startup, mutes checked on character connect. Admin commands persist state to DB.

### 14.16 Legacy Plaintext Passwords Never Upgraded

Existing accounts with plaintext passwords remained unprotected. Fixed with transparent auto-migration: on successful login with a plaintext password, the hash is silently upgraded to argon2 in-place.

### 14.17 spells.json numNpc Fields as Strings

Four spells in `data/spells.json` had `numNpc` values as JSON strings (`"512"`, `"546"`, `"89"`, `"503"`) instead of integers. This caused a serde deserialization error on server startup. Fixed by adding `deserialize_i32_or_string` helper in `game_data/mod.rs` with `#[serde(deserialize_with)]` on the `num_npc` field — accepts both `i32` and string representations, which is robust for the original AO data files that sometimes mix types.

### 14.18 Axum Route Conflict on /api/characters

Two routes collided: `GET /api/characters/{account_id}` (list characters) and `DELETE /api/characters/{char_id}` (delete character). Axum treats these as the same path pattern. Fixed by changing the GET route to `/api/characters/by-account/{account_id}` to disambiguate. No frontend impact since the SvelteKit frontend uses its own server-side SQLite for character listing.

---

## 15. Useful Commands

```bash
# Check Rust compilation
cd game-server-rs && cargo check

# Run clippy (strict)
cd game-server-rs && cargo clippy -- -D warnings

# Run all Rust tests (175 tests: 160 src + 15 crates)
cd game-server-rs && cargo test

# Build release binary (LTO + stripped)
cd game-server-rs && cargo build --release

# Run frontend type checks
cd frontend-svelte && pnpm check

# Run protocol tests (51 tests)
cd packages/protocol && pnpm test

# Run frontend netcode tests (39 tests)
cd frontend-svelte && pnpm test

# Build frontend for production
cd frontend-svelte && pnpm build

# Docker: build and run
docker compose up -d --build
```

---

## 16. Development Status Summary

### Migration Progress (as of v21)

| Phase | Status | Items | Complete | % |
|-------|--------|-------|----------|---|
| Phase 1: Foundation | ✅ COMPLETED | 5 | 5 | 100% |
| Phase 2: ELR2 Protocol | ✅ COMPLETED | 16 | 16 | 100% |
| Phase 3: Architecture Split | ✅ COMPLETED | 10 | 10 | 100% |
| Phase 4: Session Management | ✅ COMPLETED | 17 | 17 | 100% |
| Phase 5: Advanced Gameplay | ✅ COMPLETED | 16 | 16 | 100% |
| Phase 6: Production Readiness | ✅ COMPLETED | 41 | 40 | 98% |
| Phase 7: Elura Full Integration | ✅ COMPLETED | 7 | 7 | 100% |
| Phase 8: Netcode Wiring | ✅ COMPLETED | 6 | 6 | 100% |
| Phase 9: Netcode Polish | ✅ COMPLETED | 8 | 8 | 100% |
| Phase 10: Hardening & Quality | ✅ COMPLETED | 8 | 8 | 100% |
| Phase 11: Enhancement (A–F) | ✅ COMPLETED | 30 | 30 | 100% |
| Phase 12: Parity & Optimization | ✅ COMPLETED | 36 | 36 | 100% |
| Phase 13: Combat Fidelity | ✅ COMPLETED | 4 | 4 | 100% |
| Phase 14: Game Systems | ✅ COMPLETED | 4 | 4 | 100% |
| Phase 15: Polish | ✅ COMPLETED | 7 | 7 | 100% |
| Phase 16: Rust Optimizations | ✅ COMPLETED | 4 | 4 | 100% |
| Phase 17: Combat & Gameplay Fidelity | ✅ COMPLETED | 10 | 10 | 100% |
| Phase 18: Gameplay Refinement | ✅ COMPLETED | 10 | 10 | 100% |
| Phase 19: Further Parity & Refinements | ✅ COMPLETED | 8 | 8 | 100% |
| Phase 20: PvP Rewards & Death Mechanics | ✅ COMPLETED | 7 | 7 | 100% |
| Phase 21: Combat & AI Fidelity | ✅ COMPLETED | 10 | 10 | 100% |
| Phase 22: NPC Summon System | ✅ COMPLETED | 4 | 4 | 100% |
| Phase 23: Missing Cooldowns | ✅ COMPLETED | 3 | 3 | 100% |
| Phase 24: Activity Logging | ✅ COMPLETED | 3 | 3 | 100% |
| Phase 25: Spell Effects Parity | ✅ COMPLETED | 4 | 4 | 100% |
| Phase 26: Fidelity & Optimization | ✅ COMPLETED | 9 | 9 | 100% |
| Phase 27: Admin & Testing | ✅ COMPLETED | 5 | 5 | 100% |
| Phase 28: Parity Audit & Polish | ✅ COMPLETED | 4 | 4 | 100% |
| Phase 29: Class ID Parity Fix | ✅ COMPLETED | 4 | 4 | 100% |
| **Total** | | **300** | **299** | **~100%** |

### Remaining Items

1. **Phase 6.41**: Evaluate Redis adapters if scaling beyond single process — N/A for monolith

### What's Fully Functional

The game server is fully playable with:
- Complete user registration, login, and character creation flow
- Full movement, combat (melee/ranged/spell PvE + PvP), and death/respawn
- 47 data-driven spells, 336 NPC types, 167 maps, 1062 objects
- Complete inventory system (20 slots, equip, use, drop, pickup, reorder)
- NPC AI (random walk, chase, melee attack with hit feedback)
- Party system (max 4 players, faction-compatible, shared XP with 10% bonus)
- Clan system (full CRUD, co-leaders, application workflow)
- Banking system (gold deposit/withdraw, item slots)
- Player-to-player market via NPC (with auto-expiry)
- NPC commerce (buy/sell)
- Crafting + smelting
- Fishing + harvesting (woodcutting/mining)
- Challenge system (1v1/2v2)
- Factions (Armada/Caos) with rank/score persistence
- Criminal system with bail
- 50+ chat commands including admin tools
- ELR2 binary protocol with `elura.v2` subprotocol negotiation
- Session reconnection (server-issued tokens + client auto-reconnect with backoff)
- AOI-filtered broadcasting (all game events)
- Lag-compensated ranged/spell combat (3-tick rewind)
- Rate limiting (60 pkt/s + per-command cooldowns + 8KB packet size limit)
- Full character state persistence (all mutable fields including faction rank/score)
- Structured error system (30+ error codes, integrated across all handlers)
- Server metrics (JSON + Prometheus, per-category packet counters)
- Graceful shutdown with 10s drain and full player save
- Optimized release binary (LTO, single codegen unit, stripped symbols)
- Elura-inspired modular route registration (`GameModule` trait, 5 domain modules)
- Per-observer entity replication via Elura `ReplicationSender` (wired into game loop)
- Per-player tick-aligned input infrastructure via Elura `InputReceiver`
- Client-side tick synchronization from ELR2 heartbeat probes (with server_received_at/server_sent_at for accurate network RTT)
- Elura Room-based challenge system (`ChallengeRoomManager` wrapping `elura::gameplay::room::Room`)
- Deterministic network simulation tests via `elura::gameplay::net_sim::SimulatedLink` (9 tests)
- Client-side InterpolationBuffer wired to entity/NPC rendering (smooth position interpolation between server ticks)
- Client-side PredictionBuffer wired to player movement (instant local prediction, server reconciliation with input replay)
- Client-side InputSender wired to movement input (sequence tracking, cumulative ACK, redundant input protocol)
- Client-side movement rate limiting (60 TPS cap prevents packet flood)
- Server echoes moveId in ACT_POSITION for precise client-side prediction reconciliation
- Server includes serverTick in MOVE_ENTITY for accurate remote entity interpolation
- ObserverReplicator sends equipment visuals on entity spawn (complete visual representation)
- Backend PlayerInputReceiver validates, de-duplicates, and reorders client movement inputs
- Redundant input protocol: POSITION packet carries historical inputs for packet-loss recovery
- TickSynchronizer fully tested (13 unit tests covering RTT separation, offset smoothing, edge cases)
- Server-side tile collision validation prevents movement onto blocked tiles (anti-cheat)
- Dynamic map bounds from terrain data (replaces all hardcoded 1-100 limits)
- Ban/mute persistence to SQLite with automatic loading on startup/connect
- Party leadership auto-transfer on leader disconnect
- Complete entity visual replication including CHANGE_ROPA on spawn
- Entity ID wrapping at u32::MAX with sentinel 0 skip
- Dev ticket reuse gated behind OPENAO_DEV_TICKETS env var
- Legacy password auto-migration to argon2 on successful login
- Buff system (tick-based agility/strength/speed buffs with duration)
- Navigation system (boats: embark/disembark, water tile restrictions)
- P2P trading (request/offer/confirm/cancel with atomic gold swap)
- Admin invisibility (AOI-filtered, toggle command)
- Jail system (timed imprisonment, blocks TP/Hogar, auto-release)
- IP ban system (SQLite persistence, checked on connection)
- Game data hot-reload (zero-downtime `/recargar` command)
- Quest system (8 quests, 5 objective types, rewards, persistence, 9 tests)
- Pet system (max 5, summon/dismiss/release, level/exp, persistence, 8 tests)
- Territory control (5 capturable zones, clan ownership, bonuses, 5 tests)
- Spell cooldowns (per-spell, tier-based defaults, 6 tests)
- Achievement system (13 achievements, 10 condition types, persistence, 6 tests)
- Real-time leaderboard (top-5 broadcast every 30s)
- Packet batching and priority system for efficient WebSocket sends
- Broadcast deduplication (ObserverReplicator + broadcast_in_range)
- Inventory caching for reduced DB queries
- IP-based rate limiting
- Structured logging with correlation IDs
- SQLite auto-backup
- CC system (paralysis/immobilization) with tick-based expiry
- Safety toggles (`/seguro`, `/seguroclan`) for PvP attack prevention
- Dead world restrictions (dead players blocked from combat/items)
- Balance system (`compute_player_stats`, `compute_damage`, `compute_spell_damage`, `compute_exp_for_kill`)
- Dual faction scores (armada + caos tracked independently)
- Item tiers, class/race restrictions, magic item modifiers
- Ground item auto-cleanup (180s lifetime, periodic sweep)
- Multi-character per account with character deletion API
- Moderation REST API (ban/unban/mute/unmute/ip-ban/ip-unban)
- Game data admin API (browse objects/NPCs/spells)
- Runtime config API (double exp/gold toggles)
- Character settings API (per-character preferences)
- Batch packet sending on connect (2 flushes vs 30+)
- SQLite WAL tuning (256 stmt cache, 8MB page cache, 256MB mmap)
- WebGPU rendering preference (Pixi.js 8)
- Exact combat formulas ported from Node.js (simulated skills, evasion, attack power, hit chance, shield block, body part absorption, stabbing)
- Exact balance formulas ported from Node.js (11-class progression, HP/Mana/Hit per level, 5-breakpoint EXP curve)
- Dead world system (15s visual transition after death)
- Gold clamping (MAX_GOLD=2,147,483,647 on all mutations)
- Working lock anti-multi-bot (prevents simultaneous gathering from same IP)
- Arena instance manager (dynamic map cloning, NPC spawning, participant tracking)
- Shared vaults (account-wide + clan-wide bank tabs with SQLite persistence)
- Connection policy (penalizes idle sessions with duplicate accounts)
- Door system (open/close with cooldowns, key requirements, visual state)
- Travel tickets (item-based teleportation with destination validation)
- NPC respawn cooldowns (per-NPC individual timers)
- Faction rank rewards (5-rank progression with level/score thresholds)
- Dragon Slayer sword (one-shot dragons, sword consumed, map entry restriction)
- Packet builder capacity hints (pre-allocated hot-path packets)
- Batch SQLite world saves (single transaction for all players)
- SmallVec for NPC loot (stack allocation for common loot tables)
- DashMap shard tuning (32 shards for reduced contention)
- Magic damage system (apply_magic_bonuses, apply_magic_resistance_to_npc/user, class modifiers, item bonuses, 6 tests)
- NPC crowd control (paralysis/immobilization applied by spells, tick-based expiry, integrated in NPC AI)
- NPC aggro system (aggro_target on NpcState, prioritize attacker in AI loop)
- Dead world visibility filtering (dead/hidden/invisible players filtered on connect/teleport)
- Arena combat integration (is_arena_map bypasses safe zone PvP checks)
- Faction PvP rules (rival faction attacks don't flag criminal, same-faction attacks do)
- Hidden skill (stealth) system (chance/duration from skill, hunter camo exemption, NPC invisibility, movement/attack reveal)
- Heal spell PvP targeting (nearest player in range, level-scaled healing, target feedback)
- Balance data hot-reloadable (/recargar reloads all game data including formulas context)
- Newbie system (NEWBIE_MAX_LEVEL=12, item restrictions, auto-unequip + removal on level 13)
- Map level restrictions (min/max level entry validation, faction portals)
- Item drop position validation (expanding radius search, avoids blocked tiles)
- Tile occupied check (prevents stacking entities on same tile)
- Unsafe logout delay (10s quiet period in PvP zones, instant in safe zones)
- Boat body resolution (dead=87, special boats preserved, default 84)
- Complete visibility system (canRenderCharacter with party/clan dead world override)
- Armada faction loss on attacking neutral citizens
- Citizen clan PvP block (citizen-aligned clan members can't attack citizens)
- Support spell PvP filter (citizens can't heal criminals outside arenas)
- NPC EXP/gold multipliers (×5 EXP, ×3 gold matching original server config)
- Armor race restrictions (razaEnana bidirectional dwarf/non-dwarf equipment check)
- Class equip restrictions (clases_no_permitidas enforced on equip)
- Admin commands: /quitarnpcpermanente, /verip, /intervalos with real packet metrics
- PvP kill faction score (Armada/Caos dual tracking, 10 pts/kill, shouldAwardArmadaScore/shouldAwardCaosScore ported)
- PvP rekill protection (5-minute window per attacker/victim pair via DashMap tracker)
- Kill counters (ciudadanos_matados/criminales_matados persisted to SQLite, incremented on PvP kills)
- PvP exp/gold rewards (base 50exp×5 + 10gold×3, double exp/gold respected, newbie victims excluded)
- PvP death cleanup (buffs/CC/meditation/stealth cleared on death)
- Enhanced bail system (cost = ciudadanos_matados × multiplicadorGold × 5000, ported from original getBailCost)
- Action cooldown system (per-action cooldowns with cross-action gates matching original vars.timing.actionCooldowns)
- Party EXP bonus corrected to 15% (was 10%, now matches original partyExpBonusPct)
- NPC spell casting (offensive/healing spells, cooldowns, FX, projectiles, magic resistance integration)
- NPC target scoring (weighted scoring: distance×3, adjacent -6, aggro -14 for smarter AI)
- Per-tile safe zones (trigger=6 tiles from specials.json, position-aware is_safe_position replaces map-only)
- Chat audit logging (structured tracing at chat_audit target for moderation)
- NPC Summon system (player-summoned NPCs via spells, max 3, 2min expiry, cleanup on disconnect)
- Drop item/equip toggle/click cooldowns (150ms/125ms/150ms preventing action spam)
- Structured activity logging (combat, economy, progression events for analytics/moderation)
- Spell effects parity (remove paralysis, invisibility, buff spells subeAg/subeFz, minSkill level gate)
- Admin bot system (/bot NPC_ID [LEVEL] with auto-heal, /bot limpiar, admin_bot_owner tracking)
- Runtime timing hot-modification (/intervalo key [value] for dynamic game loop tuning via AtomicU64)
- Expanded test suite (271 total: 166 Rust src + 15 Rust crates + 51 protocol TS + 39 frontend netcode TS)
- Class ID parity fix (original IDs 1,2,3,4,6,7,8,9 → Rust sequential 1-8 with correct combat modifier/balance values for all 14 class-indexed functions)

**Frontend pages and features:**
- Landing page, login, registration, character creation/selection
- Full game view with Pixi.js rendering (4-layer map, entity sprites, depth sorting)
- Game UI: inventory, spell bar, macro bar, chat panel, stats panel, buff sidebar
- Modals: character stats, crafting, market, challenges (retos), bail, NPC trade
- Wiki pages (items, NPCs, spells from game data)
- Ranking page with SSR
- Arenas page with join flow
- Online users stats page
- Game updates/changelog page
- Password reset flow (request + token)
- Map editor (`/construccion`) with tile painting, NPC placement, teleport triggers, blocked tiles, undo/redo
- Server-side rendering with SQLite and API proxy for auth/ranking/market
- AppChrome navigation bar with responsive design
- Weather system (rain/snow/fog/storm particle effects)
- Day/Night cycle (20min visual cycle with tint overlay)
- Minimap overlay (real-time player/entity positions)
- Toast notification system (non-blocking game event messages)
- Particle effects overlay (spell impacts, level-up sparkles)
- PixiApp decomposition (modular rendering architecture)
- NPC Inspector modal (admin NPC stat viewer)
- Admin Intervals modal (double exp/gold toggle via REST)
- Overview modal (character summary panel)
- Debug overlay (F3 toggle: position, tick, RTT, entity counts)
- Social meta tags (Open Graph + Twitter Cards)
- Wiki SSR (server-rendered wiki from game data API)

### Test Coverage

| Component | Tests | Coverage Areas |
|-----------|-------|----------------|
| ELR2 Framing | 12 | Encode/decode roundtrips, push/response/error frames, bad magic/version/length, empty payload |
| Protocol Reader/Writer | 15 | Byte/short/int/float/string/unicode roundtrips, out-of-bounds, complex packets |
| Game Module Registry | 3 | All modules register, route parity with original router, module names |
| Packet Router | 4 | Route registration, lookup, unknown routes, category grouping |
| Rate Limiter | 6 | Within limit, over limit, per-command cooldowns, IP limiter within/over limit, IP evict stale |
| Reconnect Manager | 3 | Issue/consume tokens, single-use, invalid tokens |
| Lag Compensation | 3 | Record/rewind, unknown entity, rewind window limit |
| Entity Replication | 4 | Lifecycle, despawn, ACK, reset |
| Input Queue | 4 | Accept valid input, de-duplicate redundant, reject future server tick, sequence tracking |
| Gameplay Rules | 7 | Movement validation, boundary checks, inventory add/remove/stack/full |
| Challenge Rooms | 7 | Create/join/cancel lifecycle, capacity limits, duplicate join, full room, leader succession, auto-remove empty, list filtering |
| Network Simulation | 9 | Fixed latency, total/partial loss, reordering, queue overflow, bandwidth throttle, deterministic replay, redundant input survival, jitter |
| Quests | 9 | Accept/abandon/complete lifecycle, max active limit, progress tracking, prerequisite checks, reward granting, objective types, registry loading |
| Pets | 8 | Add/summon/dismiss/release, max capacity, level-up, active pet tracking, name uniqueness |
| Territory Control | 5 | Capture progress, clan ownership transfer, contestation, bonus calculation, reset |
| Spell Cooldowns | 6 | Trigger/ready check, remaining time, cleanup, reset, default cooldowns, tier calculation |
| Achievements | 6 | Condition checking, stat tracking, unlock persistence, default achievement loading, multiple condition types |
| Balance (formulas) | 14 | Class progression (11 classes), HP/Mana/Hit per level, EXP curve breakpoints, gold clamp, level clamp, hit modifier pre/post-36, all-class HP at level 50, exp curve monotonicity, exp level 1 base value |
| Combat Formulas | 34 | Simulated skill cap, evasion by class, shield evasion, attack power (unarmed/melee/projectile), damage calculation, hit chance clamping, stabbing (chance/weapon req/skill gate/PvP result/class modifiers), body part absorption, Dragon Slayer detection, magic bonuses, magic resistance (NPC/user), hidden skill chance/duration, hidden while acting, NPC evasion scaling, newbie check, boat body resolution, NPC multipliers, dead world delay constant, unsafe logout constant, magic zero-level edge cases, PvP base rewards parity, faction rekill window, bail cost formula, all-class modifiers, unarmed wrestling range, hidden skill formula bounds |
| Arenas | 5 | Instance create/destroy lifecycle, unique map IDs, participant tracking, handover system, cleanup on empty |
| Doors | 4 | Toggle cooldown, range validation, key requirement, concurrent access |
| Buffs | 4 | Add/expire lifecycle, magnitude tracking, cleanup, multiple buff types |
| World (ActionCooldowns + RuntimeTimings) | 6 | Melee blocks until ready, cross-gate melee→spell, cross-gate spell→melee, use_item after melee gate, cooldown constants match original, runtime timings defaults |
| **Rust src Total** | **166** | |
| Rust Protocol (crates) | 15 | PacketReader (8), PacketWriter (7) — byte/short/int/float/string roundtrips, signed variants, unicode, out-of-bounds |
| **Rust Grand Total** | **181** | |
| Protocol TS | 51 | Opcode uniqueness, packet roundtrips, binary primitives, ELR2 constants, ELR2 encode/decode, ELR2 helpers, ELR2 detection, ELR2 error handling |
| Frontend Netcode TS | 39 | InterpolationBuffer (9), PredictionBuffer (8), InputSender (9), TickSynchronizer (13) — insert/sample, record/reconcile, ACK/packet, capacity, reset, RTT separation, offset smoothing |
| **Grand Total** | **271** | |
