<script lang="ts">
  import { gameState } from "$lib/game/state/gameState.svelte";
  import { assetStore } from "$lib/game/state/assetStore.svelte";

  let { npcEntityId, onClose }: { npcEntityId: number; onClose: () => void } = $props();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }

  const npc = $derived(gameState.remoteNpcs.get(npcEntityId));
  const npcData = $derived(npc ? assetStore.npcsDB?.[String(npc.npcType)] : undefined);
  const hpPct = $derived(npc && npc.maxHp > 0 ? Math.min(100, (npc.hp / npc.maxHp) * 100) : 0);
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="fixed inset-0 z-[92] flex items-center justify-center bg-black/60 px-4 py-6 backdrop-blur-sm">
  <div class="w-full max-w-sm rounded-2xl border border-white/10 bg-stone-950/95 text-stone-100 shadow-2xl">
    <div class="flex items-center justify-between border-b border-white/8 px-4 py-3">
      <p class="text-sm font-semibold text-white">
        {npcData?.name ?? `NPC #${npcEntityId}`}
      </p>
      <button
        class="rounded-lg p-1 text-stone-400 transition-colors hover:bg-white/10 hover:text-white"
        onclick={onClose}
      >✕</button>
    </div>

    <div class="space-y-3 p-4">
      {#if npc}
        <div>
          <div class="mb-1 flex items-center justify-between text-xs">
            <span class="text-stone-400">HP</span>
            <span class="text-green-400">{npc.hp} / {npc.maxHp}</span>
          </div>
          <div class="h-2 rounded-full bg-stone-800">
            <div class="h-2 rounded-full bg-green-500 transition-all" style="width: {hpPct}%"></div>
          </div>
        </div>

        <div class="grid grid-cols-2 gap-2 text-xs">
          <div class="rounded-lg bg-stone-900/60 px-3 py-2">
            <p class="text-stone-400">Tipo</p>
            <p class="font-medium">{npc.npcType}</p>
          </div>
          <div class="rounded-lg bg-stone-900/60 px-3 py-2">
            <p class="text-stone-400">Posición</p>
            <p class="font-medium">{npc.x}, {npc.y}</p>
          </div>
        </div>

        {#if npcData}
          <div class="rounded-lg bg-stone-900/60 px-3 py-2 text-xs">
            <p class="text-stone-400">Descripción</p>
            <p class="mt-0.5 font-medium">{npcData.desc ?? "Sin descripción"}</p>
          </div>
        {/if}
      {:else}
        <p class="text-sm text-stone-400">NPC no encontrado.</p>
      {/if}
    </div>
  </div>
</div>
