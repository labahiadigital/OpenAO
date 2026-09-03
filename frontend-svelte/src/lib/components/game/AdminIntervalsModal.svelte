<script lang="ts">
  let { onClose }: { onClose: () => void } = $props();

  const API_BASE = import.meta.env.VITE_API_BASE_URL || "http://localhost:7667";

  let doubleExp = $state(false);
  let doubleGold = $state(false);
  let loading = $state(true);

  async function loadConfig() {
    try {
      const res = await fetch(`${API_BASE}/api/runtime-config`);
      if (res.ok) {
        const data = await res.json();
        doubleExp = data.double_exp ?? false;
        doubleGold = data.double_gold ?? false;
      }
    } catch { /* ignore */ }
    loading = false;
  }

  async function saveConfig() {
    try {
      await fetch(`${API_BASE}/api/runtime-config`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ double_exp: doubleExp, double_gold: doubleGold }),
      });
    } catch { /* ignore */ }
  }

  loadConfig();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="fixed inset-0 z-[92] flex items-center justify-center bg-black/60 px-4 py-6 backdrop-blur-sm">
  <div class="w-full max-w-sm rounded-2xl border border-white/10 bg-stone-950/95 text-stone-100 shadow-2xl">
    <div class="flex items-center justify-between border-b border-white/8 px-4 py-3">
      <p class="text-sm font-semibold text-white">Configuración del Servidor</p>
      <button
        class="rounded-lg p-1 text-stone-400 transition-colors hover:bg-white/10 hover:text-white"
        onclick={onClose}
      >✕</button>
    </div>

    <div class="space-y-4 p-4">
      {#if loading}
        <p class="text-sm text-stone-400">Cargando...</p>
      {:else}
        <label class="flex items-center justify-between text-sm">
          <span>Experiencia Doble</span>
          <input type="checkbox" class="h-4 w-4 accent-amber-500" bind:checked={doubleExp} />
        </label>

        <label class="flex items-center justify-between text-sm">
          <span>Oro Doble</span>
          <input type="checkbox" class="h-4 w-4 accent-amber-500" bind:checked={doubleGold} />
        </label>

        <button
          class="w-full rounded-lg bg-amber-600 px-3 py-2 text-sm font-medium text-white transition-colors hover:bg-amber-500"
          onclick={saveConfig}
        >
          Guardar
        </button>
      {/if}
    </div>
  </div>
</div>
