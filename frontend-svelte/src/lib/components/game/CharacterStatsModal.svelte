<script lang="ts">
  import { gameState } from "$lib/game/state/gameState.svelte";

  let { onClose }: { onClose: () => void } = $props();

  const classLabels: Record<number, string> = {
    1: "Mago", 2: "Clérigo", 3: "Guerrero", 4: "Asesino",
    5: "Ladrón", 6: "Bardo", 7: "Druida", 8: "Paladín",
  };

  const factionLabels: Record<number, string> = {
    0: "Ciudadano", 1: "Armada Real", 2: "Legión Oscura",
  };

  const factionColors: Record<number, string> = {
    0: "text-stone-300", 1: "text-blue-400", 2: "text-red-400",
  };

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }

  const h = $derived(gameState.hud);
  const expPct = $derived(h.expNextLevel > 0 ? Math.min(100, (h.exp / h.expNextLevel) * 100) : 0);

  const equippedWeapon = $derived(h.inventory.find(i => i.equipped && [3, 4, 22].includes(i.objType)));
  const equippedArmor = $derived(h.inventory.find(i => i.equipped && i.objType === 8));
  const equippedHelmet = $derived(h.inventory.find(i => i.equipped && i.objType === 5));
  const equippedShield = $derived(h.inventory.find(i => i.equipped && i.objType === 6));
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="fixed inset-0 z-[92] flex items-center justify-center bg-black/60 px-4 py-6 backdrop-blur-sm">
  <div class="w-full max-w-md rounded-2xl border border-white/10 bg-stone-950/95 text-stone-100 shadow-2xl">
    <div class="flex items-center justify-between border-b border-white/8 px-4 py-3">
      <div>
        <p class="text-sm font-semibold text-white">{h.name || "Personaje"}</p>
        <p class="text-xs text-stone-400">Nivel {h.level}</p>
      </div>
      <button onclick={onClose}
        class="rounded-md border border-white/10 px-2.5 py-1 text-xs text-stone-300 hover:border-white/20 hover:text-white transition">
        Cerrar
      </button>
    </div>

    <div class="space-y-3 p-4 max-h-[70vh] overflow-y-auto">
      <!-- General Info -->
      <section class="rounded-lg bg-white/[2%] p-3 space-y-2">
        <p class="text-[10px] uppercase tracking-wider text-stone-500">General</p>
        <div class="grid grid-cols-2 gap-x-4 gap-y-1 text-xs">
          <span class="text-stone-400">Nivel</span>
          <span class="text-right text-stone-200">{h.level}</span>
          <span class="text-stone-400">Experiencia</span>
          <span class="text-right text-stone-200">{h.exp.toLocaleString()} / {h.expNextLevel.toLocaleString()}</span>
          <span class="text-stone-400">Oro</span>
          <span class="text-right text-yellow-400">{h.gold.toLocaleString()}</span>
          <span class="text-stone-400">Facción</span>
          <span class="text-right {factionColors[h.nameColor] ?? 'text-stone-200'}">{factionLabels[h.nameColor] ?? 'Ciudadano'}</span>
        </div>
        <div class="relative h-2 w-full rounded-full bg-stone-800 overflow-hidden mt-1">
          <div class="h-full rounded-full bg-gradient-to-r from-purple-500 to-purple-400 transition-all"
            style="width: {expPct}%"></div>
        </div>
        <p class="text-[10px] text-stone-500 text-right">{expPct.toFixed(1)}% al siguiente nivel</p>
      </section>

      <!-- Vitales -->
      <section class="rounded-lg bg-white/[2%] p-3 space-y-2">
        <p class="text-[10px] uppercase tracking-wider text-stone-500">Vitales</p>
        <div class="grid grid-cols-2 gap-x-4 gap-y-1 text-xs">
          <span class="text-stone-400">HP</span>
          <span class="text-right text-red-400">{h.hp} / {h.maxHp}</span>
          <span class="text-stone-400">Mana</span>
          <span class="text-right text-blue-400">{h.mana} / {h.maxMana}</span>
        </div>
      </section>

      <!-- Atributos -->
      <section class="rounded-lg bg-white/[2%] p-3 space-y-2">
        <p class="text-[10px] uppercase tracking-wider text-stone-500">Atributos</p>
        <div class="grid grid-cols-2 gap-x-4 gap-y-1 text-xs">
          <span class="text-stone-400">Fuerza</span>
          <span class="text-right text-orange-400">{h.attrFuerza}</span>
          <span class="text-stone-400">Agilidad</span>
          <span class="text-right text-green-400">{h.attrAgilidad}</span>
          <span class="text-stone-400">Inteligencia</span>
          <span class="text-right text-cyan-400">{h.attrInteligencia}</span>
          <span class="text-stone-400">Constitución</span>
          <span class="text-right text-amber-400">{h.attrConstitucion}</span>
        </div>
      </section>

      <!-- Combate -->
      <section class="rounded-lg bg-white/[2%] p-3 space-y-2">
        <p class="text-[10px] uppercase tracking-wider text-stone-500">Combate</p>
        <div class="grid grid-cols-2 gap-x-4 gap-y-1 text-xs">
          <span class="text-stone-400">Daño</span>
          <span class="text-right text-stone-200">{h.minHit} - {h.maxHit}</span>
        </div>
      </section>

      <!-- Equipamiento -->
      <section class="rounded-lg bg-white/[2%] p-3 space-y-2">
        <p class="text-[10px] uppercase tracking-wider text-stone-500">Equipamiento</p>
        <div class="grid grid-cols-2 gap-x-4 gap-y-1 text-xs">
          <span class="text-stone-400">Arma</span>
          <span class="text-right text-stone-200">{equippedWeapon?.name ?? 'Ninguna'}</span>
          <span class="text-stone-400">Armadura</span>
          <span class="text-right text-stone-200">{equippedArmor?.name ?? 'Ninguna'}</span>
          <span class="text-stone-400">Casco</span>
          <span class="text-right text-stone-200">{equippedHelmet?.name ?? 'Ninguno'}</span>
          <span class="text-stone-400">Escudo</span>
          <span class="text-right text-stone-200">{equippedShield?.name ?? 'Ninguno'}</span>
        </div>
      </section>

      <!-- Flags -->
      <section class="rounded-lg bg-white/[2%] p-3 space-y-2">
        <p class="text-[10px] uppercase tracking-wider text-stone-500">Estado</p>
        <div class="flex flex-wrap gap-1.5">
          {#if h.dead}
            <span class="rounded bg-red-500/20 px-2 py-0.5 text-[11px] text-red-400">Muerto</span>
          {/if}
          {#if h.navegando}
            <span class="rounded bg-cyan-500/20 px-2 py-0.5 text-[11px] text-cyan-400">Navegando</span>
          {/if}
          {#if h.inmovilizado}
            <span class="rounded bg-yellow-500/20 px-2 py-0.5 text-[11px] text-yellow-400">Inmovilizado</span>
          {/if}
          {#if h.paralizado}
            <span class="rounded bg-purple-500/20 px-2 py-0.5 text-[11px] text-purple-400">Paralizado</span>
          {/if}
          {#if h.zonaSegura === 1}
            <span class="rounded bg-green-500/20 px-2 py-0.5 text-[11px] text-green-400">Zona Segura</span>
          {/if}
          {#if h.seguroActivado}
            <span class="rounded bg-blue-500/20 px-2 py-0.5 text-[11px] text-blue-400">Seguro ON</span>
          {/if}
          {#if !h.dead && !h.navegando && !h.inmovilizado && !h.paralizado}
            <span class="rounded bg-green-500/10 px-2 py-0.5 text-[11px] text-green-300">Normal</span>
          {/if}
        </div>
      </section>
    </div>
  </div>
</div>
