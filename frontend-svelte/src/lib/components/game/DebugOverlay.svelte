<script lang="ts">
  import { gameState } from "$lib/game/state/gameState.svelte";
  import { gameSession } from "$lib/game/session/gameSession.svelte";

  let { onClose }: { onClose: () => void } = $props();

  const h = $derived(gameState.hud);
  const entityCount = $derived(gameState.remoteEntities.size);
  const npcCount = $derived(gameState.remoteNpcs.size);
  const groundItemCount = $derived(gameState.groundItems.size);
  const ts = $derived(gameSession.tickSync);
</script>

<div class="pointer-events-auto fixed right-2 top-14 z-[91] w-56 rounded-xl border border-white/10 bg-stone-950/90 text-[10px] text-stone-300 shadow-lg">
  <div class="flex items-center justify-between border-b border-white/8 px-3 py-1.5">
    <p class="font-semibold text-white">Debug</p>
    <button
      class="text-stone-400 transition-colors hover:text-white"
      onclick={onClose}
    >✕</button>
  </div>
  <div class="space-y-0.5 px-3 py-2 font-mono">
    <p>pos: {h.pos.x},{h.pos.y} map:{h.map}</p>
    <p>heading: {h.heading}</p>
    <p>entities: {entityCount}</p>
    <p>npcs: {npcCount}</p>
    <p>ground items: {groundItemCount}</p>
    <p>tick: {ts.estimatedServerTick}</p>
    <p>rtt: {ts.totalRttMs.toFixed(0)}ms (net: {ts.networkRttMs.toFixed(0)}ms)</p>
    <p>1way: {ts.oneWayDelayMs.toFixed(0)}ms</p>
    <p>hp: {h.hp}/{h.maxHp} mana: {h.mana}/{h.maxMana}</p>
    <p>gold: {h.gold} lvl: {h.level}</p>
    <p>dead: {h.dead}</p>
  </div>
</div>
