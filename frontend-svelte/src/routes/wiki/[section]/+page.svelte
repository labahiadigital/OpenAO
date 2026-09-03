<script lang="ts">
  import { page } from "$app/state";
  import type { WikiData } from "./+page.server";

  let { data } = $props();
  let wiki: WikiData | null = $derived(data.wiki);

  let section = $derived(page.params.section);

  const sectionLabels: Record<string, string> = {
    equipment: "Equipamiento",
    spells: "Hechizos",
    npcs: "NPCs",
    maps: "Mapas",
    crafting: "Crafteo",
    commands: "Comandos",
  };

  type DisplayItem = { name: string; description: string };

  const commandList: DisplayItem[] = [
    { name: "/online", description: "Muestra la cantidad de jugadores conectados." },
    { name: "/pos", description: "Muestra tu posición actual (mapa, x, y)." },
    { name: "/hp", description: "Muestra tus puntos de vida y maná." },
    { name: "/tp mapa x y", description: "Teletransporte a otro mapa en la posición indicada." },
    { name: "/revivir", description: "Revive si estás muerto (respawn en la posición de tu hogar)." },
    { name: "/help", description: "Lista de todos los comandos disponibles." },
    { name: "/global mensaje", description: "Envía un mensaje al chat global." },
    { name: "/w nombre mensaje", description: "Envía un susurro privado." },
    { name: "/meditar", description: "Meditar para recuperar maná más rápido." },
    { name: "/faccion armada|caos|salir", description: "Unirse o salir de una facción." },
    { name: "/party nombre", description: "Invitar a un jugador a tu grupo." },
    { name: "/clan crear|salir|info", description: "Gestión de clanes." },
    { name: "/misiones", description: "Ver misiones disponibles y activas." },
    { name: "/mascotas", description: "Ver tus mascotas." },
    { name: "/logros", description: "Ver logros obtenidos." },
    { name: "/comerciar nombre", description: "Iniciar comercio P2P." },
    { name: "/embarcar", description: "Subirse a un barco (requiere agua)." },
  ];

  const craftingGuide: DisplayItem[] = [
    { name: "Serrucho (Carpintería)", description: "Usa un serrucho con materiales de madera para crear objetos de carpintería." },
    { name: "Costurero (Sastrería)", description: "Usa un costurero con telas para crear vestimentas y armaduras ligeras." },
    { name: "Martillo (Herrería)", description: "Usa un martillo con lingotes para forjar armas y armaduras." },
    { name: "Fundición (/fundir)", description: "Convierte minerales en lingotes usando el comando /fundir." },
  ];

  let items: DisplayItem[] = $derived.by(() => {
    if (!section) return [];
    if (section === "commands") return commandList;
    if (section === "crafting") return craftingGuide;
    if (!wiki) return [];

    switch (section) {
      case "equipment":
        return wiki.items.map((it) => ({
          name: `${it.name} (#${it.id})`,
          description: `Tipo: ${it.type} | GRH: ${it.grhIndex}`,
        }));
      case "spells":
        return wiki.spells.map((sp) => ({
          name: `${sp.name} (#${sp.id})`,
          description: `Maná: ${sp.manaRequired} | Tipo: ${sp.type}`,
        }));
      case "npcs":
        return wiki.npcs.map((npc) => ({
          name: `${npc.name} (#${npc.id})`,
          description: `HP: ${npc.hp} | EXP: ${npc.exp}`,
        }));
      case "maps":
        return [{ name: "Mapas", description: "Consulta el mapa del mundo dentro del juego." }];
      default:
        return [];
    }
  });

  let sectionLabel = $derived(section ? (sectionLabels[section] ?? section) : "");
</script>

<svelte:head>
  <title>{sectionLabel} - Wiki | AOWeb</title>
</svelte:head>

<main class="mx-auto max-w-5xl px-4 py-8">
  <nav class="mb-6 flex flex-wrap gap-2">
    {#each Object.entries(sectionLabels) as [id, label]}
      <a
        href="/wiki/{id}"
        class="rounded-lg border px-4 py-2 text-sm transition {id === section
          ? 'border-amber-300/30 bg-amber-300/10 text-amber-300'
          : 'border-white/8 text-stone-300 hover:bg-white/5'}"
      >
        {label}
      </a>
    {/each}
  </nav>

  <h1 class="mb-6 text-3xl font-bold text-stone-100">{sectionLabel}</h1>

  {#if items.length > 0}
    <div class="flex flex-col gap-3">
      {#each items as item}
        <div class="rounded-xl border border-white/8 bg-white/3 p-4">
          <h3 class="text-sm font-semibold text-amber-300">{item.name}</h3>
          <p class="mt-1 text-sm text-stone-400">{item.description}</p>
        </div>
      {/each}
    </div>
  {:else if !wiki && section !== "commands" && section !== "crafting"}
    <p class="text-stone-400">No se pudo conectar al servidor para cargar los datos de la wiki.</p>
  {:else}
    <p class="text-stone-400">No hay contenido disponible para esta sección.</p>
  {/if}
</main>
