<script lang="ts">
  import PixiApp from "./PixiApp.svelte";
  import ParticleOverlay from "./ParticleOverlay.svelte";
  import Minimap from "./Minimap.svelte";
  import ToastContainer from "./ToastContainer.svelte";
  import WeatherOverlay from "./WeatherOverlay.svelte";
  import ChatPanel from "./ChatPanel.svelte";
  import InventoryPanel from "./InventoryPanel.svelte";
  import StatsPanel from "./StatsPanel.svelte";
  import TradeModal from "./TradeModal.svelte";
  import CraftingModal from "./CraftingModal.svelte";
  import MarketModal from "./MarketModal.svelte";
  import RetosModal from "./RetosModal.svelte";
  import BailModal from "./BailModal.svelte";
  import CharacterStatsModal from "./CharacterStatsModal.svelte";
  import NpcInspectorModal from "./NpcInspectorModal.svelte";
  import AdminIntervalsModal from "./AdminIntervalsModal.svelte";
  import OverviewModal from "./OverviewModal.svelte";
  import DebugOverlay from "./DebugOverlay.svelte";
  import { gameSession } from "$lib/game/session/gameSession.svelte";
  import { gameState, WALK_STEP_MS } from "$lib/game/state/gameState.svelte";
  import { assetStore } from "$lib/game/state/assetStore.svelte";
  import {
    registerAllPacketHandlers,
    sendPingWithTimestamp,
  } from "$lib/game/session/registerPacketHandlers";
  import {
    sendDialog,
    sendPosition,
    sendChangeHeading,
    sendAttackMelee,
    sendPickupItem,
    sendToggleSafe,
  } from "$lib/game/session/outgoingRequests";
  import { mapState } from "$lib/game/state/mapState.svelte";
  import { onDestroy, onMount } from "svelte";

  const WS_URL = import.meta.env.VITE_GAME_WS_URL || "ws://localhost:7666";
  const API_BASE = import.meta.env.VITE_API_BASE_URL || "";

  let handlersRegistered = false;
  let pingInterval: ReturnType<typeof setInterval> | undefined;
  let ticket = $state("");
  let hasValidTicket = $derived(ticket !== "" && ticket !== "no-ticket");
  let castBarPct = $state(0);
  let castBarRaf: number | undefined;
  let connectCalled = false;

  // ── Movement system (mirrors the original Engine.check / moveTo loop) ──
  // We track which direction keys are currently held down and use our own
  // timer-based loop that fires every WALK_STEP_MS while at least one key
  // is pressed.  This gives us:
  //  • Precise timing independent of OS key-repeat rate
  //  • Smooth chaining of tile-step animations
  //  • Priority: last-pressed direction wins (like the original)
  const heldDirections = new Set<number>(); // heading values: 1=up, 2=down, 3=right, 4=left
  let directionPriority: number[] = []; // most-recently pressed first
  let moveLoopTimer: ReturnType<typeof setTimeout> | undefined;
  let lastStepTime = 0;

  function startMoveLoop() {
    if (moveLoopTimer !== undefined) return;
    // First step: execute immediately and record the time.
    attemptStep();
  }

  function stopMoveLoop() {
    if (moveLoopTimer !== undefined) {
      clearTimeout(moveLoopTimer);
      moveLoopTimer = undefined;
    }
  }

  function attemptStep() {
    moveLoopTimer = undefined;
    if (heldDirections.size === 0) return;
    if (gameSession.connectionState !== "connected" && gameSession.connectionState !== "authenticated") return;

    let heading: number | undefined;
    for (const h of directionPriority) {
      if (heldDirections.has(h)) { heading = h; break; }
    }
    if (heading === undefined) return;

    const now = performance.now();
    const elapsed = now - lastStepTime;

    if (elapsed >= WALK_STEP_MS) {
      doMove(heading, now);
      lastStepTime = now;
      // Schedule next step exactly WALK_STEP_MS from now.
      moveLoopTimer = setTimeout(attemptStep, WALK_STEP_MS);
    } else {
      // Wait the remaining time.
      moveLoopTimer = setTimeout(attemptStep, WALK_STEP_MS - elapsed);
    }
  }

  function doMove(heading: number, now: number) {
    const { x, y } = gameState.hud.pos;
    const dx = heading === 3 ? 1 : heading === 4 ? -1 : 0;
    const dy = heading === 2 ? 1 : heading === 1 ? -1 : 0;
    const nx = x + dx;
    const ny = y + dy;

    if (mapState.isTileBlocked(nx, ny)) {
      sendChangeHeading(heading);
      return;
    }

    for (const [, npc] of gameState.remoteNpcs) {
      if (npc.x === nx && npc.y === ny) {
        sendChangeHeading(heading);
        return;
      }
    }

    for (const [, e] of gameState.remoteEntities) {
      if (e.x === nx && e.y === ny && !e.dead) {
        sendChangeHeading(heading);
        return;
      }
    }

    const tick = gameState.nextMoveTick();
    gameState.predictionBuffer.record(tick, { heading }, { x: nx, y: ny });
    gameState.inputSender.record(tick, { heading });
    gameState.mergeHud({ pos: { x: nx, y: ny }, heading });

    gameState.playerMoveAnim = {
      startedAt: now,
      durationMs: WALK_STEP_MS,
      dx,
      dy,
    };

    sendPosition(heading, tick);
  }

  onMount(() => {
    assetStore.load();
    ticket = localStorage.getItem("game_ticket") || "no-ticket";
  });

  function connect() {
    if (connectCalled) return;
    connectCalled = true;
    if (!handlersRegistered) {
      registerAllPacketHandlers();
      handlersRegistered = true;
    }
    gameState.reset();
    gameSession.connect(WS_URL, ticket, 0, 0);
    localStorage.removeItem("game_ticket");
    pingInterval = setInterval(sendPingWithTimestamp, 10000);
    setTimeout(sendPingWithTimestamp, 500);
  }

  function disconnect() {
    if (pingInterval) clearInterval(pingInterval);
    pingInterval = undefined;
    gameSession.disconnect();
    gameState.reset();
    connectCalled = false;
  }

  const KEY_TO_HEADING: Record<string, number> = {
    ArrowUp: 1, w: 1,
    ArrowDown: 2, s: 2,
    ArrowRight: 3, d: 3,
    ArrowLeft: 4, a: 4,
  };

  function handleKeydown(e: KeyboardEvent) {
    if (gameSession.connectionState !== "connected" && gameSession.connectionState !== "authenticated") return;
    if ((e.target as HTMLElement)?.tagName === "INPUT" || (e.target as HTMLElement)?.tagName === "TEXTAREA") return;

    const heading = KEY_TO_HEADING[e.key];
    if (heading !== undefined) {
      e.preventDefault();
      if (!heldDirections.has(heading)) {
        heldDirections.add(heading);
        directionPriority = [heading, ...directionPriority.filter(h => h !== heading)];
      }
      startMoveLoop();
      return;
    }

    switch (e.key) {
      case " ": e.preventDefault(); sendAttackMelee(); break;
      case "g": sendPickupItem(); break;
      case "u": sendToggleSafe(); break;
      case "c": gameState.showCharacterStats = !gameState.showCharacterStats; break;
      case "o": gameState.showOverview = !gameState.showOverview; break;
      case "F3": e.preventDefault(); gameState.showDebugOverlay = !gameState.showDebugOverlay; break;
      case "Escape": gameState.pendingSpellSlot = null; break;
      default: break;
    }
  }

  function handleKeyup(e: KeyboardEvent) {
    const heading = KEY_TO_HEADING[e.key];
    if (heading !== undefined) {
      heldDirections.delete(heading);
      directionPriority = directionPriority.filter(h => h !== heading);
      if (heldDirections.size === 0) {
        stopMoveLoop();
      }
    }
  }

  let isConnected = $derived(gameSession.connectionState === "connected" || gameSession.connectionState === "authenticated");
  let isConnecting = $derived(gameSession.connectionState === "connecting" || gameSession.connectionState === "authenticating");
  let isReconnecting = $derived(gameSession.connectionState === "reconnecting");

  $effect(() => {
    const bar = gameState.castBar;
    if (!bar || bar.entityId !== gameState.hud.id) {
      castBarPct = 0;
      if (castBarRaf !== undefined) cancelAnimationFrame(castBarRaf);
      castBarRaf = undefined;
      return;
    }
    function tick() {
      const b = gameState.castBar;
      if (!b) { castBarPct = 0; return; }
      const elapsed = Date.now() - b.startMs;
      castBarPct = Math.min(100, (elapsed / b.durationMs) * 100);
      if (castBarPct < 100) castBarRaf = requestAnimationFrame(tick);
    }
    castBarRaf = requestAnimationFrame(tick);
    return () => { if (castBarRaf !== undefined) cancelAnimationFrame(castBarRaf); };
  });

  onDestroy(() => { if (castBarRaf !== undefined) cancelAnimationFrame(castBarRaf); });

  $effect(() => {
    if (hasValidTicket && gameSession.connectionState === "disconnected") {
      connect();
    }
  });

  onDestroy(() => {
    if (pingInterval) clearInterval(pingInterval);
    stopMoveLoop();
    gameSession.disconnect();
  });
