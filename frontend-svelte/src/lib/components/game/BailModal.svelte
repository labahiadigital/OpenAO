<script lang="ts">
  import type { BailOffer } from "$lib/game/lib/aowProtocol";
  import { sendDialog } from "$lib/game/session/outgoingRequests";

  let { bail, onClose }: { bail: BailOffer; onClose: () => void } = $props();

  function payBail() {
    sendDialog("/fianza pagar");
    onClose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="fixed inset-0 z-[92] flex items-center justify-center bg-black/60 px-4 py-6 backdrop-blur-sm">
  <div class="w-full max-w-sm rounded-2xl border border-white/10 bg-stone-950/95 text-stone-100 shadow-2xl">
    <div class="flex items-center justify-between border-b border-white/8 px-4 py-3">
      <p class="text-sm font-semibold text-white">Fianza</p>
      <button onclick={onClose}
        class="rounded-md border border-white/10 px-2.5 py-1 text-xs text-stone-300 hover:border-white/20 hover:text-white transition">
        Cerrar
      </button>
    </div>

    <div class="space-y-3 p-4">
      <div class="grid grid-cols-2 gap-2 text-xs">
        <div class="text-stone-400">Asesinatos:</div>
        <div class="text-right text-stone-200">{bail.kills}</div>
        <div class="text-stone-400">Ciudadanos asesinados:</div>
        <div class="text-right text-stone-200">{bail.citizensKilled}</div>
        <div class="text-stone-400">Fianza acumulada:</div>
        <div class="text-right text-amber-300">{bail.fianza.toLocaleString()}</div>
        <div class="text-stone-400">Oro requerido:</div>
        <div class="text-right text-yellow-400 font-medium">{bail.goldRequired.toLocaleString()}</div>
        <div class="text-stone-400">Tu oro:</div>
        <div class="text-right {bail.goldAvailable >= bail.goldRequired ? 'text-green-400' : 'text-red-400'}">{bail.goldAvailable.toLocaleString()}</div>
      </div>

      {#if bail.canPay}
        <button onclick={payBail}
          class="w-full rounded-xl bg-amber-400 px-4 py-2.5 text-sm font-semibold text-stone-950 hover:bg-amber-300 transition">
          Pagar Fianza ({bail.goldRequired.toLocaleString()} oro)
        </button>
      {:else}
        <div class="rounded-lg bg-red-500/10 border border-red-500/20 p-3 text-center text-xs text-red-400">
          No tienes suficiente oro para pagar la fianza.
        </div>
      {/if}
    </div>
  </div>
</div>
