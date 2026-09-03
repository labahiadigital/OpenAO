<script lang="ts">
  import type { HudState } from "$lib/game/state/gameState.svelte";

  let { hud, mapName }: { hud: HudState; mapName: string } = $props();

  let expPercent = $derived(
    hud.expNextLevel > 0 ? Math.min(100, (hud.exp / hud.expNextLevel) * 100) : 0,
  );
  let hpPercent = $derived(hud.maxHp > 0 ? (hud.hp / hud.maxHp) * 100 : 0);
  let manaPercent = $derived(hud.maxMana > 0 ? (hud.mana / hud.maxMana) * 100 : 0);
</script>

<div class="flex flex-col gap-2 text-xs">
  <!-- Character header -->
  <section class="rounded-lg border border-white/10 bg-white/[2%] p-2.5">
    <div class="flex items-center gap-2">
      <div class="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-gradient-to-br from-amber-400/20 to-amber-600/10 text-sm font-bold text-amber-300 border border-amber-400/20">
        {hud.level}
      </div>
      <div class="min-w-0 flex-1">
        <p class="truncate text-sm font-semibold text-white">
          {hud.name || "Aventurero"}
        </p>
        <p class="text-[10px] text-stone-500">
          {mapName || `Mapa ${hud.map}`}
        </p>
      </div>
    </div>

    <!-- EXP bar -->
    <div class="mt-2">
      <div class="flex items-center justify-between text-[10px] text-stone-500 mb-0.5">
        <span>EXP</span>
        <span>{hud.exp.toLocaleString()} / {hud.expNextLevel.toLocaleString()}</span>
      </div>
      <div class="h-1.5 rounded-full bg-white/5 overflow-hidden">
        <div
          class="h-full rounded-full bg-gradient-to-r from-emerald-500 to-emerald-400 transition-all duration-200"
          style="width: {expPercent}%"
        ></div>
      </div>
    </div>
  </section>

  <!-- Vitals -->
  <section class="rounded-lg border border-white/10 bg-white/[2%] p-2.5">
    <div class="space-y-2">
      <!-- HP Bar -->
      <div>
        <div class="flex items-center justify-between text-[10px] mb-0.5">
          <span class="text-red-400">HP</span>
          <span class="text-stone-400">{hud.hp}/{hud.maxHp}</span>
        </div>
        <div class="h-2 rounded-full bg-white/5 overflow-hidden">
          <div
            class="h-full rounded-full bg-gradient-to-r from-red-600 to-red-500 transition-all duration-200"
            style="width: {hpPercent}%"
          ></div>
        </div>
      </div>

      <!-- Mana Bar -->
      <div>
        <div class="flex items-center justify-between text-[10px] mb-0.5">
          <span class="text-blue-400">MANA</span>
          <span class="text-stone-400">{hud.mana}/{hud.maxMana}</span>
        </div>
        <div class="h-2 rounded-full bg-white/5 overflow-hidden">
          <div
            class="h-full rounded-full bg-gradient-to-r from-blue-600 to-blue-500 transition-all duration-200"
            style="width: {manaPercent}%"
          ></div>
        </div>
      </div>
    </div>

    <!-- Stats grid -->
    <div class="mt-2.5 grid grid-cols-2 gap-x-3 gap-y-1 text-[10px]">
      <div class="flex items-center justify-between">
        <span class="text-stone-500">Oro</span>
        <span class="text-yellow-400 tabular-nums">{hud.gold.toLocaleString()}</span>
      </div>
      <div class="flex items-center justify-between">
        <span class="text-stone-500">Hit</span>
        <span class="text-stone-300 tabular-nums">{hud.minHit}-{hud.maxHit}</span>
      </div>
      <div class="flex items-center justify-between">
        <span class="text-red-400/80">FUE</span>
        <span class="tabular-nums">{hud.attrFuerza}</span>
      </div>
      <div class="flex items-center justify-between">
        <span class="text-green-400/80">AGI</span>
        <span class="tabular-nums">{hud.attrAgilidad}</span>
      </div>
    </div>

    <!-- Map info -->
    <div class="mt-2 flex items-center gap-2 text-[10px]">
      <span class="text-stone-500">Pos:</span>
      <span class="text-stone-300 tabular-nums">({hud.pos.x}, {hud.pos.y})</span>
    </div>

    {#if hud.zonaSegura}
      <p class="mt-1.5 text-[10px] font-medium text-cyan-400">⚔ Zona segura</p>
    {/if}
  </section>
</div>
