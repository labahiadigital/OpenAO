<script lang="ts">
  import "../app.css";
  import AppChrome from "$lib/components/AppChrome.svelte";

  let { children } = $props();
</script>

<svelte:head>
  <title>AOWeb Beta</title>
  <meta name="description" content="OpenAO — MMORPG de mundo abierto inspirado en Argentum Online. Juega gratis en tu navegador." />
  <meta name="application-name" content="AOWeb" />
  <meta property="og:title" content="AOWeb — Argentum Online en tu navegador" />
  <meta property="og:description" content="MMORPG de mundo abierto inspirado en Argentum Online. Combate, comercio, clanes y más." />
  <meta property="og:type" content="website" />
  <meta property="og:image" content="/static/imgs/logo-aoweb.png" />
  <meta name="twitter:card" content="summary_large_image" />
  <meta name="twitter:title" content="AOWeb — Argentum Online en tu navegador" />
  <meta name="twitter:description" content="MMORPG de mundo abierto inspirado en Argentum Online. Combate, comercio, clanes y más." />
  <meta name="twitter:image" content="/static/imgs/logo-aoweb.png" />
  <link rel="icon" href="/favicon.ico" />
  <link rel="apple-touch-icon" href="/static/imgs/logo-aoweb.png" />
  <link rel="manifest" href="/manifest.webmanifest" />
</svelte:head>

<AppChrome>
  {@render children()}
</AppChrome>
