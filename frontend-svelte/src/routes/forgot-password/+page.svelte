<script lang="ts">
  let email = $state("");
  let error = $state("");
  let success = $state(false);
  let loading = $state(false);

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    error = "";
    loading = true;

    try {
      const response = await fetch("/api/auth/request-password-reset", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email }),
      });

      if (!response.ok) {
        const data = await response.json();
        error = data.error || "Error al solicitar recuperación";
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
  <title>Recuperar contraseña | AOWeb</title>
</svelte:head>

<main class="mx-auto flex min-h-[80vh] max-w-md items-center justify-center px-4">
  <div class="w-full rounded-2xl border border-white/8 bg-white/3 p-8">
    <h1 class="mb-6 text-center text-2xl font-bold text-stone-100">
      Recuperar contraseña
    </h1>

    {#if success}
      <p class="text-center text-stone-300">
        Si el email existe, recibirás un enlace para restablecer tu contraseña.
      </p>
    {:else}
      {#if error}
        <div class="mb-4 rounded-lg bg-red-500/10 px-4 py-3 text-sm text-red-400">
          {error}
        </div>
      {/if}

      <form onsubmit={handleSubmit} class="flex flex-col gap-4">
        <div>
          <label for="email" class="mb-1 block text-sm text-stone-400">Email</label>
          <input
            id="email"
            type="email"
            bind:value={email}
            required
            class="w-full rounded-lg border border-white/10 bg-white/5 px-4 py-2.5 text-stone-100 placeholder-stone-500 focus:border-amber-300/50 focus:outline-none"
            placeholder="tu@email.com"
          />
        </div>

        <button
          type="submit"
          disabled={loading}
          class="mt-2 rounded-lg bg-amber-300 px-4 py-2.5 font-semibold text-stone-950 transition hover:bg-amber-200 disabled:opacity-50"
        >
          {loading ? "Enviando..." : "Enviar enlace"}
        </button>
      </form>

      <p class="mt-6 text-center text-sm text-stone-400">
        <a href="/login" class="text-amber-300 hover:underline">Volver al login</a>
      </p>
    {/if}
  </div>
</main>
