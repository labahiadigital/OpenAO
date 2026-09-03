<script lang="ts">
  import { onMount } from "svelte";

  type StatEntry = {
    timestamp: string;
    totalUsers: number;
    pveUsers: number;
    pvpUsers: number;
  };

  let stats = $state<StatEntry[]>([]);
  let loading = $state(true);
  let error = $state("");

  const apiBase = import.meta.env.VITE_API_BASE_URL || "http://localhost:7667";

  onMount(async () => {
    try {
      const res = await fetch(`${apiBase}/api/users-online-stats`);
      if (!res.ok) throw new Error("Failed to fetch stats");
      stats = await res.json();
    } catch {
      error = "No se pudieron cargar las estadísticas";
    } finally {
      loading = false;
    }
  });

  let currentOnline = $derived(stats.length > 0 ? stats[stats.length - 1] : null);
  let peakOnline = $derived(stats.reduce((max, s) => Math.max(max, s.totalUsers), 0));
</script>

<svelte:head>
  <title>Usuarios Online | OpenAO</title>
</svelte:head>

<main class="mx-auto max-w-4xl px-4 py-10">
  <h1 class="mb-6 text-2xl font-bold text-stone-100">Estadísticas de Usuarios Online</h1>

  {#if loading}
    <div class="flex items-center justify-center py-20 text-stone-400">
      <div class="h-8 w-8 animate-spin rounded-full border-2 border-amber-300 border-t-transparent"></div>
      <span class="ml-3">Cargando estadísticas...</span>
    </div>
  {:else if error}
    <div class="rounded-lg bg-red-500/10 px-4 py-3 text-sm text-red-400">{error}</div>
  {:else}
    <div class="mb-8 grid grid-cols-1 gap-4 sm:grid-cols-3">
      <div class="rounded-xl border border-white/8 bg-white/3 p-5 text-center">
        <div class="text-3xl font-bold text-amber-300">{currentOnline?.totalUsers ?? 0}</div>
        <div class="mt-1 text-sm text-stone-400">Online Ahora</div>
      </div>
      <div class="rounded-xl border border-white/8 bg-white/3 p-5 text-center">
        <div class="text-3xl font-bold text-emerald-400">{currentOnline?.pveUsers ?? 0}</div>
        <div class="mt-1 text-sm text-stone-400">PvE</div>
      </div>
      <div class="rounded-xl border border-white/8 bg-white/3 p-5 text-center">
        <div class="text-3xl font-bold text-red-400">{currentOnline?.pvpUsers ?? 0}</div>
        <div class="mt-1 text-sm text-stone-400">PvP</div>
      </div>
    </div>

    {#if stats.length > 0}
      <div class="rounded-xl border border-white/8 bg-white/3 p-5">
        <h2 class="mb-4 text-lg font-semibold text-stone-200">Pico máximo: {peakOnline} usuarios</h2>
        <div class="overflow-x-auto">
          <table class="w-full text-sm">
            <thead>
              <tr class="border-b border-white/8 text-left text-stone-400">
                <th class="px-3 py-2">Hora</th>
                <th class="px-3 py-2">Total</th>
                <th class="px-3 py-2">PvE</th>
                <th class="px-3 py-2">PvP</th>
              </tr>
            </thead>
            <tbody>
              {#each stats.slice(-24).reverse() as stat}
                <tr class="border-b border-white/5 text-stone-300">
                  <td class="px-3 py-2 text-stone-400">
                    {new Date(stat.timestamp).toLocaleTimeString("es-AR", { hour: "2-digit", minute: "2-digit" })}
                  </td>
                  <td class="px-3 py-2 font-medium text-amber-300">{stat.totalUsers}</td>
                  <td class="px-3 py-2 text-emerald-400">{stat.pveUsers}</td>
                  <td class="px-3 py-2 text-red-400">{stat.pvpUsers}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </div>
    {:else}
      <div class="rounded-lg bg-stone-800/50 px-4 py-8 text-center text-stone-500">
        No hay datos de usuarios online disponibles todavía.
      </div>
    {/if}
  {/if}
</main>
