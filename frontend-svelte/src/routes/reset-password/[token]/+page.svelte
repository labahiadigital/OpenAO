<script lang="ts">
  import { page } from "$app/state";
  import { goto } from "$app/navigation";

  let password = $state("");
  let confirmPassword = $state("");
  let error = $state("");
  let success = $state(false);
  let loading = $state(false);

  let token = $derived(page.params.token);

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    error = "";

    if (password !== confirmPassword) {
      error = "Las contraseñas no coinciden";
      return;
    }

    loading = true;

    try {
      const response = await fetch("/api/auth/reset-password", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ token, password }),
      });

      if (!response.ok) {
        const data = await response.json();
        error = data.error || "Error al restablecer contraseña";
        return;
      }

      success = true;
    } catch {
      error = "Error de conexión";
    } finally {
      loading = false;
    }
  }
</script>

<svelte:head>
  <title>Restablecer contraseña | AOWeb</title>
</svelte:head>

<main class="mx-auto flex min-h-[80vh] max-w-md items-center justify-center px-4">
  <div class="w-full rounded-2xl border border-white/8 bg-white/3 p-8">
    <h1 class="mb-6 text-center text-2xl font-bold text-stone-100">
      Restablecer contraseña
    </h1>

    {#if success}
      <div class="text-center">
        <p class="mb-4 text-stone-300">Tu contraseña fue restablecida.</p>
        <a
          href="/login"
          class="inline-flex rounded-lg bg-amber-300 px-6 py-2.5 font-semibold text-stone-950 transition hover:bg-amber-200"
        >
          Ingresar
        </a>
      </div>
    {:else}
      {#if error}
        <div class="mb-4 rounded-lg bg-red-500/10 px-4 py-3 text-sm text-red-400">
          {error}
        </div>
      {/if}

      <form onsubmit={handleSubmit} class="flex flex-col gap-4">
        <div>
          <label for="password" class="mb-1 block text-sm text-stone-400">Nueva contraseña</label>
          <input
            id="password"
            type="password"
            bind:value={password}
            required
            class="w-full rounded-lg border border-white/10 bg-white/5 px-4 py-2.5 text-stone-100 placeholder-stone-500 focus:border-amber-300/50 focus:outline-none"
          />
        </div>

        <div>
          <label for="confirmPassword" class="mb-1 block text-sm text-stone-400">Confirmar contraseña</label>
          <input
            id="confirmPassword"
            type="password"
            bind:value={confirmPassword}
            required
            class="w-full rounded-lg border border-white/10 bg-white/5 px-4 py-2.5 text-stone-100 placeholder-stone-500 focus:border-amber-300/50 focus:outline-none"
          />
        </div>

        <button
          type="submit"
          disabled={loading}
          class="mt-2 rounded-lg bg-amber-300 px-4 py-2.5 font-semibold text-stone-950 transition hover:bg-amber-200 disabled:opacity-50"
        >
          {loading ? "Restableciendo..." : "Restablecer"}
        </button>
      </form>
    {/if}
  </div>
</main>
