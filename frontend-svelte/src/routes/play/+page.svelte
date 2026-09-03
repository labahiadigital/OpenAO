<script lang="ts">
  import { browser } from "$app/environment";

  let GameView: typeof import("$lib/components/game/GameView.svelte").default | undefined = $state();

  if (browser) {
    import("$lib/components/game/GameView.svelte").then((m) => {
      GameView = m.default;
    });
  }
</script>

<svelte:head>
  <title>Jugar | AOWeb</title>
</svelte:head>

{#if GameView}
  <GameView />
{:else}
  <div class="flex h-screen w-screen items-center justify-center bg-black">
    <p class="text-stone-400">Cargando juego...</p>
  </div>
{/if}
