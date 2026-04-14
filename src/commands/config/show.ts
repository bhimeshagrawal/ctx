import { defineCommand, option } from "@bunli/core";
import { z } from "zod";
import { createPaths } from "../../config/paths.js";
import { loadConfig } from "../../config/load-config.js";
import { renderData } from "../../output/render.js";

export default defineCommand({
  name: "show",
  description: "Show the effective ctx configuration",
  options: {
    json: option(z.coerce.boolean().default(false), {
      description: "Print machine-readable output"
    })
  },
  handler: async ({ flags }) => {
    const paths = createPaths();
    const config = await loadConfig(paths);
    renderData(config, flags.json);
  }
});
