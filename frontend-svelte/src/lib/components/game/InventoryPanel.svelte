<script lang="ts">
  import type { InventoryItem, SpellEntry } from "$lib/game/lib/aowProtocol";
  import { sendEquipItem, sendUseItemClick, sendDropItem, sendAttackSpell, sendReorderInventory } from "$lib/game/session/outgoingRequests";
  import ItemGraphic from "./ItemGraphic.svelte";
  import { assetStore } from "$lib/game/state/assetStore.svelte";

  let { inventory, spells }: { inventory: InventoryItem[]; spells: SpellEntry[] } = $props();

  let activeTab = $state<"inventory" | "spells">("inventory");
  let selectedSlot = $state<number | null>(null);
  let selectedSpellSlot = $state<number | null>(null);
  let hoveredItem = $state<InventoryItem | null>(null);

  const TOTAL_SLOTS = 20;
  const INV_COLS = 5;

  let slots = $derived(() => {
    const grid: (InventoryItem | null)[] = new Array(TOTAL_SLOTS).fill(null);
    for (const item of inventory) {
      if (item.slot >= 0 && item.slot < TOTAL_SLOTS) {
        grid[item.slot] = item;
      }
    }
    return grid;
  });

  let sortedSpells = $derived(
    [...spells].sort((a, b) => a.slot - b.slot),
  );

  let selectedSpell = $derived(
    selectedSpellSlot !== null ? spells.find((s) => s.slot === selectedSpellSlot) ?? null : null,
  );

  function handleSlotClick(slot: number) {
    const item = slots()[slot];
    if (!item) {
      selectedSlot = null;
      hoveredItem = null;
      return;
    }
    if (selectedSlot === slot) {
      sendUseItemClick(slot);
      selectedSlot = null;
    } else {
      selectedSlot = slot;
      hoveredItem = item;
    }
  }

  function handleEquip() {
    if (selectedSlot !== null) {
      sendEquipItem(selectedSlot);
      selectedSlot = null;
    }
  }

  function handleDrop() {
    if (selectedSlot !== null) {
      const item = slots()[selectedSlot];
      if (item) sendDropItem(selectedSlot, item.amount);
      selectedSlot = null;
    }
  }

  function handleCastSpell() {
    if (selectedSpellSlot !== null) {
      sendAttackSpell(selectedSpellSlot);
    }
  }
</script>

