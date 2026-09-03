<script lang="ts">
  import { goto } from "$app/navigation";

  const sections = [
    { id: "equipment", label: "Equipamiento" },
    { id: "spells", label: "Hechizos" },
    { id: "npcs", label: "NPCs" },
    { id: "maps", label: "Mapas" },
    { id: "crafting", label: "Crafteo" },
  ];

  goto("/wiki/equipment");
</script>

<svelte:head>
  <title>Wiki | AOWeb</title>
</svelte:head>

<main class="mx-auto max-w-5xl px-4 py-8">
  <h1 class="mb-6 text-3xl font-bold text-stone-100">Wiki</h1>
  <nav class="flex flex-wrap gap-2">
    {#each sections as section}
      <a
        href="/wiki/{section.id}"
        class="rounded-lg border border-white/8 px-4 py-2 text-sm text-stone-300 transition hover:bg-white/5"
      >
        {section.label}
      </a>
    {/each}
  </nav>
</main>
