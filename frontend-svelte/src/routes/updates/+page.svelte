<script lang="ts">
  const updates = [
    {
      date: "2026-09-01",
      title: "Migración a Rust Game Server",
      content: "El servidor de juego ha sido completamente migrado de Node.js a Rust con tokio + WebSocket. Mejora significativa en rendimiento y estabilidad.",
      tags: ["backend", "rust"],
    },
    {
      date: "2026-09-01",
      title: "Sistema de Inventario",
      content: "Implementado el sistema completo de inventario: equipar, usar y tirar items. Los items se persisten en SQLite.",
      tags: ["gameplay"],
    },
    {
      date: "2026-09-01",
      title: "NPCs y Combate",
      content: "Los NPCs ahora aparecen en los mapas y pueden ser atacados. Sistema de respawn automático cada 30 segundos.",
      tags: ["gameplay", "npcs"],
    },
    {
      date: "2026-09-01",
      title: "Teletransporte entre mapas",
      content: "Implementado el comando /tp para cambiar de mapa. Los jugadores pueden moverse entre distintas zonas del mundo.",
      tags: ["gameplay"],
    },
    {
      date: "2026-09-01",
      title: "Chat en tiempo real",
      content: "Los mensajes de chat ahora se transmiten a todos los jugadores en el mismo mapa vía broadcast.",
      tags: ["social"],
    },
    {
      date: "2026-09-01",
      title: "API REST de Autenticación",
      content: "Sistema de registro e inicio de sesión completamente funcional con tickets de juego.",
      tags: ["backend", "auth"],
    },
  ];
</script>

<svelte:head>
  <title>Novedades | AOWeb</title>
</svelte:head>

<main class="mx-auto max-w-5xl px-4 py-8">
  <h1 class="mb-6 text-3xl font-bold text-stone-100">Novedades</h1>

  <div class="flex flex-col gap-6">
    {#each updates as update}
      <article class="rounded-xl border border-white/8 bg-white/3 p-6">
        <div class="mb-2 flex items-center gap-3">
          <time class="text-xs text-stone-500">{update.date}</time>
          <div class="flex gap-1.5">
            {#each update.tags as tag}
              <span class="rounded-full bg-amber-300/10 px-2 py-0.5 text-xs text-amber-300">{tag}</span>
            {/each}
          </div>
        </div>
        <h2 class="mb-2 text-lg font-semibold text-stone-100">{update.title}</h2>
        <p class="text-sm text-stone-400 leading-relaxed">{update.content}</p>
      </article>
    {/each}
  </div>
</main>
