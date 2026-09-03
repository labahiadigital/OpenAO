<script lang="ts">
  import { siteTitle } from "$lib/seo";

  const features = [
    { title: "Juego en el navegador", desc: "Sin descargas. Conecta y juega desde cualquier dispositivo." },
    { title: "Servidor en Rust", desc: "Backend de alto rendimiento con WebSocket y 60 TPS." },
    { title: "Combate en tiempo real", desc: "Pelea contra NPCs, sube de nivel y mejora tu equipamiento." },
    { title: "Mundo persistente", desc: "Tu progreso se guarda automáticamente en la base de datos." },
  ];
</script>

<svelte:head>
  <title>{siteTitle}</title>
</svelte:head>

<main class="mx-auto max-w-7xl px-4 py-16">
  <section class="mb-20 text-center">
    <h1 class="mb-4 text-5xl font-bold tracking-tight text-stone-100">
      AOWeb
    </h1>
    <p class="mb-8 text-lg text-stone-400">
      Argentum Online en tu navegador — servidor Rust, frontend moderno
    </p>
    <div class="flex items-center justify-center gap-4">
      <a
        href="/play"
        class="inline-flex items-center gap-2 rounded-xl bg-amber-300 px-8 py-3 text-lg font-semibold text-stone-950 transition hover:bg-amber-200"
      >
        Jugar ahora
      </a>
      <a
        href="/register"
        class="inline-flex items-center gap-2 rounded-xl border border-white/10 px-8 py-3 text-lg font-semibold text-stone-300 transition hover:bg-white/5"
      >
        Crear cuenta
      </a>
    </div>
  </section>

  <section class="grid gap-6 sm:grid-cols-2 lg:grid-cols-4">
    {#each features as feature}
      <div class="rounded-2xl border border-white/8 bg-white/3 p-6">
        <h3 class="mb-2 text-lg font-semibold text-amber-300">{feature.title}</h3>
        <p class="text-sm leading-relaxed text-stone-400">{feature.desc}</p>
      </div>
    {/each}
  </section>

  <section class="mt-20 text-center">
    <h2 class="mb-4 text-2xl font-bold text-stone-100">Estado del servidor</h2>
    <div class="mx-auto inline-flex items-center gap-2 rounded-full border border-green-500/20 bg-green-500/10 px-4 py-2">
      <span class="h-2 w-2 rounded-full bg-green-500 animate-pulse"></span>
      <span class="text-sm text-green-400">Online — WebSocket :7666 / API :7667</span>
    </div>
  </section>
</main>
