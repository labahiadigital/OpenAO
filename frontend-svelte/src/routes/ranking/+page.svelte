<script lang="ts">
  let { data } = $props();

  type RankingEntry = {
    name: string;
    level: number;
    gold: number;
  };

  let rankings: RankingEntry[] = $derived(data.rankings ?? []);
</script>

<svelte:head>
  <title>Ranking | AOWeb</title>
</svelte:head>

<main class="mx-auto max-w-5xl px-4 py-8">
  <h1 class="mb-6 text-3xl font-bold text-stone-100">Ranking</h1>

  {#if rankings.length === 0}
    <p class="text-stone-400">No hay datos de ranking disponibles.</p>
  {:else}
    <div class="overflow-x-auto rounded-xl border border-white/8">
      <table class="w-full text-sm">
        <thead>
          <tr class="border-b border-white/8 text-left text-stone-400">
            <th class="px-4 py-3">#</th>
            <th class="px-4 py-3">Nombre</th>
            <th class="px-4 py-3">Nivel</th>
            <th class="px-4 py-3">Oro</th>
          </tr>
        </thead>
        <tbody>
          {#each rankings as entry, i}
            <tr class="border-b border-white/5 text-stone-200 hover:bg-white/3">
              <td class="px-4 py-3 font-semibold text-amber-300">{i + 1}</td>
              <td class="px-4 py-3 font-medium">{entry.name}</td>
              <td class="px-4 py-3">{entry.level}</td>
              <td class="px-4 py-3 text-yellow-400">{entry.gold.toLocaleString()}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</main>
