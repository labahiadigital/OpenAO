<script lang="ts">
  import { browser } from "$app/environment";

  let rooms = $state<Array<{ id: string; name: string; capacity: number; memberCount: number }>>([]);
  let loading = $state(true);

  $effect(() => {
    if (!browser) { loading = false; return; }

    fetch("/api/arenas")
      .then(async (res) => res.ok ? await res.json() : [])
      .then((data) => { rooms = data; })
      .catch(() => { rooms = []; })
      .finally(() => { loading = false; });
  });
</script>

<svelte:head>
  <title>Arenas | AOWeb</title>
</svelte:head>

<main class="mx-auto max-w-5xl px-4 py-8">
  <h1 class="mb-6 text-3xl font-bold text-stone-100">Arenas PvP</h1>

  {#if loading}
    <p class="text-stone-400">Cargando salas...</p>
  {:else if rooms.length === 0}
    <p class="text-stone-400">No hay salas públicas disponibles.</p>
  {:else}
    <div class="grid gap-4 sm:grid-cols-2">
      {#each rooms as room}
        <div class="rounded-xl border border-white/8 bg-white/3 p-4">
          <h2 class="text-lg font-semibold text-stone-100">{room.name}</h2>
          <p class="text-sm text-stone-400">
            {room.memberCount}/{room.capacity} jugadores
          </p>
          <a
            href="/arenas/join/{room.id}"
            class="mt-3 inline-flex rounded-lg bg-amber-300/10 px-4 py-2 text-sm font-medium text-amber-300 transition hover:bg-amber-300/20"
          >
            Unirse
          </a>
        </div>
      {/each}
    </div>
  {/if}
</main>
