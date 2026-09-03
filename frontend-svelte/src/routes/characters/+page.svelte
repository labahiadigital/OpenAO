<script lang="ts">
  import { browser } from "$app/environment";
  import type { AuthSession, AuthErrorResponse } from "$lib/auth";

  let session = $state<AuthSession | null>(null);
  let loading = $state(true);

  $effect(() => {
    if (!browser) { loading = false; return; }

    fetch("/api/auth/me", { cache: "no-store" })
      .then(async (res) => {
        if (!res.ok) return null;
        const data = (await res.json()) as AuthSession | AuthErrorResponse;
        return "error" in data ? null : data;
      })
      .then((s) => { session = s; })
      .catch(() => { session = null; })
      .finally(() => { loading = false; });
  });
</script>

<svelte:head>
  <title>Personajes | AOWeb</title>
</svelte:head>

<main class="mx-auto max-w-5xl px-4 py-8">
  <h1 class="mb-6 text-3xl font-bold text-stone-100">Personajes</h1>

  {#if loading}
    <p class="text-stone-400">Cargando...</p>
  {:else if !session}
    <p class="text-stone-400">
      <a href="/login" class="text-amber-300 hover:underline">Ingresá</a> para ver tus personajes.
    </p>
  {:else if session.characters.length === 0}
    <p class="text-stone-400">
      No tenés personajes.
      <a href="/createcharacter" class="text-amber-300 hover:underline">Creá uno</a>.
    </p>
  {:else}
    <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
      {#each session.characters as char}
        <div class="rounded-xl border border-white/8 bg-white/3 p-4">
          <h2 class="text-lg font-semibold text-stone-100">{char.name}</h2>
          <p class="text-sm text-stone-400">
            Nivel {char.level} - {char.className} - {char.raceName}
          </p>
          {#if char.clanName}
            <p class="mt-1 text-sm text-amber-300/80">{char.clanName}</p>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</main>
