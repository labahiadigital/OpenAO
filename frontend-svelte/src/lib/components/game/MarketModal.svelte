<script lang="ts">
  import type { MarketState, MarketListingGroupEntry } from "@openao/protocol";

  let { marketState, onClose }: { marketState: MarketState; onClose: () => void } = $props();

  let activeTab = $state<"buy" | "sell" | "my">("buy");
  let selectedGroup = $state<MarketListingGroupEntry | null>(null);
</script>

<div class="pointer-events-auto fixed inset-0 z-50 flex items-center justify-center bg-black/60">
  <div class="w-[700px] max-h-[85vh] overflow-hidden rounded-xl border border-white/10 bg-[#0a0e15]">
    <div class="flex items-center justify-between border-b border-white/8 px-6 py-4">
      <h2 class="text-lg font-semibold text-stone-100">
        Mercado - {marketState.npcName}
      </h2>
      <button onclick={onClose} class="text-stone-400 hover:text-stone-200">&times;</button>
    </div>

    <div class="flex border-b border-white/8">
      {#each [["buy", "Comprar"], ["sell", "Vender"], ["my", "Mis Anuncios"]] as [tab, label]}
        <button
          onclick={() => { activeTab = tab as "buy" | "sell" | "my"; }}
          class="flex-1 px-4 py-2.5 text-sm transition {activeTab === tab
            ? 'border-b-2 border-amber-300 text-amber-300'
            : 'text-stone-400 hover:text-stone-200'}"
        >
          {label}
        </button>
      {/each}
    </div>

    <div class="max-h-[60vh] overflow-y-auto p-4">
      {#if activeTab === "buy"}
        {#each marketState.listingGroups as group}
          <button
            onclick={() => { selectedGroup = group; }}
            class="mb-2 w-full rounded-lg border border-white/8 bg-white/3 p-3 text-left transition hover:border-white/15"
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="flex h-8 w-8 items-center justify-center rounded border border-white/10 bg-white/5 text-xs text-stone-300">
                  {group.itemGrhIndex}
                </div>
                <div>
                  <p class="text-sm text-stone-200">{group.itemName}</p>
                  <p class="text-xs text-stone-400">{group.totalListings} anuncios</p>
                </div>
              </div>
              <p class="text-sm text-amber-300">{group.minUnitPrice} c/u</p>
            </div>
          </button>
        {/each}
      {:else if activeTab === "my"}
        {#each marketState.myListings as listing}
          <div class="mb-2 rounded-lg border border-white/8 bg-white/3 p-3">
            <div class="flex items-center justify-between">
              <p class="text-sm text-stone-200">{listing.itemName} x{listing.quantity}</p>
              <span class="rounded px-2 py-0.5 text-xs {listing.status === 'active'
                ? 'bg-green-500/10 text-green-400'
                : 'bg-stone-500/10 text-stone-400'}">
                {listing.status}
              </span>
            </div>
          </div>
        {/each}
      {:else}
        <p class="text-sm text-stone-400">Selecciona items de tu inventario para vender.</p>
      {/if}
    </div>
  </div>
</div>
