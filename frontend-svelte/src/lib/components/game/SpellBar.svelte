<script lang="ts">
  import type { SpellEntry } from "$lib/game/lib/aowProtocol";
  import { sendAttackSpell } from "$lib/game/session/outgoingRequests";

  let { spells }: { spells: SpellEntry[] } = $props();

  const TOTAL_SLOTS = 10;

  let slots = $derived(() => {
    const grid: (SpellEntry | null)[] = new Array(TOTAL_SLOTS).fill(null);
    for (const spell of spells) {
      if (spell.slot >= 0 && spell.slot < TOTAL_SLOTS) {
        grid[spell.slot] = spell;
      }
    }
    return grid;
  });

  function castSpell(slot: number) {
    sendAttackSpell(slot);
  }
</script>

<div class="flex gap-1">
  {#each slots() as spell, idx}
    <button
      onclick={() => castSpell(idx)}
      class="w-10 h-10 rounded-lg border transition relative flex items-center justify-center
        {spell
          ? 'border-purple-500/30 bg-purple-500/10 hover:bg-purple-500/20 text-purple-300'
          : 'border-white/5 bg-black/40 text-stone-600'}"
      title={spell ? `${spell.name} (Mana: ${spell.manaRequired})` : `Slot ${idx + 1}`}
    >
      {#if spell}
        <span class="text-[9px] leading-tight text-center">{spell.name.slice(0, 5)}</span>
      {:else}
        <span class="text-[10px]">{idx + 1}</span>
      {/if}
    </button>
  {/each}
</div>
