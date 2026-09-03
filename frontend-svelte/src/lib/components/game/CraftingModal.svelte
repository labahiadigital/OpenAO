<script lang="ts">
  import { sendCraftItem } from "$lib/game/session/outgoingRequests";
  import type { CraftingState } from "@openao/protocol";

  let { craftingState, onClose }: { craftingState: CraftingState; onClose: () => void } = $props();

  let selectedRecipe = $state<number | null>(null);

  function handleCraft() {
    if (selectedRecipe !== null) {
      sendCraftItem(selectedRecipe);
    }
  }
</script>

<div class="pointer-events-auto fixed inset-0 z-50 flex items-center justify-center bg-black/60">
  <div class="w-[500px] max-h-[80vh] overflow-hidden rounded-xl border border-white/10 bg-[#0a0e15]">
    <div class="flex items-center justify-between border-b border-white/8 px-6 py-4">
      <h2 class="text-lg font-semibold text-stone-100">
        {craftingState.title}
      </h2>
      <button onclick={onClose} class="text-stone-400 hover:text-stone-200">&times;</button>
    </div>

    <div class="max-h-[60vh] overflow-y-auto p-4">
      {#each craftingState.recipes as recipe, i}
        <button
          onclick={() => { selectedRecipe = i; }}
          class="mb-2 w-full rounded-lg border p-3 text-left transition {selectedRecipe === i
            ? 'border-amber-300/40 bg-amber-300/10'
            : 'border-white/8 bg-white/3 hover:border-white/15'}"
        >
          <div class="flex items-center gap-3">
            <div class="flex h-10 w-10 items-center justify-center rounded border border-white/10 bg-white/5 text-xs text-stone-300">
              {recipe.grhIndex}
            </div>
            <div>
              <p class="text-sm font-medium text-stone-200">{recipe.name}</p>
              <p class="text-xs text-stone-400">{recipe.details}</p>
              {#if recipe.stats}
                <p class="text-xs text-amber-300/80">{recipe.stats}</p>
              {/if}
            </div>
          </div>

          {#if selectedRecipe === i}
            <div class="mt-2 flex flex-wrap gap-2 border-t border-white/5 pt-2">
              {#each recipe.materials as mat}
                <span class="rounded bg-white/5 px-2 py-0.5 text-xs {mat.owned >= mat.amount ? 'text-green-400' : 'text-red-400'}">
                  {mat.name}: {mat.owned}/{mat.amount}
                </span>
              {/each}
            </div>
          {/if}
        </button>
      {/each}
    </div>

    <div class="border-t border-white/8 px-6 py-4">
      <button
        onclick={handleCraft}
        disabled={selectedRecipe === null}
        class="w-full rounded-lg bg-amber-300/20 px-4 py-2 text-sm font-medium text-amber-300 transition hover:bg-amber-300/30 disabled:opacity-40"
      >
        Craftear
      </button>
    </div>
  </div>
</div>
