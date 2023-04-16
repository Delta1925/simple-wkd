import { defineConfig } from "astro/config";
import tailwind from "@astrojs/tailwind";

import critters from "astro-critters";

// https://astro.build/config
export default defineConfig({
  integrations: [tailwind(), critters({ critters: { preload: "swap" } })],
});
