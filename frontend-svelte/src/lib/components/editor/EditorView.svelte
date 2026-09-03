<script lang="ts">
  import PixiApp from "$lib/components/game/PixiApp.svelte";
  import { editorStore, type EditorTool, type EditorLayer } from "$lib/editor/editorStore.svelte";

  const tools: { id: EditorTool; label: string }[] = [
    { id: "select", label: "Seleccionar" },
    { id: "paint", label: "Pintar" },
    { id: "erase", label: "Borrar" },
    { id: "fill", label: "Rellenar" },
    { id: "block", label: "Bloquear" },
    { id: "npc", label: "NPC" },
    { id: "spawn", label: "Spawn" },
    { id: "tp", label: "Teleport" },
  ];

  const layers: { id: EditorLayer; label: string }[] = [
    { id: "ground", label: "Suelo" },
    { id: "objects", label: "Objetos" },
    { id: "roofs", label: "Techos" },
    { id: "triggers", label: "Triggers" },
    { id: "blocked", label: "Bloqueo" },
  ];
</script>

<div class="flex h-screen bg-[#05080d]">
  <!-- Toolbar -->
  <div class="flex w-56 flex-col border-r border-white/8 bg-[#080c14]">
    <div class="border-b border-white/8 p-4">
      <h2 class="text-sm font-semibold text-stone-200">Constructor</h2>
      <p class="text-xs text-stone-400">Mapa #{editorStore.mapId}</p>
    </div>

    <div class="border-b border-white/8 p-3">
      <p class="mb-2 text-xs font-medium text-stone-400">Herramientas</p>
      <div class="grid grid-cols-2 gap-1">
        {#each tools as tool}
          <button
            onclick={() => editorStore.setTool(tool.id)}
            class="rounded px-2 py-1.5 text-xs transition {editorStore.currentTool === tool.id
              ? 'bg-amber-300/15 text-amber-300'
              : 'text-stone-400 hover:bg-white/5 hover:text-stone-200'}"
          >
            {tool.label}
          </button>
        {/each}
      </div>
    </div>

    <div class="border-b border-white/8 p-3">
      <p class="mb-2 text-xs font-medium text-stone-400">Capas</p>
      <div class="flex flex-col gap-1">
        {#each layers as layer}
          <button
            onclick={() => editorStore.setLayer(layer.id)}
            class="rounded px-2 py-1.5 text-left text-xs transition {editorStore.currentLayer === layer.id
              ? 'bg-amber-300/15 text-amber-300'
              : 'text-stone-400 hover:bg-white/5 hover:text-stone-200'}"
          >
            {layer.label}
          </button>
        {/each}
      </div>
    </div>

    <div class="mt-auto border-t border-white/8 p-3">
      <div class="flex gap-2">
        <button
          onclick={() => editorStore.undo()}
          disabled={!editorStore.canUndo}
          class="flex-1 rounded bg-white/8 px-2 py-1.5 text-xs text-stone-300 transition hover:bg-white/12 disabled:opacity-30"
        >
          Deshacer
        </button>
        <button
          onclick={() => editorStore.redo()}
          disabled={!editorStore.canRedo}
          class="flex-1 rounded bg-white/8 px-2 py-1.5 text-xs text-stone-300 transition hover:bg-white/12 disabled:opacity-30"
        >
          Rehacer
        </button>
      </div>
    </div>
  </div>

  <!-- Canvas -->
  <div class="flex-1">
    <PixiApp />
  </div>
</div>
