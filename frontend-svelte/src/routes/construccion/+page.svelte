<script lang="ts">
  import { browser } from "$app/environment";

  let EditorView: typeof import("$lib/components/editor/EditorView.svelte").default | undefined = $state();

  if (browser) {
    import("$lib/components/editor/EditorView.svelte").then((m) => {
      EditorView = m.default;
    });
  }
</script>

<svelte:head>
  <title>Constructor de Mapas | AOWeb</title>
</svelte:head>

{#if EditorView}
  <EditorView />
{:else}
  <div class="flex h-screen items-center justify-center bg-[#05080d]">
    <p class="text-stone-400">Cargando editor...</p>
  </div>
{/if}
