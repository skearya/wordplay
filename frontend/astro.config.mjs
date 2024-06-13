import solidJs from "@astrojs/solid-js";
import tailwind from "@astrojs/tailwind";
import { defineConfig } from "astro/config";

import node from "@astrojs/node";

// https://astro.build/config
export default defineConfig({
  output: "server",
  adapter: node({
    mode: "standalone",
  }),
  integrations: [
    tailwind({
      applyBaseStyles: false,
    }),
    solidJs(),
  ],
  vite: {
    envDir: "../",
    envPrefix: "PUBLIC",
  },
});
