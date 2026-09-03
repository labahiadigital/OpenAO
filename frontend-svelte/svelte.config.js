import adapter from "@sveltejs/adapter-cloudflare";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      routes: {
        include: ["/*"],
        exclude: ["<build>", "<prerendered>", "/static/*"],
      },
    }),
    alias: {
      $lib: "./src/lib",
      $game: "./src/lib/game",
      $components: "./src/lib/components",
    },
  },
};

export default config;
