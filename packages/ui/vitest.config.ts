import { defineConfig } from "vitest/config";
import vueJsx from "@vitejs/plugin-vue-jsx";

export default defineConfig({
  plugins: [vueJsx()],
  css: {
    preprocessorOptions: {
      // vite 8 defaults to sass-embedded; use the `sass` package instead.
      scss: { api: "modern" },
    },
  },
  test: {
    environment: "happy-dom",
    include: ["tests/**/*.test.ts", "tests/**/*.test.tsx"],
  },
});