</script>

<svelte:window onkeydown={handleKeydown} onkeyup={handleKeyup} onblur={() => { heldDirections.clear(); directionPriority = []; stopMoveLoop(); }} />

<!-- Connection overlay -->
{#if !isConnected}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-[#0a0714]">
    <div class="rounded-2xl border border-cyan-300/15 bg-slate-950/90 p-8 text-center max-w-md backdrop-blur-xl shadow-2xl">
      <h2 class="text-2xl font-bold text-amber-200 mb-1">OpenAO</h2>
      <p class="text-xs text-stone-500 mb-6">Conecta al servidor de juego</p>

      {#if gameSession.error}
        <div class="mb-4 rounded-lg bg-red-500/10 border border-red-500/20 p-3 text-sm text-red-400">
          {gameSession.error}
        </div>
      {/if}

      {#if hasValidTicket}
        <button
          onclick={connect}
          disabled={isConnecting}
          class="w-full rounded-xl bg-amber-400 px-6 py-3 font-semibold text-stone-950 hover:bg-amber-300 transition disabled:opacity-50"
        >
          {isConnecting ? "Conectando..." : "Conectar al servidor"}
        </button>
      {:else if !isConnecting}
        <p class="text-sm text-stone-400 mb-4">Inicia sesion para jugar</p>
        <a
          href="/login"
          class="inline-block w-full rounded-xl bg-amber-400 px-6 py-3 font-semibold text-stone-950 hover:bg-amber-300 transition text-center"
        >
          Iniciar sesion
        </a>
        <a
          href="/register"
          class="mt-3 inline-block text-sm text-stone-400 hover:text-stone-200 transition"
        >
          Crear cuenta
        </a>
      {:else}
        <div class="py-3 text-stone-400">Conectando...</div>
      {/if}
      <p class="mt-3 text-[10px] text-stone-600">{WS_URL}</p>
    </div>
  </div>
{/if}

<!-- Reconnecting overlay -->
{#if isReconnecting}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm pointer-events-none">
    <div class="rounded-xl border border-amber-400/20 bg-slate-950/90 px-8 py-5 text-center shadow-2xl pointer-events-auto">
      <div class="mb-2 h-5 w-5 mx-auto animate-spin rounded-full border-2 border-amber-400 border-t-transparent"></div>
      <p class="text-sm font-medium text-amber-200">Reconectando...</p>
      <p class="mt-1 text-[11px] text-stone-500">Intentando restablecer la conexion</p>
    </div>
  </div>
{/if}

<!-- Toast notifications -->
<ToastContainer />

<!-- Main game layout -->
{#if (isConnected || isReconnecting) && gameState.sceneReady}
  <div class="game-hud flex h-screen w-screen overflow-hidden bg-black">
    <!-- Left: Game canvas + overlays -->
    <div class="flex-1 relative min-w-0 flex flex-col">
      <!-- Top bar -->
      <div class="h-auto shrink-0 bg-[#1a1a1a] border-b-2 border-[#555] px-2 py-0.5 flex items-center justify-between">
        <div class="flex items-center gap-2">
          <span class="text-[9px] text-[#cbb18e] font-bold">
            {gameState.mapName || `Mapa ${gameState.hud.map}`} — ({gameState.hud.pos.x}, {gameState.hud.pos.y})
          </span>
        </div>
        <div class="flex items-center gap-2">
          <span class="text-[8px] text-stone-500">{gameState.pingText}</span>
          {#if gameState.hud.dead}
            <span class="text-[9px] font-bold text-red-400 animate-pulse">MUERTO</span>
          {/if}
          <button
            onclick={disconnect}
            class="border border-[#660000] bg-[#2a1111] px-2 py-0.5 text-[8px] font-bold text-red-400 hover:bg-[#331111]"
          >
            Desconectar
          </button>
        </div>
      </div>

      <!-- Pixi canvas area -->
      <div class="flex-1 relative min-h-0">
        <PixiApp />
        <ParticleOverlay />
        <WeatherOverlay />
        <Minimap />

        <!-- Cast bar overlay -->
        {#if gameState.castBar && gameState.castBar.entityId === gameState.hud.id}
          <div class="absolute bottom-24 left-1/2 -translate-x-1/2 z-40 w-48">
            <div class="h-3 bg-[#222] border border-[#555] overflow-hidden">
              <div class="h-full bg-[#00aacc]" style="width: {castBarPct}%"></div>
            </div>
            <p class="text-[8px] text-center text-stone-400 mt-0.5">Canalizando... {Math.floor(castBarPct)}%</p>
          </div>
        {/if}

        <!-- Buff status indicators -->
        <div class="absolute top-1 left-1 z-40 flex flex-col gap-0.5">
          {#if gameState.hud.dead}
            <span class="bg-[#330000] border border-[#660000] px-1.5 py-0 text-[8px] font-bold text-red-400">Muerto</span>
          {/if}
          {#if gameState.hud.inmovilizado}
            <span class="bg-[#332200] border border-[#665500] px-1.5 py-0 text-[8px] font-bold text-yellow-400">Inmovilizado</span>
          {/if}
          {#if gameState.hud.paralizado}
            <span class="bg-[#220033] border border-[#440066] px-1.5 py-0 text-[8px] font-bold text-purple-400">Paralizado</span>
          {/if}
          {#if gameState.hud.navegando}
            <span class="bg-[#001133] border border-[#003366] px-1.5 py-0 text-[8px] font-bold text-blue-400">Navegando</span>
          {/if}
          {#if gameState.hud.seguroActivado}
            <span class="bg-[#003300] border border-[#006600] px-1.5 py-0 text-[8px] font-bold text-emerald-400">Seguro</span>
          {/if}
          {#if gameState.hud.zonaSegura}
            <span class="bg-[#003333] border border-[#006666] px-1.5 py-0 text-[8px] font-bold text-cyan-400">Zona Segura</span>
          {/if}
        </div>

        <!-- Party overlay -->
        {#if gameState.hud.partyMembers.length > 0}
          <div class="absolute top-1 right-1 z-40 w-36">
            <div class="bg-[#111] border border-[#555] p-1">
              <p class="text-[7px] uppercase tracking-wider text-stone-500 mb-0.5 px-0.5">Party</p>
              {#each gameState.hud.partyMembers as member}
                <div class="flex items-center gap-1 px-0.5 py-0 {member.isLeader ? 'bg-[#332200]' : ''}">
                  <span class="w-1 h-1 {member.online ? 'bg-green-400' : 'bg-stone-600'}"></span>
                  <span class="text-[8px] text-stone-300 truncate flex-1">{member.nameCharacter}</span>
                  {#if member.isLeader}
                    <span class="text-[7px] text-amber-400">★</span>
                  {/if}
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>

      <!-- Bottom: Chat panel -->
      <div class="shrink-0 w-full max-w-lg p-1">
        <ChatPanel
          messages={gameState.chatMessages}
          consoleMessages={gameState.consoleMessages}
          onSendChat={(msg) => {
            sendDialog(msg);
            gameState.addChat(gameState.hud.name || "Yo", msg, "#fcd34d");
            if (gameState.hud.id > 0) {
              gameState.setEntityChatText(gameState.hud.id, msg);
            }
          }}
        />
      </div>

      <!-- Bottom bar -->
      <div class="shrink-0 bg-[#1a1a1a] border-t-2 border-[#555] px-2 py-1 flex items-center justify-center gap-1">
        <button
          class="border border-[#555] bg-[#2a2a2a] px-3 py-0.5 text-[8px] font-bold uppercase tracking-wider text-amber-200 hover:bg-[#333]"
        >
          Arenas
        </button>
        <button
          class="border border-[#555] bg-[#2a2a2a] px-3 py-0.5 text-[8px] font-bold uppercase tracking-wider text-amber-200 hover:bg-[#333]"
        >
          Personaje
        </button>
        <button
          onclick={() => { gameState.showCharacterStats = !gameState.showCharacterStats; }}
          class="border border-[#555] bg-[#2a2a2a] px-3 py-0.5 text-[8px] font-bold uppercase tracking-wider text-amber-200 hover:bg-[#333]"
          title="Estadisticas (C)"
        >
          Stats
        </button>
      </div>
    </div>

    <!-- Right sidebar -->
    <div class="w-[220px] shrink-0 flex flex-col bg-[#111] border-l-2 border-[#555] overflow-y-auto">
      <div class="p-1.5 flex flex-col gap-1 flex-1">
        <StatsPanel hud={gameState.hud} mapName={gameState.mapName} />
        <div class="flex-1 min-h-0">
          <InventoryPanel
            inventory={gameState.hud.inventory}
            spells={gameState.hud.spells}
          />
        </div>
      </div>
    </div>
  </div>

  <!-- Modals -->
  {#if gameState.tradeState}
    <TradeModal
      merchantItems={gameState.tradeState.merchantItems ?? []}
      onClose={() => { gameState.tradeState = null; }}
    />
  {/if}

  {#if gameState.craftingState}
    <CraftingModal
      craftingState={gameState.craftingState}
      onClose={() => { gameState.craftingState = null; }}
    />
  {/if}

  {#if gameState.marketState}
    <MarketModal
      marketState={gameState.marketState}
      onClose={() => { gameState.marketState = null; }}
    />
  {/if}

  {#if gameState.retosState}
    <RetosModal
      retosState={gameState.retosState}
      onClose={() => { gameState.retosState = null; }}
    />
  {/if}

  {#if gameState.bailOffer}
    <BailModal
      bail={gameState.bailOffer}
      onClose={() => { gameState.bailOffer = null; }}
    />
  {/if}

  {#if gameState.showCharacterStats}
    <CharacterStatsModal
      onClose={() => { gameState.showCharacterStats = false; }}
    />
  {/if}

  {#if gameState.showNpcInspector !== null}
    <NpcInspectorModal
      npcEntityId={gameState.showNpcInspector}
      onClose={() => { gameState.showNpcInspector = null; }}
    />
  {/if}

  {#if gameState.showAdminIntervals}
    <AdminIntervalsModal
      onClose={() => { gameState.showAdminIntervals = false; }}
    />
  {/if}

  {#if gameState.showOverview}
    <OverviewModal
      onClose={() => { gameState.showOverview = false; }}
    />
  {/if}

  {#if gameState.showDebugOverlay}
    <DebugOverlay
      onClose={() => { gameState.showDebugOverlay = false; }}
    />
  {/if}
{:else if isConnected}
  <div class="flex h-screen w-screen items-center justify-center bg-[#0a0714]">
    <div class="text-center">
      <div class="mb-4 h-8 w-8 mx-auto animate-spin rounded-full border-2 border-amber-400 border-t-transparent"></div>
      <p class="text-sm text-stone-400">Cargando mundo...</p>
    </div>
  </div>
{/if}
