<script lang="ts">
  import { gameState } from "$lib/game/state/gameState.svelte";

  let { onClose }: { onClose: () => void } = $props();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }

  const h = $derived(gameState.hud);
  const hpPct = $derived(h.maxHp > 0 ? Math.min(100, (h.hp / h.maxHp) * 100) : 0);
  const manaPct = $derived(h.maxMana > 0 ? Math.min(100, (h.mana / h.maxMana) * 100) : 0);
  const expPct = $derived(h.expNextLevel > 0 ? Math.min(100, (h.exp / h.expNextLevel) * 100) : 0);

  const equippedCount = $derived(h.inventory.filter(i => i.equipped).length);
  const freeSlots = $derived(h.inventory.filter(i => i.idItem === 0).length);
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="fixed inset-0 z-[92] flex items-center justify-center bg-black/60 px-4 py-6 backdrop-blur-sm">
  <div class="w-full max-w-md rounded-2xl border border-white/10 bg-stone-950/95 text-stone-100 shadow-2xl">
    <div class="flex items-center justify-between border-b border-white/8 px-4 py-3">
      <p class="text-sm font-semibold text-white">Resumen del Personaje</p>
      <button
        class="rounded-lg p-1 text-stone-400 transition-colors hover:bg-white/10 hover:text-white"
        onclick={onClose}
      >✕</button>
    </div>

    <div class="space-y-3 p-4 text-xs">
      <div class="grid grid-cols-3 gap-2">
        <div class="rounded-lg bg-stone-900/60 px-3 py-2 text-center">
          <p class="text-stone-400">Nivel</p>
          <p class="text-lg font-bold text-amber-400">{h.level}</p>
        </div>
        <div class="rounded-lg bg-stone-900/60 px-3 py-2 text-center">
          <p class="text-stone-400">Oro</p>
          <p class="text-lg font-bold text-yellow-400">{h.gold.toLocaleString()}</p>
        </div>
        <div class="rounded-lg bg-stone-900/60 px-3 py-2 text-center">
          <p class="text-stone-400">Mapa</p>
          <p class="text-lg font-bold">{h.map}</p>
        </div>
      </div>

      <div>
        <div class="mb-1 flex justify-between">
          <span class="text-stone-400">HP</span>
          <span class="text-green-400">{h.hp}/{h.maxHp}</span>
        </div>
        <div class="h-2 rounded-full bg-stone-800">
          <div class="h-2 rounded-full bg-green-500 transition-all" style="width: {hpPct}%"></div>
        </div>
      </div>

      <div>
        <div class="mb-1 flex justify-between">
          <span class="text-stone-400">Mana</span>
          <span class="text-blue-400">{h.mana}/{h.maxMana}</span>
        </div>
        <div class="h-2 rounded-full bg-stone-800">
          <div class="h-2 rounded-full bg-blue-500 transition-all" style="width: {manaPct}%"></div>
        </div>
      </div>

      <div>
        <div class="mb-1 flex justify-between">
          <span class="text-stone-400">EXP</span>
          <span class="text-amber-400">{h.exp}/{h.expNextLevel}</span>
        </div>
        <div class="h-2 rounded-full bg-stone-800">
          <div class="h-2 rounded-full bg-amber-500 transition-all" style="width: {expPct}%"></div>
        </div>
      </div>

      <div class="grid grid-cols-4 gap-2">
        <div class="rounded-lg bg-stone-900/60 px-2 py-1.5 text-center">
          <p class="text-stone-400">FUE</p>
          <p class="font-bold">{h.attrFuerza}</p>
        </div>
        <div class="rounded-lg bg-stone-900/60 px-2 py-1.5 text-center">
          <p class="text-stone-400">AGI</p>
          <p class="font-bold">{h.attrAgilidad}</p>
        </div>
        <div class="rounded-lg bg-stone-900/60 px-2 py-1.5 text-center">
          <p class="text-stone-400">INT</p>
          <p class="font-bold">{h.attrInteligencia}</p>
        </div>
        <div class="rounded-lg bg-stone-900/60 px-2 py-1.5 text-center">
          <p class="text-stone-400">CON</p>
          <p class="font-bold">{h.attrConstitucion}</p>
        </div>
      </div>

      <div class="flex gap-2 text-center">
        <div class="flex-1 rounded-lg bg-stone-900/60 px-2 py-1.5">
          <p class="text-stone-400">Equipados</p>
          <p class="font-bold">{equippedCount}</p>
        </div>
        <div class="flex-1 rounded-lg bg-stone-900/60 px-2 py-1.5">
          <p class="text-stone-400">Slots Libres</p>
          <p class="font-bold">{freeSlots}</p>
        </div>
      </div>
    </div>
  </div>
</div>
