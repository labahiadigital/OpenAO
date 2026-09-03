import tailwindcss from "@tailwindcss/vite";
import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";
import path from "path";

export default defineConfig({
  plugins: [tailwindcss(), sveltekit()],
  ssr: {
    noExternal: ["@openao/protocol"],
  },
  build: {
    target: "esnext",
    minify: "esbuild",
    sourcemap: false,
  },
  test: {
    include: ["src/**/*.test.ts"],
    alias: {
      "$lib": path.resolve(__dirname, "src/lib"),
    },
  },
});
