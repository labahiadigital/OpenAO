<script lang="ts">
  import { sendCloseTrade, sendBuyItem, sendSellItem } from "$lib/game/session/outgoingRequests";
  import type { TradeItem } from "$lib/game/lib/aowProtocol";

  let {
    merchantItems = [],
    onClose,
  }: {
    merchantItems: TradeItem[];
    onClose: () => void;
  } = $props();

  let selectedIdx = $state<number | null>(null);
  let buyAmount = $state(1);

  function handleClose() {
    sendCloseTrade();
    onClose();
  }

  function handleBuy() {
    if (selectedIdx !== null) {
      sendBuyItem(selectedIdx, buyAmount);
    }
  }
</script>

<div class="pointer-events-auto fixed inset-0 z-50 flex items-center justify-center bg-black/60">
  <div class="w-[500px] max-h-[80vh] overflow-hidden rounded-xl border border-white/10 bg-[#0a0e15]">
    <div class="flex items-center justify-between border-b border-white/8 px-6 py-4">
      <h2 class="text-lg font-semibold text-stone-100">Comercio NPC</h2>
      <button onclick={handleClose} class="text-stone-400 hover:text-stone-200 text-xl">&times;</button>
    </div>

    <div class="max-h-[60vh] overflow-y-auto p-4">
      {#each merchantItems as item, i}
        <button
          onclick={() => { selectedIdx = i; buyAmount = 1; }}
          class="mb-2 w-full rounded-lg border p-3 text-left transition {selectedIdx === i
            ? 'border-amber-300/40 bg-amber-300/10'
            : 'border-white/8 bg-white/3 hover:border-white/15'}"
        >
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-3">
              <div class="flex h-10 w-10 items-center justify-center rounded border border-white/10 bg-white/5 text-xs text-stone-400">
                {item.grhIndex}
              </div>
              <div>
                <p class="text-sm font-medium text-stone-200">{item.name}</p>
                {#if item.details}
                  <p class="text-xs text-stone-400">{item.details}</p>
                {/if}
              </div>
            </div>
            <p class="text-sm text-amber-300">{item.value} oro</p>
          </div>
        </button>
      {/each}

      {#if merchantItems.length === 0}
        <p class="text-center text-sm text-stone-500 py-8">No hay items disponibles.</p>
      {/if}
    </div>

    {#if selectedIdx !== null}
      <div class="border-t border-white/8 px-6 py-4 flex items-center gap-3">
        <label for="trade-amount" class="text-xs text-stone-400">Cantidad:</label>
        <input
          id="trade-amount"
          type="number"
          min="1"
          max="100"
          bind:value={buyAmount}
          class="w-16 rounded bg-white/8 px-2 py-1 text-sm text-stone-200 border border-white/10"
        />
        <button
          onclick={handleBuy}
          class="flex-1 rounded-lg bg-amber-300/20 px-4 py-2 text-sm font-medium text-amber-300 transition hover:bg-amber-300/30"
        >
          Comprar ({(merchantItems[selectedIdx]?.value ?? 0) * buyAmount} oro)
        </button>
      </div>
    {/if}
  </div>
</div>
