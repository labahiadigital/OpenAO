<script lang="ts">
  import type { InventoryItem, SpellEntry } from "$lib/game/lib/aowProtocol";
  import { gameState } from "$lib/game/state/gameState.svelte";
  import { sendUseItemClick, sendAttackSpell, sendAttackMelee, sendAttackRange, sendDialog } from "$lib/game/session/outgoingRequests";

  type MacroTargetType = "item" | "spell" | "command" | "melee" | "range";

  type StoredMacro = {
    keyCode: string;
    label: string;
    targetType: MacroTargetType;
    targetSlot: number;
    targetId: number;
    command?: string;
  };

  const MACRO_SLOT_COUNT = 12;

  let macros = $state<(StoredMacro | null)[]>(loadMacros());
  let editingIndex = $state<number | null>(null);
  let draftType = $state<MacroTargetType>("item");
  let draftSlot = $state<number>(0);
  let draftCommand = $state("");
  let listeningForKey = $state(false);
  let draftKeyCode = $state("");

  const h = $derived(gameState.hud);
  const items = $derived(h.inventory.filter(i => i.idItem > 0).sort((a, b) => a.slot - b.slot));
  const spells = $derived(h.spells.filter(s => s.idSpell > 0).sort((a, b) => a.slot - b.slot));

  function loadMacros(): (StoredMacro | null)[] {
    try {
      const stored = localStorage.getItem("openao_macros");
      if (stored) {
        const parsed = JSON.parse(stored) as (StoredMacro | null)[];
        if (Array.isArray(parsed) && parsed.length === MACRO_SLOT_COUNT) return parsed;
      }
    } catch { /* ignore */ }
    return Array(MACRO_SLOT_COUNT).fill(null);
  }

  function saveMacros() {
    try { localStorage.setItem("openao_macros", JSON.stringify(macros)); } catch { /* ignore */ }
  }

  function executeMacro(macro: StoredMacro) {
    if (!macro) return;
    switch (macro.targetType) {
      case "item": {
        const item = items.find(i => i.idItem === macro.targetId) ?? items.find(i => i.slot === macro.targetSlot);
        if (item) sendUseItemClick(item.slot);
        break;
      }
      case "spell": {
        const spell = spells.find(s => s.idSpell === macro.targetId) ?? spells.find(s => s.slot === macro.targetSlot);
        if (spell) sendAttackSpell(spell.slot);
        break;
      }
      case "melee":
        sendAttackMelee();
        break;
      case "range":
        sendAttackRange();
        break;
      case "command":
        if (macro.command) sendDialog(macro.command);
        break;
    }
  }

  function handleGlobalKeydown(e: KeyboardEvent) {
    if (editingIndex !== null) return;

    const target = e.target as HTMLElement;
    if (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable) return;

    for (const macro of macros) {
      if (macro && macro.keyCode === e.code) {
        e.preventDefault();
        executeMacro(macro);
        return;
      }
    }
  }

  function handleSlotClick(index: number) {
    const macro = macros[index];
    if (macro) {
      executeMacro(macro);
    } else {
      openEditor(index);
    }
  }

  function openEditor(index: number) {
    editingIndex = index;
    const existing = macros[index];
    if (existing) {
      draftType = existing.targetType;
      draftSlot = existing.targetSlot;
      draftKeyCode = existing.keyCode;
      draftCommand = existing.command ?? "";
    } else {
      draftType = "item";
      draftSlot = 0;
      draftKeyCode = `F${index + 1}`;
      draftCommand = "";
    }
    listeningForKey = false;
  }

  function saveSlot() {
    if (editingIndex === null) return;
    const labelMap: Record<MacroTargetType, string> = {
      item: items.find(i => i.slot === draftSlot)?.name ?? `Item #${draftSlot}`,
      spell: spells.find(s => s.slot === draftSlot)?.name ?? `Spell #${draftSlot}`,
      command: draftCommand.slice(0, 20),
      melee: "Melee",
      range: "Rango",
    };

    let targetId = 0;
    if (draftType === "item") targetId = items.find(i => i.slot === draftSlot)?.idItem ?? 0;
    if (draftType === "spell") targetId = spells.find(s => s.slot === draftSlot)?.idSpell ?? 0;

    macros[editingIndex] = {
      keyCode: draftKeyCode,
      label: labelMap[draftType],
      targetType: draftType,
      targetSlot: draftSlot,
      targetId,
      command: draftType === "command" ? draftCommand : undefined,
    };
    saveMacros();
    editingIndex = null;
  }

  function clearSlot() {
    if (editingIndex === null) return;
    macros[editingIndex] = null;
    saveMacros();
    editingIndex = null;
  }

  function handleEditorKeydown(e: KeyboardEvent) {
    if (!listeningForKey) return;
    e.preventDefault();
    e.stopPropagation();
    draftKeyCode = e.code;
    listeningForKey = false;
  }

  function getSlotLabel(index: number): string {
    return `F${index + 1}`;
  }

  function getMacroDisplay(macro: StoredMacro | null): string {
    if (!macro) return "";
    return macro.label || macro.targetType;
  }
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

