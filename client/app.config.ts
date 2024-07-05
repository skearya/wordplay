import { defineConfig } from "@solidjs/start/config";
import { resolve } from "node:path";

export default defineConfig({
  server: { preset: "static" },
  vite: {
    envDir: "../",
    envPrefix: "PUBLIC",
    resolve: {
      alias: {
        "~/": resolve(import.meta.dirname, "./src"),
      },
    },
  },
});
