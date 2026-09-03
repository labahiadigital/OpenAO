<script lang="ts">
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";

  let error = $state("");
  let loading = $state(true);

  const apiBase = import.meta.env.VITE_API_BASE_URL || "http://localhost:7667";

  onMount(async () => {
    const joinToken = page.params.joinToken;

    if (!joinToken) {
      error = "Link de sala inválido";
      loading = false;
      return;
    }

    try {
      const res = await fetch(`${apiBase}/api/arenas/join/${joinToken}`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        credentials: "include",
      });

      if (!res.ok) {
        const data = await res.json();
        error = data.error || "No se pudo unir a la sala";
        loading = false;
        return;
      }

      const data = await res.json();
      goto(`/arenas?room=${data.roomId}`, { replaceState: true });
    } catch {
      error = "Error de conexión";
      loading = false;
    }
  });
</script>

<svelte:head>
  <title>Unirse a Arena | OpenAO</title>
</svelte:head>

<main class="flex min-h-[80vh] items-center justify-center px-4">
  {#if loading}
    <div class="flex items-center gap-3 text-stone-400">
      <div class="h-8 w-8 animate-spin rounded-full border-2 border-amber-300 border-t-transparent"></div>
      <span>Uniéndose a la sala...</span>
    </div>
  {:else if error}
    <div class="max-w-md rounded-xl border border-white/8 bg-white/3 p-8 text-center">
      <div class="mb-4 text-4xl">😕</div>
      <h1 class="mb-2 text-xl font-bold text-stone-100">No se pudo unir</h1>
      <p class="mb-6 text-sm text-stone-400">{error}</p>
      <a href="/arenas" class="rounded-lg bg-amber-300 px-6 py-2.5 font-semibold text-stone-950 transition hover:bg-amber-200">
        Ver Arenas
      </a>
    </div>
  {/if}
</main>