<div class="flex gap-0.5 rounded-lg border border-white/5 bg-stone-950/80 p-1 backdrop-blur-sm">
  {#each macros as macro, i}
    <button
      class="group relative flex h-10 w-10 flex-col items-center justify-center rounded border transition
        {macro ? 'border-white/10 bg-white/[3%] hover:bg-white/[6%]' : 'border-white/5 bg-transparent hover:bg-white/[3%]'}"
      onclick={() => handleSlotClick(i)}
      oncontextmenu={(e) => { e.preventDefault(); openEditor(i); }}
      title="{macro ? `${macro.label} [${macro.keyCode}] (click derecho para editar)` : `F${i + 1} (click derecho para asignar)`}"
    >
      {#if macro}
        <span class="text-[9px] font-medium text-stone-200 leading-none truncate max-w-[38px] px-0.5">
          {getMacroDisplay(macro)}
        </span>
      {/if}
      <span class="text-[8px] text-stone-500 {macro ? '' : 'mt-0.5'}">{getSlotLabel(i)}</span>
    </button>
  {/each}
</div>

<!-- Editor popup -->
{#if editingIndex !== null}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="fixed inset-0 z-[95] flex items-center justify-center bg-black/50 backdrop-blur-sm"
    role="dialog" aria-modal="true" tabindex="-1"
    onclick={(e) => { if (e.target === e.currentTarget) editingIndex = null; }}
    onkeydown={(e) => { if (e.key === 'Escape') editingIndex = null; }}>
    <div class="w-80 rounded-xl border border-white/10 bg-stone-950 p-4 shadow-2xl text-stone-100 space-y-3"
      onkeydown={handleEditorKeydown}>
      <p class="text-sm font-semibold">Macro Slot {editingIndex !== null ? editingIndex + 1 : ''}</p>

      <!-- Key binding -->
      <div class="space-y-1">
        <label for="macro-key" class="text-[10px] uppercase text-stone-500">Tecla</label>
        <div class="flex gap-2 items-center">
          <span class="rounded bg-white/5 px-3 py-1.5 text-xs text-stone-300 font-mono min-w-[60px] text-center">{draftKeyCode || '(ninguna)'}</span>
          <button
            class="rounded border border-white/10 px-2 py-1 text-[10px] text-stone-400 hover:text-white transition
              {listeningForKey ? 'border-amber-500/50 text-amber-400' : ''}"
            onclick={() => listeningForKey = !listeningForKey}>
            {listeningForKey ? 'Presioná una tecla...' : 'Cambiar'}
          </button>
        </div>
      </div>

      <!-- Type -->
      <div class="space-y-1">
        <label for="macro-type" class="text-[10px] uppercase text-stone-500">Tipo</label>
        <select id="macro-type" bind:value={draftType}
          class="w-full rounded bg-white/5 border border-white/10 px-2 py-1.5 text-xs text-stone-200">
          <option value="item">Ítem</option>
          <option value="spell">Hechizo</option>
          <option value="melee">Ataque Melee</option>
          <option value="range">Ataque Rango</option>
          <option value="command">Comando</option>
        </select>
      </div>

      <!-- Target -->
      {#if draftType === "item"}
        <div class="space-y-1">
          <label for="macro-item-slot" class="text-[10px] uppercase text-stone-500">Ítem</label>
          <select id="macro-item-slot" bind:value={draftSlot}
            class="w-full rounded bg-white/5 border border-white/10 px-2 py-1.5 text-xs text-stone-200">
            {#each items as item}
              <option value={item.slot}>{item.name} (slot {item.slot + 1})</option>
            {/each}
          </select>
        </div>
      {:else if draftType === "spell"}
        <div class="space-y-1">
          <label for="macro-spell-slot" class="text-[10px] uppercase text-stone-500">Hechizo</label>
          <select id="macro-spell-slot" bind:value={draftSlot}
            class="w-full rounded bg-white/5 border border-white/10 px-2 py-1.5 text-xs text-stone-200">
            {#each spells as spell}
              <option value={spell.slot}>{spell.name} (slot {spell.slot + 1})</option>
            {/each}
          </select>
        </div>
      {:else if draftType === "command"}
        <div class="space-y-1">
          <label for="macro-command" class="text-[10px] uppercase text-stone-500">Comando</label>
          <input id="macro-command" type="text" bind:value={draftCommand} placeholder="/meditar"
            class="w-full rounded bg-white/5 border border-white/10 px-2 py-1.5 text-xs text-stone-200 placeholder-stone-600" />
        </div>
      {/if}

      <div class="flex gap-2 pt-1">
        <button onclick={saveSlot}
          class="flex-1 rounded-lg bg-amber-500/20 border border-amber-500/30 px-3 py-1.5 text-xs text-amber-300 hover:bg-amber-500/30 transition">
          Guardar
        </button>
        <button onclick={clearSlot}
          class="rounded-lg border border-red-500/20 px-3 py-1.5 text-xs text-red-400 hover:bg-red-500/10 transition">
          Limpiar
        </button>
        <button onclick={() => editingIndex = null}
          class="rounded-lg border border-white/10 px-3 py-1.5 text-xs text-stone-400 hover:text-white transition">
          Cancelar
        </button>
      </div>
    </div>
  </div>
{/if}
