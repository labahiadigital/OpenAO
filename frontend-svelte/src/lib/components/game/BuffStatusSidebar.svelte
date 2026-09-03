<script lang="ts">
  import { gameState } from "$lib/game/state/gameState.svelte";

  type BuffEntry = {
    key: string;
    label: string;
    accent: string;
    icon: string;
    active: boolean;
  };

  const h = $derived(gameState.hud);

  const buffs: BuffEntry[] = $derived.by(() => {
    const list: BuffEntry[] = [];

    if (h.dead) {
      list.push({ key: "dead", label: "Muerto", accent: "bg-red-500/20 text-red-400 border-red-500/30", icon: "💀", active: true });
    }

    if (h.inmovilizado) {
      list.push({ key: "inmo", label: "Inmovilizado", accent: "bg-yellow-500/20 text-yellow-400 border-yellow-500/30", icon: "⚡", active: true });
    }

    if (h.paralizado) {
      list.push({ key: "para", label: "Paralizado", accent: "bg-purple-500/20 text-purple-400 border-purple-500/30", icon: "🔮", active: true });
    }

    if (h.navegando) {
      list.push({ key: "nav", label: "Navegando", accent: "bg-cyan-500/20 text-cyan-400 border-cyan-500/30", icon: "⛵", active: true });
    }

    if (h.seguroActivado) {
      list.push({ key: "safe", label: "Seguro", accent: "bg-blue-500/20 text-blue-400 border-blue-500/30", icon: "🛡️", active: true });
    }

    if (h.zonaSegura === 1) {
      list.push({ key: "zona", label: "Zona Segura", accent: "bg-green-500/20 text-green-400 border-green-500/30", icon: "🏠", active: true });
    }

    return list;
  });
</script>

{#if buffs.length > 0}
  <div class="flex flex-col gap-1">
    {#each buffs as buff}
      <div class="flex items-center gap-1.5 rounded-lg border {buff.accent} px-2 py-1 backdrop-blur-sm">
        <span class="text-sm">{buff.icon}</span>
        <span class="text-[11px] font-medium">{buff.label}</span>
      </div>
    {/each}
  </div>
{/if}
