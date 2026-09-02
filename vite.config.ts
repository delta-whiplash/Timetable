import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte()],
  resolve: {
    alias: {
      $lib: "/src/lib"
    }
  },
  test: {
    environment: "node",
    include: ["src/**/*.test.ts"]
  },
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/target/**"]
    }
  },
  clearScreen: false
});
