import { defineConfig } from "@solidjs/start/config";
import path from 'node:path'

export default defineConfig({
  ssr: false,
  server: { preset: "static" },
  vite: {
    envDir: "../",
    envPrefix: "PUBLIC",
    resolve: {
      alias: {
        '~/': path.resolve(import.meta.dirname, './src'),
        '@game': path.resolve(import.meta.dirname, "./src/game")
      }
    }
  },
});