<div class="flex h-full flex-col rounded-lg border border-white/10 bg-white/[2%]">
  <!-- Tabs -->
  <div class="flex border-b border-white/10">
    <button
      onclick={() => { activeTab = "inventory"; }}
      class="flex-1 px-2 py-1.5 text-[10px] font-medium transition
        {activeTab === 'inventory'
          ? 'bg-white/5 text-white border-b-2 border-amber-400'
          : 'text-stone-400 hover:text-stone-200'}"
    >
      Inventario
    </button>
    <button
      onclick={() => { activeTab = "spells"; }}
      class="flex-1 px-2 py-1.5 text-[10px] font-medium transition
        {activeTab === 'spells'
          ? 'bg-white/5 text-white border-b-2 border-amber-400'
          : 'text-stone-400 hover:text-stone-200'}"
    >
      Hechizos
    </button>
  </div>

  <div class="flex-1 min-h-0 overflow-y-auto p-2">
    {#if activeTab === "inventory"}
      <!-- Inventory grid -->
      <div class="grid gap-1" style="grid-template-columns: repeat({INV_COLS}, 1fr);">
        {#each slots() as item, idx}
          <button
            onclick={() => handleSlotClick(idx)}
            class="relative flex items-center justify-center rounded-md border transition aspect-square
              {item
                ? selectedSlot === item.slot
                  ? 'border-cyan-400/60 bg-cyan-400/10'
                  : 'border-white/10 bg-white/[3%] hover:border-white/20'
                : 'border-white/5 bg-white/[1%]'}"
            title={item ? `${item.name} (x${item.amount})` : `Slot ${idx + 1}`}
            onmouseenter={() => { if (item) hoveredItem = item; }}
            onmouseleave={() => { hoveredItem = null; }}
          >
            {#if item}
              <ItemGraphic
                graphicData={assetStore.getItemGraphic(item.grhIndex)}
                name={item.name}
                size={30}
              />
              {#if item.amount > 1}
                <span class="pointer-events-none absolute left-0.5 top-0 text-[8px] font-bold text-white drop-shadow-[0_1px_1px_#000]">
                  {item.amount}
                </span>
              {/if}
              {#if item.equipped}
                <span class="pointer-events-none absolute bottom-0 right-0 rounded-tl bg-amber-400 px-0.5 text-[7px] font-black text-black">
                  E
                </span>
              {/if}
            {/if}
          </button>
        {/each}
      </div>

      <!-- Hovered item tooltip -->
      {#if hoveredItem}
        <div class="mt-1.5 rounded-md border border-white/10 bg-white/[3%] px-2 py-1 text-[10px] text-stone-200">
          <p class="font-semibold text-white">{hoveredItem.name}</p>
          {#if hoveredItem.details}
            <p class="mt-0.5 whitespace-pre-wrap text-stone-400">{hoveredItem.details}</p>
          {/if}
        </div>
      {/if}

      <!-- Actions for selected item -->
      {#if selectedSlot !== null && slots()[selectedSlot]}
        <div class="mt-1.5 flex gap-1">
          <button onclick={() => { if (selectedSlot !== null) { sendUseItemClick(selectedSlot); selectedSlot = null; } }}
            class="flex-1 rounded-md border border-white/10 bg-white/5 px-2 py-1 text-[10px] font-medium text-stone-200 hover:bg-white/10 transition">
            Usar
          </button>
          <button onclick={handleEquip}
            class="flex-1 rounded-md border border-white/10 bg-white/5 px-2 py-1 text-[10px] font-medium text-stone-200 hover:bg-white/10 transition">
            Equipar
          </button>
          <button onclick={handleDrop}
            class="flex-1 rounded-md border border-red-500/20 bg-red-500/5 px-2 py-1 text-[10px] font-medium text-red-400 hover:bg-red-500/10 transition">
            Tirar
          </button>
        </div>
      {/if}

    {:else}
      <!-- Spells list -->
      <div class="max-h-52 overflow-y-auto rounded-md border border-white/10 bg-white/[1%]">
        {#if sortedSpells.length > 0}
          {#each sortedSpells as spell}
            <button
              onclick={() => { selectedSpellSlot = selectedSpellSlot === spell.slot ? null : spell.slot; }}
              class="flex w-full items-center px-2 text-left text-[10px] leading-6 select-none transition
                {selectedSpellSlot === spell.slot
                  ? 'bg-blue-500/20 text-white'
                  : 'bg-transparent text-stone-200 hover:bg-white/5'}"
            >
              <span class="truncate flex-1">{spell.name}</span>
              <span class="ml-1 text-[9px] text-cyan-300/60">{spell.manaRequired}mp</span>
            </button>
          {/each}
        {:else}
          <div class="px-2 py-4 text-center text-[10px] text-stone-500">
            Sin hechizos disponibles.
          </div>
        {/if}
      </div>

      {#if selectedSpell}
        <div class="mt-1.5 rounded-md border border-white/10 bg-white/[3%] px-2 py-1 text-[10px] text-stone-200">
          <p class="font-semibold text-white">{selectedSpell.name}</p>
          <p class="text-stone-500">Mana: {selectedSpell.manaRequired}</p>
        </div>
      {/if}

      <button
        onclick={handleCastSpell}
        disabled={!selectedSpell}
        class="mt-1.5 w-full rounded-md border border-white/10 bg-white/5 px-2 py-1.5 text-[10px] font-medium text-stone-200 hover:bg-white/10 transition disabled:cursor-not-allowed disabled:opacity-40"
      >
        Lanzar
      </button>
    {/if}
  </div>
</div>
