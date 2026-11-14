import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vitest/config";

export default defineConfig({
  base: process.env.ARITY_BASE ?? "/",
  plugins: [svelte()],
  build: {
    target: "es2023",
    chunkSizeWarningLimit: 2048,
  },
  test: {
    environment: "jsdom",
    include: ["src/**/*.test.ts"],
  },
});
